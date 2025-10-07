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
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub struct JapScan;

impl JapScan {
    fn get_html(&self, url: &str) -> Result<aidoku::imports::html::Document> {
        use aidoku::AidokuError;

        let mut attempt = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let request = Request::get(url)?
                .header("User-Agent", USER_AGENT)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("DNT", "1")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .header("Referer", &format!("{}/", BASE_URL));

            let response = match request.send() {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::RequestError(e));
                    }
                    attempt += 1;
                    continue;
                }
            };

            match response.status_code() {
                200..=299 => return Ok(response.get_html()?),
                403 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Cloudflare block (403)"));
                    }
                    attempt += 1;
                    continue;
                },
                429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Rate limited (429)"));
                    }
                    attempt += 1;
                    continue;
                },
                503 | 502 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                },
                _ => return Err(AidokuError::message("Request failed")),
            }
        }
    }

    fn search_by_query(&self, query: String) -> Result<MangaPageResult> {
        use aidoku::AidokuError;

        let encoded_query = helper::urlencode(query);
        let body = format!("search={}", encoded_query);

        let mut attempt = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let request = Request::post(&format!("{}/ls/", BASE_URL))?
                .header("User-Agent", USER_AGENT)
                .header("Content-Type", "application/x-www-form-urlencoded")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Referer", &format!("{}/", BASE_URL))
                .header("Origin", BASE_URL)
                .header("Accept", "application/json, text/plain, */*")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Accept-Encoding", "gzip, deflate, br")
                .body(body.as_bytes());

            let response = match request.send() {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::RequestError(e));
                    }
                    attempt += 1;
                    continue;
                }
            };

            let status = response.status_code();
            match status {
                200..=299 => {
                    let json_str = response.get_string()?;
                    return parser::parse_search_json(json_str);
                },
                403 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Cloudflare block (403)"));
                    }
                    attempt += 1;
                    continue;
                },
                429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Rate limited (429)"));
                    }
                    attempt += 1;
                    continue;
                },
                503 | 502 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                },
                _ => {
                    return Err(AidokuError::message(&format!("Search request failed with status {}", status)));
                },
            }
        }
    }

    fn get_spoiler_preference(&self) -> bool {
        true
    }

    fn get_string(&self, url: &str) -> Result<String> {
        use aidoku::AidokuError;

        let mut attempt = 0;
        const MAX_RETRIES: u32 = 3;

        loop {
            let request = Request::get(url)?
                .header("User-Agent", USER_AGENT)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("DNT", "1")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .header("Referer", &format!("{}/", BASE_URL));

            let response = match request.send() {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::RequestError(e));
                    }
                    attempt += 1;
                    continue;
                }
            };

            match response.status_code() {
                200..=299 => return response.get_string(),
                403 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Cloudflare block (403)"));
                    }
                    attempt += 1;
                    continue;
                },
                429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Rate limited (429)"));
                    }
                    attempt += 1;
                    continue;
                },
                503 | 502 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                },
                _ => return Err(AidokuError::message("Request failed")),
            }
        }
    }
}

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
        let html = self.get_html(&url)?;

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

            let html = self.get_html(&url)?;

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

            let html = self.get_html(&url)?;

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

        let html_content = self.get_string(&url)?;

        parser::parse_page_list(html_content, BASE_URL)
    }
}

impl ListingProvider for JapScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.name.as_str() {
            "Populaire" => format!("{}/mangas/?sort=popular&p={}", BASE_URL, page),
            "Dernières Mises à Jour" => format!("{}/mangas/?sort=updated&p={}", BASE_URL, page),
            _ => format!("{}/mangas/?sort=popular&p={}", BASE_URL, page),
        };

        let html = self.get_html(&url)?;

        parser::parse_manga_list(html)
    }
}

register_source!(JapScan, ListingProvider);
