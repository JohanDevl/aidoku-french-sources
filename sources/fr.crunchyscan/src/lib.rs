#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;
extern crate serde_json;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://crunchyscan.fr";
pub static API_URL: &str = "https://crunchyscan.fr/api/manga/search/advance";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

pub struct CrunchyScan;

impl Source for CrunchyScan {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let search_query = query.unwrap_or_default();

        // Step 1: Get the catalog page to extract CSRF token
        let catalog_url = format!("{}/catalog", BASE_URL);
        let catalog_response = Request::get(&catalog_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .send()?;

        let html_string = catalog_response.get_string()?;

        // Extract CSRF token from meta tag: <meta name="csrf-token" content="...">
        let csrf_token = helper::extract_csrf_token(&html_string).unwrap_or_default();
        println!("[CrunchyScan] CSRF Token: {}", if csrf_token.is_empty() { "NOT FOUND" } else { &csrf_token });

        if csrf_token.is_empty() {
            println!("[CrunchyScan] Warning: No CSRF token found, API call may fail");
        }

        // Step 2: Make the API POST request with CSRF token
        let body = format!(
            "page={}&search={}&order=latest&type=&status=&categories=",
            page,
            helper::urlencode(&search_query)
        );

        let response = Request::post(API_URL)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-Csrf-Token", &csrf_token)
            .header("Origin", BASE_URL)
            .header("Referer", &catalog_url)
            .body(body.as_bytes())
            .send()?;

        let json_string = response.get_string()?;

        // Debug: log the response
        let preview = if json_string.len() > 500 {
            &json_string[..500]
        } else {
            &json_string
        };
        println!("[CrunchyScan] API Response (first 500 chars): {}", preview);
        println!("[CrunchyScan] Response length: {}", json_string.len());

        parser::parse_manga_list_json(&json_string, page)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{}/lecture-en-ligne/{}", BASE_URL, manga.key);
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        if needs_details {
            let details = parser::parse_manga_details(&html, &manga.key)?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.cover = details.cover;
            manga.status = details.status;
            manga.authors = details.authors;
        }

        if needs_chapters {
            manga.chapters = Some(parser::parse_chapter_list(&html)?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let html = Request::get(&chapter.key)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        parser::parse_page_list(&html)
    }
}

impl ImageRequestProvider for CrunchyScan {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

register_source!(CrunchyScan, ImageRequestProvider);
