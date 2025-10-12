#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
    Page, PageContext, Result, Source,
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
        // Parse filter values
        let mut status_filter: Option<String> = None;
        let mut type_filter: Option<String> = None;
        let mut genre_filter: Option<String> = None;
        let mut sort_filter: Option<String> = None;
        
        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    match id.as_str() {
                        "status" => {
                            if !value.is_empty() && value != "Tous les statuts" {
                                status_filter = Some(value);
                            }
                        }
                        "type" => {
                            if !value.is_empty() && value != "Tous les types" {
                                type_filter = Some(value);
                            }
                        }
                        "genre" => {
                            if !value.is_empty() && value != "Tous les genres" {
                                genre_filter = Some(value);
                            }
                        }
                        "sort" => {
                            if !value.is_empty() {
                                sort_filter = Some(value);
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
        
        // Fetch all manga and apply client-side filtering
        let url = format!("{}/manga/all", API_URL);
        let response = helper::build_api_request(&url)?.string()?;
        let search_query = query.unwrap_or_else(|| String::new());
        // Parse with new serde-based parser including all filters
        parser::parse_manga_list(response, search_query, status_filter, type_filter, genre_filter, sort_filter, page)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let encoded_key = helper::urlencode(manga.key.clone());
        let url = format!("{}/serie/{}", BASE_URL, encoded_key);
        let html = helper::build_html_request(&url)?.html()?;
            
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
        let encoded_manga_key = helper::urlencode(manga.key);
        let encoded_chapter_key = helper::urlencode(chapter.key);

        let url = format!("{}/serie/{}/chapter/{}", BASE_URL, encoded_manga_key, encoded_chapter_key);
        let html = helper::build_html_request(&url)?.html()?;
        parser::parse_page_list(&html, url)
    }

}

impl ListingProvider for PoseidonScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.name.as_str() {
            "Dernières Sorties" => {
                let url = format!("{}/manga/lastchapters?page={}&limit=20", API_URL, page);
                let response = helper::build_api_request(&url)?.string()?;
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
                let response = helper::build_api_request(&url)?.string()?;
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

impl ImageRequestProvider for PoseidonScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        
        // Special handling for API image URLs that require proper headers
        if url.contains("/api/chapters/") {
            
            // Try simpler headers first, similar to what the browser actually sends
            Ok(Request::get(&url)?
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8")
                .header("Referer", BASE_URL)
            )
        } else {
            // Fallback for other image URLs
            Ok(Request::get(url)?
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
                .header("Referer", BASE_URL)
            )
        }
    }
}

register_source!(PoseidonScans, ListingProvider, ImageRequestProvider);