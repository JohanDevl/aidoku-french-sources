#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, Page, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document},
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
            // Detect pagination using "Page X of Y" pattern first
            let mut total_pages = 1;
            
            // Look for "Page X of Y" pattern in the HTML text
            if let Some(all_elements) = html.select("div, span, p, text()") {
                for elem in all_elements {
                    if let Some(text) = elem.text() {
                        if text.contains("Page ") && text.contains(" of ") {
                            // Extract "Page 1 of 6" pattern
                            if let Some(of_pos) = text.find(" of ") {
                                let after_of = &text[of_pos + 4..];
                                if let Some(total_pages_str) = after_of.split_whitespace().next() {
                                    if let Ok(pages) = total_pages_str.parse::<i32>() {
                                        total_pages = pages;
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // If no "Page X of Y" found, try pagination links
            if total_pages == 1 {
                let pagination_selectors = [".pagination", ".page-numbers", ".pages"];
                for selector in pagination_selectors {
                    if let Some(pagination) = html.select(selector) {
                        if let Some(first_pagination) = pagination.first() {
                            // Look for ellipsis indicating more pages
                            if let Some(pagination_text) = first_pagination.text() {
                                if pagination_text.contains("…") || pagination_text.contains("...") {
                                    total_pages = 10; // Conservative estimate if ellipsis found
                                    break;
                                }
                            }
                            // Extract max page number from links
                            if let Some(links) = first_pagination.select("a") {
                                for link in links {
                                    if let Some(link_text) = link.text() {
                                        if let Ok(page_num) = link_text.trim().parse::<i32>() {
                                            if page_num > total_pages {
                                                total_pages = page_num;
                                            }
                                        }
                                    }
                                }
                            }
                            break;
                        }
                    }
                }
            }
            
            // Fetch all chapter pages
            let mut all_docs: Vec<Document> = vec![html];
            for page in 2..=total_pages {
                let page_url = format!("{}/manga/{}?page={}", BASE_URL, manga.key, page);
                let page_html = Request::get(&page_url)?
                    .header("User-Agent", USER_AGENT)
                    .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
                    .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                    .html()?;
                all_docs.push(page_html);
            }
            
            manga.chapters = Some(parser::parse_chapter_list(&manga.key, all_docs)?);
        }
        
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/{}", BASE_URL, chapter.key);
        let html = Request::get(&url)?.html()?;
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