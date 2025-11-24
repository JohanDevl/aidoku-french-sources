#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
    Page, PageContext, Result, Source,
    alloc::{String, Vec},
    imports::{net::Request, std::send_partial_result},
    println,
    prelude::*,
};

mod parser;
mod helper;

pub static BASE_URL: &str = "https://poseidon-scans.com";
pub static API_URL: &str = "https://poseidon-scans.com/api";

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
        // Build URL with query parameters for /series page
        let mut url = format!("{}/series", BASE_URL);
        let mut params = Vec::new();

        // Parse filters by ID (not index, as Aidoku only sends modified filters)
        let mut status_filter: Option<String> = None;
        let mut genre_filter: Option<String> = None;
        let mut sort_filter: Option<String> = None;

        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "status" && !value.is_empty() {
                        status_filter = Some(value.clone());
                    } else if id == "genre" && !value.is_empty() && value != "Tous les genres" {
                        genre_filter = Some(value.clone());
                    } else if id == "sort" && !value.is_empty() {
                        sort_filter = Some(value.clone());
                    }
                }
                _ => {}
            }
        }

        // Build tags parameter (genres only)
        // Values come from filters.json options
        if let Some(genre) = genre_filter {
            params.push(format!("tags={}", helper::urlencode(genre)));
        }

        // Add status parameter (value from filters.json ids already in lowercase)
        if let Some(status) = status_filter {
            params.push(format!("status={}", helper::urlencode(status)));
        }

        // Add sortBy parameter (value from filters.json ids)
        if let Some(sort) = sort_filter {
            params.push(format!("sortBy={}", sort));
        }

        // Add search query if provided
        if let Some(ref q) = query {
            if !q.is_empty() {
                params.push(format!("search={}", helper::urlencode(q.clone())));
            }
        }

        // Add page parameter if not first page
        if page > 1 {
            params.push(format!("page={}", page));
        }

        // Append parameters to URL
        if !params.is_empty() {
            url = format!("{}?{}", url, params.join("&"));
        }

        // Fetch and parse HTML
        let html = helper::build_html_request(&url)?.html()?;
        parser::parse_series_page(&html)
    }

    fn get_manga_update(&self, manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        println!("[PoseidonScans] get_manga_update called for: {}", manga.key);
        println!("[PoseidonScans] needs_details: {}, needs_chapters: {}", needs_details, needs_chapters);

        let encoded_key = helper::urlencode(manga.key.clone());
        let url = format!("{}/serie/{}", BASE_URL, encoded_key);
        println!("[PoseidonScans] Fetching URL: {}", url);

        let html = helper::build_html_request(&url)?.html()?;
        println!("[PoseidonScans] HTML fetched successfully");

        let mut updated_manga = parser::parse_manga_details(manga.key.clone(), &html)?;

        println!("[PoseidonScans] Parsed manga details:");
        println!("  - Title: {}", updated_manga.title);
        println!("  - Authors: {:?}", updated_manga.authors);
        println!("  - Artists: {:?}", updated_manga.artists);
        println!("  - Description: {:?}", updated_manga.description.as_ref().map(|d| &d[..50.min(d.len())]));
        println!("  - Status: {:?}", updated_manga.status);
        println!("  - Tags count: {:?}", updated_manga.tags.as_ref().map(|t| t.len()));
        println!("  - UpdateStrategy: {:?}", updated_manga.update_strategy);

        if needs_details {
            println!("[PoseidonScans] Sending partial result for details");
            send_partial_result(&updated_manga);
        }

        if needs_chapters {
            println!("[PoseidonScans] Parsing chapters");
            let chapters = parser::parse_chapter_list(manga.key, &html)?;
            println!("[PoseidonScans] Found {} chapters", chapters.len());
            updated_manga.chapters = Some(chapters);
        }

        println!("[PoseidonScans] get_manga_update completed successfully");
        Ok(updated_manga)
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