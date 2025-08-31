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
        
        // Process filters - ignore for now
        let _ = filters;
        
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
            
            // Method 3.5: Ultra-aggressive fallback like the old implementation
            if total_pages == 1 {
                // Look for pagination elements more aggressively
                let aggressive_selectors = ["div", "span", "p", "nav"];
                for selector in aggressive_selectors {
                    if let Some(elements) = html.select(selector) {
                        for elem in elements {
                            if let Some(text) = elem.text() {
                                // Check for ellipsis patterns that indicate many pages
                                if text.contains("…") || text.contains("...") {
                                    // Look for numbers near ellipsis
                                    let numbers: Vec<i32> = text
                                        .split_whitespace()
                                        .filter_map(|s| s.parse().ok())
                                        .filter(|&n| n > 1 && n < 200)
                                        .collect();
                                    
                                    if !numbers.is_empty() {
                                        let max_num = *numbers.iter().max().unwrap_or(&1);
                                        if max_num > 1 {
                                            // When ellipsis is present, estimate there are more pages
                                            total_pages = (max_num * 5).min(100); // Aggressive estimate
                                            break;
                                        }
                                    } else {
                                        total_pages = 30; // Default when ellipsis found but no numbers
                                        break;
                                    }
                                }
                            }
                        }
                        if total_pages > 1 {
                            break;
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
            
            // Fetch additional pages using optimized batching
            if total_pages > 1 {
                all_chapters.extend(helper::fetch_pages_batch(&manga.key, 2, total_pages)?);
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