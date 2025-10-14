#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga,
    MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::{net::Request, std::send_partial_result},
    prelude::*,
};

extern crate alloc;

mod helper;
mod parser;

use helper::{build_filter_params, detect_pagination, urlencode};
use parser::{has_next_page, parse_chapter_list, parse_manga_details, parse_manga_list, parse_page_list};

pub static BASE_URL: &str = "https://mangas-scans.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

fn build_request(url: &str) -> Result<Request> {
    Ok(Request::get(url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Referer", BASE_URL))
}

pub struct MangasScans;

impl Source for MangasScans {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let search_query = query.unwrap_or_default();
        let filter_params = build_filter_params(filters);

        let url = if !search_query.is_empty() {
            let encoded = urlencode(search_query);
            format!("{}/manga/?title={}&page={}{}", BASE_URL, encoded, page, filter_params)
        } else {
            format!("{}/manga/?page={}{}", BASE_URL, page, filter_params)
        };

        let html = build_request(&url)?.html()?;

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

            let html = build_request(&manga_url)?.html()?;

            if needs_details {
                updated_manga = parse_manga_details(&html, BASE_URL, manga.key.clone())?;
                send_partial_result(&updated_manga);
            }

            if needs_chapters {
                let total_pages = detect_pagination(&html);
                let mut all_chapters = Vec::new();

                let first_page_chapters = parse_chapter_list(&html, BASE_URL);
                all_chapters.extend(first_page_chapters);

                for page in 2..=total_pages {
                    let page_url = format!("{}?page={}", manga_url, page);
                    let page_html = build_request(&page_url)?.html()?;

                    let page_chapters = parse_chapter_list(&page_html, BASE_URL);
                    all_chapters.extend(page_chapters);
                }

                updated_manga.chapters = Some(all_chapters);
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

        let html = build_request(&chapter_url)?.html()?;

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

        let html = build_request(&url)?.html()?;

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
