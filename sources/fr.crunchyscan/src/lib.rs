#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, Result, Source,
    alloc::{String, Vec, format, vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
extern crate serde;

use serde::Deserialize;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://crunchyscan.fr";
pub static API_URL: &str = "https://crunchyscan.fr/api";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

// API Response Structures
#[derive(Deserialize)]
struct ApiSearchResponse {
    data: Vec<ApiManga>,
    current_page: Option<i32>,
    last_page: Option<i32>,
}

#[derive(Deserialize)]
struct ApiManga {
    slug: String,
    title: String,
    #[serde(rename = "cover_path")]
    cover: Option<String>,
    #[serde(rename = "type")]
    manga_type: Option<String>,
}

// HTTP request function for HTML
fn make_request(url: &str) -> Result<Document> {
    Ok(Request::get(url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
        .header("Referer", BASE_URL)
        .html()?)
}

// HTTP request function for JSON API
fn make_api_request(sort: &str, page: i32, query: Option<&str>, filters: &[FilterValue]) -> Result<String> {
    let url = format!("{}/manga/search/advance", API_URL);

    // Build JSON request body
    let mut body_parts = vec![
        format!("\"page\":{}", page),
        format!("\"sort\":\"{}\"", sort),
    ];

    // Add search query if present
    if let Some(q) = query {
        if !q.is_empty() {
            body_parts.push(format!("\"title\":\"{}\"", q));
        }
    }

    // Add filters
    for filter in filters {
        match filter {
            FilterValue::Select { id, value } => {
                if id == "type" && !value.is_empty() && value != "Tout" {
                    let type_val = match value.as_str() {
                        "Manga" => "manga",
                        "Manhwa" => "manhwa",
                        "Manhua" => "manhua",
                        _ => &value.to_lowercase(),
                    };
                    body_parts.push(format!("\"type\":\"{}\"", type_val));
                } else if id == "status" && !value.is_empty() && value != "Tout" {
                    let status_val = match value.as_str() {
                        "En cours" => "ongoing",
                        "Terminé" => "completed",
                        "Abandonné" => "dropped",
                        _ => &value.to_lowercase(),
                    };
                    body_parts.push(format!("\"status\":\"{}\"", status_val));
                } else if id == "genres" && !value.is_empty() && value != "Tout" {
                    body_parts.push(format!("\"genre\":\"{}\"", value));
                }
            }
            _ => {}
        }
    }

    let body = format!("{{{}}}", body_parts.join(","));

    Ok(Request::post(&url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/json, text/plain, */*")
        .header("Content-Type", "application/json")
        .header("Referer", BASE_URL)
        .header("Origin", BASE_URL)
        .body(body.as_bytes())
        .string()?)
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
        let query_ref = query.as_ref().map(|s| s.as_str());
        let response = make_api_request("recent", page, query_ref, &filters)?;
        parser::parse_api_manga_list(&response, page)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = helper::build_manga_url(&manga.key);
        let html = make_request(&url)?;

        if needs_details {
            let details = parser::parse_manga_details(&html, &manga.key)?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.status = details.status;
            manga.cover = details.cover;
        }

        if needs_chapters {
            manga.chapters = Some(parser::parse_chapter_list(&html, &manga.key)?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let html = make_request(&chapter.key)?;
        parser::parse_page_list(&html)
    }
}

impl ListingProvider for CrunchyScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let sort = match listing.name.as_str() {
            "Récents" => "recent",
            "Populaires" => "popular",
            _ => "recent",
        };

        let response = make_api_request(sort, page, None, &[])?;
        parser::parse_api_manga_list(&response, page)
    }
}

impl ImageRequestProvider for CrunchyScan {
    fn get_image_request(&self, url: String, _headers: Option<aidoku::HashMap<String, String>>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .header("Accept", "image/webp,image/apng,image/*,*/*;q=0.8"))
    }
}

register_source!(CrunchyScan, ListingProvider, ImageRequestProvider);
