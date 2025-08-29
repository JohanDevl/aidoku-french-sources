#![no_std]

use aidoku::{
    Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult,
    Page, Result, Source,
    alloc::{String, Vec},
    imports::net::Request,
    prelude::*,
};

mod parser;
mod helper;

pub static BASE_URL: &str = "https://poseidonscans.com";
pub static API_URL: &str = "https://poseidonscans.com/api";

pub struct PoseidonScans;

impl Source for PoseidonScans {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self, 
        query: Option<String>, 
        page: i32, 
        filters: Vec<FilterValue>
    ) -> Result<MangaPageResult> {
        let _ = filters; // Ignore filters for now - Poseidon Scans uses client-side filtering
        
        // Fetch all manga and apply client-side filtering
        let url = format!("{}/manga/all", API_URL);
        let response = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", "application/json, text/plain, */*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Referer", BASE_URL)
            .header("Origin", BASE_URL)
            .string()?;
        let search_query = query.unwrap_or_else(|| String::new());
        // Parse with new serde-based parser
        parser::parse_manga_list(response, search_query, None, page)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let url = format!("{}/serie/{}", BASE_URL, manga.key);
        let html = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;
            
        if needs_chapters {
            // Parse both details and chapters from the same HTML
            let mut updated_manga = parser::parse_manga_details(manga.key.clone(), &html)?;
            let chapters = parser::parse_chapter_list(manga.key, &html)?;
            updated_manga.chapters = Some(chapters);
            Ok(updated_manga)
        } else {
            parser::parse_manga_details(manga.key, &html)
        }
    }


    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/serie/{}/chapter/{}", BASE_URL, manga.key, chapter.key);
        let html = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;
        parser::parse_page_list(&html, url)
    }

}

impl ListingProvider for PoseidonScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.name.as_str() {
            "Dernières Sorties" => {
                let url = format!("{}/manga/lastchapters?page={}&limit=20", API_URL, page);
                let response = Request::get(&url)?
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                    .header("Accept", "application/json, text/plain, */*")
                    .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                    .header("Accept-Encoding", "gzip, deflate, br")
                    .header("Referer", BASE_URL)
                    .header("Origin", BASE_URL)
                    .string()?;

                parser::parse_latest_manga(response)
            },
            "Populaire" => {
                // Popular listing only has one page
                if page > 1 {
                    return Ok(MangaPageResult {
                        entries: Vec::new(),
                        has_next_page: false,
                    });
                }

                let url = format!("{}/manga/popular", API_URL);
                let response = Request::get(&url)?
                    .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                    .header("Accept", "application/json, text/plain, */*")
                    .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                    .header("Accept-Encoding", "gzip, deflate, br")
                    .header("Referer", BASE_URL)
                    .header("Origin", BASE_URL)
                    .string()?;

                parser::parse_popular_manga(response)
            },
            _ => {
                Ok(MangaPageResult {
                    entries: Vec::new(),
                    has_next_page: false,
                })
            }
        }
    }
}

register_source!(PoseidonScans, ListingProvider);