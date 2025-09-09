#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;
use alloc::vec;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://lelscanfr.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

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
                // Encode the genre name properly for URL
                let encoded_genre = genre.replace(" ", "+")
                    .replace("é", "%C3%A9")
                    .replace("è", "%C3%A8")
                    .replace("à", "%C3%A0")
                    .replace("ç", "%C3%A7")
                    .replace("ô", "%C3%B4")
                    .replace("â", "%C3%A2")
                    .replace("ê", "%C3%AA")
                    .replace("î", "%C3%AE")
                    .replace("ù", "%C3%B9")
                    .replace("û", "%C3%BB")
                    .replace("ï", "%C3%AF")
                    .replace("ë", "%C3%AB")
                    .replace("ü", "%C3%BC")
                    .replace("ö", "%C3%B6")
                    .replace("É", "%C3%89")
                    .replace("È", "%C3%88")
                    .replace("À", "%C3%80")
                    .replace("Ç", "%C3%87")
                    .replace("Ô", "%C3%94")
                    .replace("Â", "%C3%82")
                    .replace("Ê", "%C3%8A")
                    .replace("Î", "%C3%8E")
                    .replace("Ù", "%C3%99")
                    .replace("Û", "%C3%9B")
                    .replace("Ï", "%C3%8F")
                    .replace("Ë", "%C3%8B")
                    .replace("Ü", "%C3%9C")
                    .replace("Ö", "%C3%96")
                    .replace("-", "%2D");
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
        let url = format!("{}/manga/{}", BASE_URL, manga.key);
        let html = Request::get(&url)?.html()?;
        
        if needs_details {
            manga = parser::parse_manga_details(manga, &html)?;
        }
        
        if needs_chapters {
            let mut total_pages = 1;
            
            // Optimized pagination detection - target specific elements first
            // Method 1: Look for pagination containers
            let pagination_containers = [".pagination", ".page-numbers", ".pages", "nav"];
            for selector in pagination_containers {
                if let Some(pagination_element) = html.select(selector) {
                    if let Some(text) = pagination_element.text() {
                        // Look for "Page X of Y" pattern with regex-like logic
                        if let Some(total) = helper::extract_pagination_total(&text) {
                            total_pages = total;
                            break;
                        }
                    }
                }
            }
            
            // Method 2: If no pagination container found, check common patterns
            if total_pages == 1 {
                // Look for "Page X of Y" in more limited scope
                let limited_selectors = [".pagination-info", ".page-info", ".pagination-text"];
                for selector in limited_selectors {
                    if let Some(element) = html.select(selector) {
                        if let Some(text) = element.text() {
                            if let Some(total) = helper::extract_pagination_total(&text) {
                                total_pages = total;
                                break;
                            }
                        }
                    }
                }
            }
            
            // Method 3: Fallback - scan body text for pagination patterns
            if total_pages == 1 {
                if let Some(body) = html.select("body") {
                    let body_text = body.text().unwrap_or_default();
                    if let Some(total) = helper::extract_pagination_total(&body_text) {
                        total_pages = total;
                    }
                }
            }
            
            // Method 3.5: RESTORE original aggressive pagination detection that found hidden pages!
            if total_pages <= 6 { // Even if we found 6, look for more!
                // Use the EXACT original logic that worked
                let pagination_selectors = ["div", "span", "p", ".pagination", ".page-numbers", ".pages", "nav"];
                for selector in pagination_selectors {
                    if let Some(elements) = html.select(selector) {
                        for elem in elements {
                            if let Some(text) = elem.text() {
                                // Look for numbered page links - ORIGINAL LOGIC
                                let numbers: Vec<i32> = text
                                    .split_whitespace()
                                    .filter_map(|s| s.parse().ok())
                                    .filter(|&n| n > 1 && n < 150) // Expanded range
                                    .collect();
                                
                                if !numbers.is_empty() {
                                    let max_num = *numbers.iter().max().unwrap_or(&1);
                                    if max_num > total_pages {
                                        total_pages = max_num; // THIS IS KEY - could find page 25, 50, etc!
                                    }
                                }
                                
                                // Also check for ellipsis - ORIGINAL LOGIC
                                if text.contains("…") || text.contains("...") {
                                    // If there's ellipsis, there are definitely more pages
                                    let estimated_pages = if !numbers.is_empty() {
                                        let max_visible = *numbers.iter().max().unwrap_or(&6);
                                        (max_visible * 3).min(100) // Estimate 3x more pages beyond visible
                                    } else {
                                        25 // Conservative estimate with ellipsis
                                    };
                                    if estimated_pages > total_pages {
                                        total_pages = estimated_pages;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Method 4: Heuristic fallback based on chapter count
            if total_pages == 1 {
                // Make a quick check for many chapters without parsing all
                if let Some(chapter_links) = html.select("a[href*=\"/manga/\"]") {
                    let mut chapter_count = 0;
                    for _link in chapter_links {
                        chapter_count += 1;
                        if chapter_count >= 20 {
                            total_pages = 5; // Conservative estimate for manga with many chapters
                            break;
                        }
                    }
                }
            }
            
            // Safety limit to prevent infinite loops - increased for manga with many chapters
            if total_pages > 150 {
                total_pages = 150;
            }
            
            // Fetch all chapter pages with optimized batching approach
            let mut all_chapters: Vec<Chapter> = Vec::new();
            
            // Process first page
            let page_chapters = parser::parse_chapter_list(&manga.key, vec![html])?;
            all_chapters.extend(page_chapters);
            
            // Fetch additional pages - simple sequential approach
            for page in 2..=total_pages {
                let page_url = format!("{}/manga/{}?page={}", BASE_URL, manga.key, page);
                let page_html = Request::get(&page_url)?
                    .header("User-Agent", USER_AGENT)
                    .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
                    .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                    .html()?;
                
                // Parse immediately to save memory
                let page_chapters = parser::parse_chapter_list(&manga.key, vec![page_html])?;
                all_chapters.extend(page_chapters);
            }
            
            manga.chapters = Some(all_chapters);
        }
        
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

register_source!(LelscanFr, ListingProvider, ImageRequestProvider);