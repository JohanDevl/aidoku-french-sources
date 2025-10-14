#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source, AidokuError,
    alloc::{String, Vec, format},
    imports::{net::Request, std::send_partial_result},
    prelude::*,
};

extern crate alloc;
use alloc::vec;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://lelscanfr.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

const MAX_PAGINATION_PAGES: i32 = 150;
const MAX_RETRIES: u32 = 3;

pub struct LelscanFr;

impl Source for LelscanFr {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let page = page.max(1).min(MAX_PAGINATION_PAGES);

        let mut query_params = String::new();
        
        // Add search query if provided
        if let Some(search_query) = query {
            query_params.push_str(&format!("&title={}", helper::urlencode(search_query)));
        }
        
        // Process filters
        let mut selected_genres: Vec<String> = Vec::new();
        let mut selected_status = String::new();
        let mut selected_type = String::new();
        
        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "type" && !value.is_empty() && value != "Tout" {
                        // Map type filter values to server values
                        selected_type = match value.as_str() {
                            "Manga" => String::from("manga"),
                            "Manhua" => String::from("manhua"),
                            "Manhwa" => String::from("manhwa"),
                            "Bande Dessinée" => String::from("bd"),
                            _ => value.to_lowercase(),
                        };
                    } else if id == "status" && !value.is_empty() && value != "Tout" {
                        // Map French status values to server values
                        selected_status = match value.as_str() {
                            "En cours" => String::from("en-cours"),
                            "En pause" => String::from("en-pause"),
                            "Terminé" => String::from("termin"),
                            _ => value.clone(),
                        };
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" {
                        // Add included genres only (site doesn't support exclusion)
                        for value in included {
                            if !value.is_empty() && value != "Tout" {
                                selected_genres.push(value.clone());
                            }
                        }
                        // Note: excluded genres are ignored as the site doesn't support exclusion
                    }
                }
                _ => {}
            }
        }
        
        // Add filter parameters to query
        if !selected_type.is_empty() {
            query_params.push_str(&format!("&type={}", helper::urlencode(selected_type)));
        }
        
        if !selected_status.is_empty() {
            query_params.push_str(&format!("&status={}", helper::urlencode(selected_status)));
        }
        
        for genre in &selected_genres {
            if !genre.is_empty() {
                let encoded_genre = helper::urlencode(genre.clone());
                query_params.push_str(&format!("&genre%5B%5D={}", encoded_genre));
            }
        }
        
        let url = format!("{}/manga?page={}{}", BASE_URL, page, query_params);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .html()?;
        
        parser::parse_manga_list(html)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        println!("[lelscanfr] get_manga_update START - manga_id: {}, needs_details: {}, needs_chapters: {}",
            manga.key, needs_details, needs_chapters);

        let url = format!("{}/manga/{}", BASE_URL, manga.key);
        let html = Self::request_with_retry(&url)?;

        if needs_details {
            manga = parser::parse_manga_details(manga, &html)?;
            println!("[lelscanfr] Metadata fetched successfully - title: {}", manga.title);
            send_partial_result(&manga);
        }

        if needs_chapters {
            let total_pages = helper::detect_pagination(&html);
            
            // Fetch all chapter pages with optimized batching approach
            let mut all_chapters: Vec<Chapter> = Vec::new();
            
            // Process first page
            let page_chapters = parser::parse_chapter_list(&manga.key, vec![html])?;
            all_chapters.extend(page_chapters);
            
            for page in 2..=total_pages {
                let page_url = format!("{}/manga/{}?page={}", BASE_URL, manga.key, page);
                let page_html = Self::request_with_retry_headers(&page_url, vec![
                    ("User-Agent", USER_AGENT),
                    ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8"),
                    ("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8"),
                ])?;

                let page_chapters = parser::parse_chapter_list(&manga.key, vec![page_html])?;
                all_chapters.extend(page_chapters);
            }

            manga.chapters = Some(all_chapters.clone());
            println!("[lelscanfr] Chapters fetched successfully - count: {} (across {} pages)",
                all_chapters.len(), total_pages);
        }

        println!("[lelscanfr] get_manga_update COMPLETE");
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        // chapter.key already contains the full path like "/manga/some-manga/123"
        let url = if chapter.key.starts_with("/") {
            format!("{}{}", BASE_URL, chapter.key)
        } else {
            format!("{}/{}", BASE_URL, chapter.key)
        };
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;
        
        parser::parse_page_list(&html)
    }
}

impl ListingProvider for LelscanFr {
    fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
        let page = page.max(1).min(MAX_PAGINATION_PAGES);

        let url = format!("{}/manga?page={}", BASE_URL, page);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .html()?;
        
        parser::parse_manga_list(html)
    }
}

impl ImageRequestProvider for LelscanFr {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl LelscanFr {
    fn request_with_retry(url: &str) -> Result<aidoku::imports::html::Document> {
        let mut attempt = 0;
        loop {
            let request = Request::get(url)?;

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
                200..=299 => {
                    return response.get_html().map_err(|e| AidokuError::RequestError(e));
                }
                408 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Request timeout"));
                    }
                    attempt += 1;
                    continue;
                }
                502 | 503 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                }
                403 | 429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Access blocked or rate limited"));
                    }
                    attempt += 1;
                    continue;
                }
                _ => {
                    return Err(AidokuError::message("Request failed"));
                }
            }
        }
    }

    fn request_with_retry_headers(url: &str, headers: Vec<(&str, &str)>) -> Result<aidoku::imports::html::Document> {
        let mut attempt = 0;
        loop {
            let mut request = Request::get(url)?;
            for (key, value) in &headers {
                request = request.header(key, value);
            }

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
                200..=299 => {
                    return response.get_html().map_err(|e| AidokuError::RequestError(e));
                }
                408 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Request timeout"));
                    }
                    attempt += 1;
                    continue;
                }
                502 | 503 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                }
                403 | 429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Access blocked or rate limited"));
                    }
                    attempt += 1;
                    continue;
                }
                _ => {
                    return Err(AidokuError::message("Request failed"));
                }
            }
        }
    }
}

register_source!(LelscanFr, ListingProvider, ImageRequestProvider);