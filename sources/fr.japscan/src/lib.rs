#![no_std]

use aidoku::{
    Chapter, FilterValue, Listing, ListingProvider,
    Manga, MangaPageResult, Page, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://www.japscan.si";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36";

pub struct JapScan;

impl Source for JapScan {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        if let Some(search_query) = query {
            if !search_query.is_empty() {
                return self.search_by_query(search_query);
            }
        }

        let url = format!("{}/mangas/{}", BASE_URL, page);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", &format!("{}/", BASE_URL))
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .html()?;

        let base_host = "japscan.si";
        parser::parse_browse_list(html, base_host)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        if needs_details {
            let url = if manga.key.starts_with("http") {
                manga.key.clone()
            } else {
                format!("{}{}", BASE_URL, manga.key)
            };

            let html = Request::get(&url)?
                .header("User-Agent", USER_AGENT)
                .header("Referer", &format!("{}/", BASE_URL))
                .html()?;

            if let Ok(details) = parser::parse_manga_details(html) {
                manga.title = details.title;
                manga.cover = details.cover;
                manga.authors = details.authors;
                manga.artists = details.artists;
                manga.description = details.description;
                manga.tags = details.tags;
                manga.status = details.status;
                manga.content_rating = details.content_rating;
                manga.viewer = details.viewer;
            }
        }

        if needs_chapters {
            let url = if manga.key.starts_with("http") {
                manga.key.clone()
            } else {
                format!("{}{}", BASE_URL, manga.key)
            };

            let html = Request::get(&url)?
                .header("User-Agent", USER_AGENT)
                .header("Referer", &format!("{}/", BASE_URL))
                .html()?;

            let hide_spoilers = self.get_spoiler_preference();
            if let Ok(chapters) = parser::parse_chapter_list(html, hide_spoilers) {
                manga.chapters = Some(chapters);
            }
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let chapter_url = chapter.url.unwrap_or_else(|| chapter.key.clone());

        let url = if chapter_url.starts_with("http") {
            chapter_url
        } else {
            format!("{}{}", BASE_URL, chapter_url)
        };

        let html_content = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", &format!("{}/", BASE_URL))
            .send()?
            .get_string()?;

        parser::parse_page_list(html_content, BASE_URL)
    }
}

impl JapScan {
    fn search_by_query(&self, query: String) -> Result<MangaPageResult> {
        let encoded_query = helper::urlencode(query);
        let body = format!("search={}", encoded_query);

        let response = Request::post(&format!("{}/ls/", BASE_URL))?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("Referer", &format!("{}/", BASE_URL))
            .header("Origin", BASE_URL)
            .body(body.as_bytes())
            .send()?
            .get_string()?;

        parser::parse_search_json(response)
    }

    fn get_spoiler_preference(&self) -> bool {
        true
    }
}

impl ListingProvider for JapScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.name.as_str() {
            "Populaire" => format!("{}/mangas/?sort=popular&p={}", BASE_URL, page),
            "Dernières Mises à Jour" => format!("{}/mangas/?sort=updated&p={}", BASE_URL, page),
            _ => format!("{}/mangas/?sort=popular&p={}", BASE_URL, page),
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", &format!("{}/", BASE_URL))
            .html()?;

        parser::parse_manga_list(html)
    }
}

register_source!(JapScan, ListingProvider);
