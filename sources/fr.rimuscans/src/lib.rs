#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod helper;
mod parser;

use helper::urlencode;
use parser::{parse_chapter_list, parse_manga_details, parse_manga_list, parse_page_list, has_next_page};

pub static BASE_URL: &str = "https://rimuscans.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub struct RimuScans;

impl Source for RimuScans {
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
            let encoded_query = urlencode(search_query);
            if page == 1 {
                format!("{}/manga/?s={}", BASE_URL, encoded_query)
            } else {
                format!("{}/manga/page/{}/?s={}", BASE_URL, page, encoded_query)
            }
        } else {
            if page == 1 {
                format!("{}/manga/", BASE_URL)
            } else {
                format!("{}/manga/page/{}/", BASE_URL, page)
            }
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

        let mangas = parse_manga_list(&html, BASE_URL);
        let has_more = has_next_page(&html);

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: has_more,
        })
    }

    fn get_manga_update(&self, manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let mut updated_manga = manga.clone();

        if needs_details || needs_chapters {
            let manga_url = if let Some(url) = &manga.url {
                url.clone()
            } else {
                manga.key.clone()
            };

            let html = Request::get(&manga_url)?
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
                updated_manga = parse_manga_details(&html, manga.key.clone(), BASE_URL)?;
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
            chapter.key.clone()
        };

        let html = Request::get(&chapter_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        Ok(parse_page_list(&html))
    }
}

impl ListingProvider for RimuScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.id.as_str() {
            "popular" => self.get_popular_manga(page),
            "latest" => self.get_latest_manga(page),
            _ => self.get_latest_manga(page),
        }
    }
}

impl ImageRequestProvider for RimuScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8"))
    }
}

impl RimuScans {
    fn get_popular_manga(&self, page: i32) -> Result<MangaPageResult> {
        let url = if page == 1 {
            format!("{}/manga/?order=popular", BASE_URL)
        } else {
            format!("{}/manga/page/{}/?order=popular", BASE_URL, page)
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

        let mangas = parse_manga_list(&html, BASE_URL);
        let has_more = has_next_page(&html);

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: has_more,
        })
    }

    fn get_latest_manga(&self, page: i32) -> Result<MangaPageResult> {
        let url = if page == 1 {
            format!("{}/manga/?order=update", BASE_URL)
        } else {
            format!("{}/manga/page/{}/?order=update", BASE_URL, page)
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

        let mangas = parse_manga_list(&html, BASE_URL);
        let has_more = has_next_page(&html);

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: has_more,
        })
    }
}

register_source!(RimuScans, ListingProvider, ImageRequestProvider);
