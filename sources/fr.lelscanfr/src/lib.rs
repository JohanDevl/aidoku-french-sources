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
            // Back to aggressive pagination detection that worked in v22
            let mut total_pages = 1;
            
            // Method 1: Look for "Page X of Y" pattern by scanning all elements
            let selectors = ["div", "span", "p", "text", "*"];
            for selector in selectors {
                if let Some(elements) = html.select(selector) {
                    for elem in elements {
                        if let Some(text) = elem.text() {
                            if text.contains("Page ") && text.contains(" of ") {
                                // Look for pattern like "Page 1 of 6"
                                let lines: Vec<&str> = text.split('\n').collect();
                                for line in lines {
                                    let trimmed = line.trim();
                                    if trimmed.contains("Page ") && trimmed.contains(" of ") {
                                        if let Some(of_pos) = trimmed.find(" of ") {
                                            let after_of = &trimmed[of_pos + 4..]; // Skip " of "
                                            if let Some(total_str) = after_of.split_whitespace().next() {
                                                if let Ok(pages) = total_str.parse::<i32>() {
                                                    total_pages = pages;
                                                    break;
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                    if total_pages > 1 {
                        break;
                    }
                }
            }
            
            // Method 2: If no pattern found, aggressively search for pagination
            if total_pages == 1 {
                let pagination_selectors = ["div", "span", "p", ".pagination", ".page-numbers", ".pages", "nav"];
                for selector in pagination_selectors {
                    if let Some(elements) = html.select(selector) {
                        for elem in elements {
                            if let Some(text) = elem.text() {
                                // Look for numbered page links 
                                let numbers: Vec<i32> = text
                                    .split_whitespace()
                                    .filter_map(|s| s.parse().ok())
                                    .filter(|&n| n > 1 && n < 50) // Reasonable page range
                                    .collect();
                                
                                if !numbers.is_empty() {
                                    let max_num = *numbers.iter().max().unwrap_or(&1);
                                    if max_num > total_pages {
                                        total_pages = max_num;
                                    }
                                }
                                
                                // Also check for ellipsis
                                if text.contains("…") || text.contains("...") {
                                    if total_pages < 10 {
                                        total_pages = 10;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Fetch all chapter pages (keep the optimized memory approach)
            let mut all_chapters: Vec<Chapter> = Vec::new();
            
            // Process first page
            let page_chapters = parser::parse_chapter_list(&manga.key, vec![html])?;
            all_chapters.extend(page_chapters);
            
            // Fetch additional pages
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
        // Use Madara template approach: add ?style=list parameter like mangascantrad
        let url = format!("{}/{}?style=list", BASE_URL, chapter.key);
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