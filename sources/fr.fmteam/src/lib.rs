#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://fmteam.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

pub struct FMTeam;

impl Source for FMTeam {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = if let Some(search_query) = query {
            format!("{}/api/comics/search?q={}", BASE_URL, helper::urlencode(search_query))
        } else {
            format!("{}/api/comics", BASE_URL)
        };
        
        // Process filters if needed
        let _ = filters;
        let _ = page; // API doesn't seem to support pagination yet
        
        let response = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .string()?;
        
        parser::parse_manga_list_json(response)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{}/api/comics/{}", BASE_URL, manga.key);
        let response = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .string()?;
        
        if needs_details {
            manga = parser::parse_manga_details_json(manga, response.clone())?;
        }
        
        if needs_chapters {
            let chapters = parser::parse_chapter_list_json(&manga.key, response)?;
            manga.chapters = Some(chapters);
        }
        
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        // Try API first, fallback to HTML if needed
        let api_url = format!("{}/api/chapters/{}/pages", BASE_URL, chapter.key);
        
        match Request::get(&api_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .string() {
            Ok(response) => parser::parse_page_list_json(response),
            Err(_) => {
                // Fallback to HTML parsing
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
    }
}

impl ListingProvider for FMTeam {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        // All listings use the main API endpoint for now
        let url = format!("{}/api/comics", BASE_URL);
        let _ = page; // API doesn't support pagination yet
        
        let response = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .string()?;
        
        parser::parse_manga_listing_json(response, &listing.id)
    }
}

impl ImageRequestProvider for FMTeam {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Sec-Fetch-Dest", "image")
            .header("Sec-Fetch-Mode", "no-cors")
            .header("Sec-Fetch-Site", "same-origin")
            .header("Referer", BASE_URL))
    }
}

register_source!(FMTeam, ListingProvider, ImageRequestProvider);