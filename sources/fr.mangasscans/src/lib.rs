#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga,
    MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod helper;
mod parser;

use helper::urlencode;
use parser::{has_next_page, parse_chapter_list, parse_manga_details, parse_manga_list, parse_page_list};

pub static BASE_URL: &str = "https://mangas-scans.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

pub struct MangasScans;

impl Source for MangasScans {
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

        let url = if !search_query.is_empty() {
            let encoded = urlencode(search_query);
            format!("{}/manga/?title={}&page={}", BASE_URL, encoded, page)
        } else {
            format!("{}/manga/?page={}", BASE_URL, page)
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        let entries = parse_manga_list(&html, BASE_URL);
        let has_next = has_next_page(&html);

        Ok(MangaPageResult {
            entries,
            has_next_page: has_next,
        })
    }

    fn get_manga_update(
        &self,
        manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let mut updated_manga = manga.clone();

        if needs_details || needs_chapters {
            let manga_url = if let Some(url) = &manga.url {
                url.clone()
            } else {
                format!("{}/manga/{}/", BASE_URL, manga.key)
            };

            let html = Request::get(&manga_url)?
                .header("User-Agent", USER_AGENT)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Referer", BASE_URL)
                .html()?;

            if needs_details {
                updated_manga = parse_manga_details(&html, BASE_URL, manga.key.clone())?;
            }

            if needs_chapters {
                updated_manga.chapters = Some(parse_chapter_list(&html));
            }
        }

        Ok(updated_manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let chapter_url = if let Some(url) = &chapter.url {
            url.clone()
        } else {
            format!("{}/{}/", BASE_URL, chapter.key)
        };

        let html = Request::get(&chapter_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        Ok(parse_page_list(&html, BASE_URL))
    }
}

impl ListingProvider for MangasScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.id.as_str() {
            "populaire" => format!("{}/manga/?page={}&order=popular", BASE_URL, page),
            "dernieres" => format!("{}/manga/?page={}&order=update", BASE_URL, page),
            _ => format!("{}/manga/?page={}", BASE_URL, page),
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        let entries = parse_manga_list(&html, BASE_URL);
        let has_next = has_next_page(&html);

        Ok(MangaPageResult {
            entries,
            has_next_page: has_next,
        })
    }
}

impl ImageRequestProvider for MangasScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

register_source!(MangasScans, ListingProvider, ImageRequestProvider);
