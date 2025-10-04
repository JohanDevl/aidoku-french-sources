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
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

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
        // Use catalog page with HTML parsing instead of API
        let url = if let Some(search_query) = query {
            if !search_query.is_empty() {
                format!("{}/catalog?page={}&search={}", BASE_URL, page, helper::urlencode(&search_query))
            } else {
                format!("{}/catalog?page={}", BASE_URL, page)
            }
        } else {
            format!("{}/catalog?page={}", BASE_URL, page)
        };

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

        parser::parse_manga_list(&html, page)
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
