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
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

// Enhanced HTTP request function with anti-Cloudflare headers and detailed error logging
fn make_request(url: &str) -> Result<Document> {
    // Log the request URL for debugging
    println!("[DEBUG] Requesting URL: {}", url);
    
    let request = Request::get(url)
        .map_err(|e| {
            println!("[ERROR] Failed to create request for {}: {:?}", url, e);
            e
        })?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
        .header("Accept-Encoding", "gzip, deflate, br, zstd")
        .header("Cache-Control", "max-age=0")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Sec-Fetch-Dest", "document")
        .header("Sec-Fetch-Mode", "navigate")
        .header("Sec-Fetch-Site", "none")
        .header("Sec-Fetch-User", "?1")
        .header("Sec-CH-UA", "\"Chromium\";v=\"122\", \"Not(A:Brand\";v=\"24\", \"Google Chrome\";v=\"122\"")
        .header("Sec-CH-UA-Mobile", "?0")
        .header("Sec-CH-UA-Platform", "\"Windows\"")
        .header("Referer", BASE_URL);
    
    println!("[DEBUG] Headers set, executing request...");
    
    let result = request.html().map_err(|e| {
        println!("[ERROR] HTTP request failed for {}: {:?}", url, e);
        println!("[ERROR] Error type: {}", core::any::type_name_of_val(&e));
        e
    });
    
    match &result {
        Ok(_) => println!("[DEBUG] Request successful for {}", url),
        Err(e) => println!("[ERROR] Final error for {}: {:?}", url, e),
    }
    
    Ok(result?)
}

// Simple test function with minimal headers to debug connectivity
fn make_simple_request(url: &str) -> Result<Document> {
    println!("[DEBUG] Making simple request to: {}", url);
    
    let request = Request::get(url)
        .map_err(|e| {
            println!("[ERROR] Failed to create simple request for {}: {:?}", url, e);
            e
        })?
        .header("User-Agent", USER_AGENT);
    
    let result = request.html().map_err(|e| {
        println!("[ERROR] Simple HTTP request failed for {}: {:?}", url, e);
        e
    });
    
    match &result {
        Ok(_) => println!("[DEBUG] Simple request successful for {}", url),
        Err(e) => println!("[ERROR] Simple request final error for {}: {:?}", url, e),
    }
    
    Ok(result?)
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

        // Test with simple request first, then try full request
        println!("[DEBUG] Testing connectivity with simple request...");
        
        // Try simple request first
        match make_simple_request(&url) {
            Ok(simple_html) => {
                println!("[SUCCESS] Simple request worked, trying full request...");
                match make_request(&url) {
                    Ok(html) => {
                        println!("[SUCCESS] Full request also worked");
                        parser::parse_manga_list(html)
                    },
                    Err(e) => {
                        println!("[WARNING] Full request failed, using simple request result: {:?}", e);
                        parser::parse_manga_list(simple_html)
                    }
                }
            },
            Err(e) => {
                println!("[ERROR] Even simple request failed: {:?}", e);
                // Try once more with the full request as fallback
                let html = make_request(&url)?;
                parser::parse_manga_list(html)
            }
        }
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = helper::build_manga_url(&manga.key);
        println!("[DEBUG] Getting manga update for: {} (URL: {})", manga.key, url);
        
        let html = match make_request(&url) {
            Ok(html) => html,
            Err(e) => {
                println!("[ERROR] Failed to get manga update for {}: {:?}", manga.key, e);
                return Err(e);
            }
        };

        if needs_details {
            let details = parser::parse_manga_details(html, manga.key.clone())?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.status = details.status;
            manga.cover = details.cover;
        }

        if needs_chapters {
            println!("[DEBUG] Getting chapters for manga: {}", manga.key);
            let chapter_html = match make_request(&helper::build_manga_url(&manga.key)) {
                Ok(html) => html,
                Err(e) => {
                    println!("[ERROR] Failed to get chapters for {}: {:?}", manga.key, e);
                    return Err(e);
                }
            };
            manga.chapters = Some(parser::parse_chapter_list(chapter_html, manga.key.clone())?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        println!("[DEBUG] Getting page list for chapter: {}", chapter.key);
        
        let html = match make_request(&chapter.key) {
            Ok(html) => html,
            Err(e) => {
                println!("[ERROR] Failed to get page list for chapter {}: {:?}", chapter.key, e);
                return Err(e);
            }
        };
        
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

        // Convert listing to filter for API request (not used anymore but kept for future)
        let _filters: Vec<FilterValue> = match listing.name.as_str() {
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
        
        // Test with simple request first, then try full request
        println!("[DEBUG] Testing listing connectivity with simple request...");
        
        // Try simple request first
        match make_simple_request(&url) {
            Ok(simple_html) => {
                println!("[SUCCESS] Simple listing request worked, trying full request...");
                match make_request(&url) {
                    Ok(html) => {
                        println!("[SUCCESS] Full listing request also worked");
                        parser::parse_manga_list(html)
                    },
                    Err(e) => {
                        println!("[WARNING] Full listing request failed, using simple request result: {:?}", e);
                        parser::parse_manga_list(simple_html)
                    }
                }
            },
            Err(e) => {
                println!("[ERROR] Even simple listing request failed: {:?}", e);
                // Try once more with the full request as fallback
                let html = make_request(&url)?;
                parser::parse_manga_list(html)
            }
        }
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
        // Use HTML search instead of API (API has CSRF issues)
        let search_url = format!("{}/catalog?title={}&page={}", BASE_URL, helper::urlencode(query), page);
        println!("[DEBUG] Searching manga with query: '{}' on page {} (URL: {})", query, page, search_url);
        
        let html = match make_request(&search_url) {
            Ok(html) => html,
            Err(e) => {
                println!("[ERROR] Failed to search manga with query '{}': {:?}", query, e);
                return Err(e);
            }
        };
        parser::parse_manga_list(html)
    }
}

register_source!(CrunchyScan, ListingProvider, ImageRequestProvider);