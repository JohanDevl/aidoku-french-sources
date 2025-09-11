#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, Result, Source,
    alloc::{String, Vec, format, vec, string::ToString},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://crunchyscan.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

// Simple HTTP request function - following SushiScans/MangasOrigines pattern
fn make_request(url: &str) -> Result<Document> {
    Ok(Request::get(url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Cache-Control", "max-age=0")
        .header("Referer", BASE_URL)
        .html()?)
}

// Simplified fallback - just return empty for now and focus on getting search to work
fn make_api_request(_page: i32, _filters: &[FilterValue]) -> Result<String> {
    // For now, return empty JSON to avoid compilation issues
    // TODO: Implement proper API request once we understand the exact format needed
    Ok("{\"data\":[]}".to_string())
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

        let html = make_request(&url)?;
        let result = parser::parse_manga_list(html)?;
        
        // If HTML parsing returned empty (dynamic content), try API
        if result.entries.is_empty() {
            let api_response = make_api_request(page, &filters)?;
            return parser::parse_api_response(&api_response);
        }
        
        Ok(result)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = helper::build_manga_url(&manga.key);
        let html = make_request(&url)?;

        if needs_details {
            let details = parser::parse_manga_details(html, manga.key.clone())?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.status = details.status;
            manga.cover = details.cover;
        }

        if needs_chapters {
            let chapter_html = make_request(&helper::build_manga_url(&manga.key))?;
            manga.chapters = Some(parser::parse_chapter_list(chapter_html, manga.key.clone())?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let html = make_request(&chapter.key)?;
        
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

        let html = make_request(&url)?;
        let result = parser::parse_manga_list(html)?;
        
        // If HTML parsing returned empty (dynamic content), try API
        if result.entries.is_empty() {
            // Convert listing to filter for API request
            let filters: Vec<FilterValue> = match listing.name.as_str() {
                "Récents" => vec![FilterValue::Select { 
                    id: "sort".to_string(), 
                    value: "recent".to_string() 
                }],
                "Populaires" => vec![FilterValue::Select { 
                    id: "sort".to_string(), 
                    value: "popular".to_string() 
                }],
                _ => vec![],
            };
            
            let api_response = make_api_request(page, &filters)?;
            return parser::parse_api_response(&api_response);
        }
        
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
    fn search_manga(&self, query: &str, _page: i32) -> Result<MangaPageResult> {
        // Use the search API directly for text searches
        let search_url = format!("{}/api/manga/search/manga/{}", BASE_URL, helper::urlencode(query));
        
        let search_response = Request::get(&search_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", &format!("{}/catalog", BASE_URL))
            .string()?;
            
        // Use the existing search_manga parser for the search API response
        parser::search_manga(&search_response)
    }
}

register_source!(CrunchyScan, ListingProvider, ImageRequestProvider);