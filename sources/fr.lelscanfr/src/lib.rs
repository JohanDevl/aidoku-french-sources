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
            // Version temporaire avec chapitres hardcodés pour debug
            let mut test_chapters: Vec<Chapter> = Vec::new();
            
            // Ajouter 5 chapitres de test
            for i in 1..=5 {
                let chapter_key = format!("/manga/{}/{}", manga.key, 1150 + i);
                let chapter_title = format!("Chapitre {}", 1150 + i);
                
                test_chapters.push(Chapter {
                    key: chapter_key.clone(),
                    title: Some(chapter_title),
                    chapter_number: Some((1150 + i) as f32),
                    volume_number: None,
                    date_uploaded: None,
                    scanlators: None,
                    language: Some(String::from("fr")),
                    locked: false,
                    thumbnail: None,
                    url: Some(format!("{}{}", BASE_URL, chapter_key)),
                });
            }
            
            manga.chapters = Some(test_chapters);
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