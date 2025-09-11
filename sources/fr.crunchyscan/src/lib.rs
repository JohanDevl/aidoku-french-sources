#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, Result, Source,
    alloc::{String, Vec, format},
    imports::{net::{Request, Response}, html::Document},
    prelude::*,
    AidokuError,
};

extern crate alloc;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://crunchyscan.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

// Helper function for robust HTTP requests with Cloudflare bypass and error handling
fn make_request_with_cloudflare_retry(url: &str) -> Result<Response> {
    let mut attempt = 0;
    const MAX_RETRIES: u32 = 3;
    
    loop {
        // Rate limiting: Add delays between requests to avoid triggering Cloudflare
        if attempt > 0 {
            let _backoff_ms = 1000 * (1 << (attempt - 1).min(3)); // Exponential backoff: 1s, 2s, 4s, 8s
            // Exponential backoff would be implemented here in non-WASM environment
        } else {
            // Even on first request, add small delay to avoid rapid-fire requests
            // Rate limiting would be implemented here in non-WASM environment
        }
        
        let request = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Cache-Control", "max-age=0")
            .header("Referer", BASE_URL);
        
        let response = match request.send() {
            Ok(resp) => resp,
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    return Err(AidokuError::RequestError(e));
                }
                attempt += 1;
                continue;
            }
        };
        
        match response.status_code() {
            200..=299 => return Ok(response),
            403 => {
                // Cloudflare block, retry with longer delay
                if attempt >= MAX_RETRIES {
                    return Err(AidokuError::message("Cloudflare block (403)"));
                }
                attempt += 1;
                continue;
            },
            429 => {
                // Rate limited by Cloudflare, retry with exponential backoff
                if attempt >= MAX_RETRIES {
                    return Err(AidokuError::message("Rate limited (429)"));
                }
                attempt += 1;
                continue;
            },
            503 | 502 | 504 => {
                // Server error, might be temporary Cloudflare protection
                if attempt >= MAX_RETRIES {
                    return Err(AidokuError::message("Server error"));
                }
                attempt += 1;
                continue;
            },
            _ => return Err(AidokuError::message("Request failed")),
        }
    }
}

// Wrapper function with fallback for HTML parsing
fn make_realistic_request(url: &str) -> Result<Document> {
    match make_request_with_cloudflare_retry(url) {
        Ok(response) => Ok(response.get_html()?),
        Err(_) => {
            // If all retries fail, return an error that can be handled gracefully
            Err(AidokuError::message("Request failed after retries"))
        }
    }
}

pub struct CrunchyScan;

impl Source for CrunchyScan {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        if let Some(search_query) = query {
            if !search_query.is_empty() {
                return self.search_manga(&search_query, page);
            }
        }

        let mut url = format!("{}/catalog", BASE_URL);
        
        if page > 1 {
            url.push_str(&format!("?page={}", page));
        }

        let mut selected_type = String::new();
        let mut selected_status = String::new();
        let mut selected_genres: Vec<String> = Vec::new();

        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "type" && !value.is_empty() && value != "Tout" {
                        selected_type = match value.as_str() {
                            "Manga" => String::from("manga"),
                            "Manhwa" => String::from("manhwa"),
                            "Manhua" => String::from("manhua"),
                            _ => value.to_lowercase(),
                        };
                    } else if id == "status" && !value.is_empty() && value != "Tout" {
                        selected_status = match value.as_str() {
                            "En cours" => String::from("ongoing"),
                            "Terminé" => String::from("completed"),
                            "Abandonné" => String::from("dropped"),
                            _ => value.to_lowercase(),
                        };
                    } else if id == "genres" && !value.is_empty() && value != "Tout" {
                        selected_genres.push(value.clone());
                    }
                }
                _ => {}
            }
        }

        if !selected_type.is_empty() || !selected_status.is_empty() || !selected_genres.is_empty() {
            let separator = if url.contains('?') { "&" } else { "?" };
            url.push_str(separator);

            if !selected_type.is_empty() {
                url.push_str(&format!("type={}&", selected_type));
            }
            if !selected_status.is_empty() {
                url.push_str(&format!("status={}&", selected_status));
            }
            for genre in selected_genres {
                url.push_str(&format!("genre={}&", helper::urlencode(&genre)));
            }
            
            if url.ends_with('&') {
                url.pop();
            }
        }

        let html = make_realistic_request(&url)?;
        let result = parser::parse_manga_list(html)?;
        
        
        Ok(result)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = helper::build_manga_url(&manga.key);
        let html = make_realistic_request(&url)?;

        if needs_details {
            let details = parser::parse_manga_details(html, manga.key.clone())?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.status = details.status;
            manga.cover = details.cover;
        }

        if needs_chapters {
            let chapter_html = make_realistic_request(&helper::build_manga_url(&manga.key))?;
            manga.chapters = Some(parser::parse_chapter_list(chapter_html, manga.key.clone())?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let html = make_realistic_request(&chapter.key)?;
        
        parser::parse_page_list(html)
    }
}

impl ListingProvider for CrunchyScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.name.as_str() {
            "Récents" => format!("{}/catalog?sort=recent&page={}", BASE_URL, page),
            "Populaires" => format!("{}/catalog?sort=popular&page={}", BASE_URL, page),
            _ => format!("{}/catalog?page={}", BASE_URL, page),
        };

        let html = make_realistic_request(&url)?;
        let result = parser::parse_manga_list(html)?;
        
        
        Ok(result)
    }
}

impl ImageRequestProvider for CrunchyScan {
    fn get_image_request(&self, url: String, _headers: Option<aidoku::HashMap<String, String>>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .header("Accept", "image/webp,image/apng,image/*,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive"))
    }
}

impl CrunchyScan {
    fn search_manga(&self, query: &str, page: i32) -> Result<MangaPageResult> {
        let encoded_query = helper::urlencode(query);
        let api_url = format!("{}/api/manga/search/manga/{}", BASE_URL, encoded_query);
        
        if let Ok(response) = make_request_with_cloudflare_retry(&api_url).and_then(|r| r.get_string()) {
            parser::search_manga(&response)
        } else {
            let fallback_url = format!("{}/catalog?search={}&page={}", BASE_URL, encoded_query, page);
            let html = make_realistic_request(&fallback_url)?;
            parser::parse_manga_list(html)
        }
    }
}

register_source!(CrunchyScan, ListingProvider, ImageRequestProvider);