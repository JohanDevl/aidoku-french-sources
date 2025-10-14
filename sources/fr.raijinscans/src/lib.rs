#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document, std::send_partial_result},
    prelude::*,
};

extern crate alloc;

mod helper;
mod parser;

use helper::urlencode;
use parser::{parse_chapter_list, parse_manga_details, parse_page_list, has_next_page};

pub static BASE_URL: &str = "https://raijinscan.co";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub struct RaijinScans;

impl Source for RaijinScans {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let mut genre_filters: Vec<String> = Vec::new();
        let mut status_filters: Vec<String> = Vec::new();
        let mut type_filters: Vec<String> = Vec::new();
        let mut release_filters: Vec<String> = Vec::new();
        let mut sort_filter = String::from("recently_added");

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "sort" && !value.is_empty() {
                        sort_filter = value;
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    match id.as_str() {
                        "genre" => {
                            for genre_id in included {
                                if !genre_id.is_empty() {
                                    genre_filters.push(genre_id);
                                }
                            }
                        }
                        "status" => {
                            for status_id in included {
                                if !status_id.is_empty() {
                                    status_filters.push(status_id);
                                }
                            }
                        }
                        "type" => {
                            for type_id in included {
                                if !type_id.is_empty() {
                                    type_filters.push(type_id);
                                }
                            }
                        }
                        "release" => {
                            for release_id in included {
                                if !release_id.is_empty() {
                                    release_filters.push(release_id);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        let search_query = query.unwrap_or_default();
        let encoded_query = if !search_query.is_empty() {
            urlencode(search_query)
        } else {
            String::new()
        };

        let mut url = if page == 1 {
            format!("{}/?post_type=wp-manga&s={}&sort={}", BASE_URL, encoded_query, sort_filter)
        } else {
            format!("{}/page/{}/?post_type=wp-manga&s={}&sort={}", BASE_URL, page, encoded_query, sort_filter)
        };

        Self::append_filter_params(&mut url, &genre_filters, "genre");
        Self::append_filter_params(&mut url, &status_filters, "status");
        Self::append_filter_params(&mut url, &type_filters, "type");
        Self::append_filter_params(&mut url, &release_filters, "release");

        let html = Self::create_html_request(&url)?;

        let mangas = self.parse_search_results(&html);
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

            let html = Self::create_html_request(&manga_url)?;

            if needs_details {
                updated_manga = parse_manga_details(&html, manga.key.clone(), BASE_URL)?;
                send_partial_result(&updated_manga);
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

        let html = Self::create_html_request(&chapter_url)?;

        Ok(parse_page_list(&html))
    }
}

impl ListingProvider for RaijinScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.id.as_str() {
            "popular" => self.get_popular_manga(page),
            "latest" => self.get_latest_manga(page),
            _ => self.get_latest_manga(page),
        }
    }
}

impl ImageRequestProvider for RaijinScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .header("Accept", "image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8"))
    }
}

impl RaijinScans {
    fn create_html_request(url: &str) -> Result<Document> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?)
    }

    fn append_filter_params(url: &mut String, filters: &[String], param_name: &str) {
        for filter in filters {
            url.push_str(&format!("&{}%5B%5D={}", param_name, filter));
        }
    }

    fn parse_manga_item(item: &aidoku::imports::html::Element, link_selector: &str, img_selector: &str) -> Option<Manga> {
        let link = item.select(link_selector)?.first()?;

        let url = link.attr("href").unwrap_or_default();
        let title = link.text().unwrap_or_default();

        if url.is_empty() || title.is_empty() {
            return None;
        }

        let cover = if let Some(imgs) = item.select(img_selector) {
            let mut cover_img = None;
            for img in imgs {
                let is_flag = img.attr("class").map(|c| c.contains("flag-icon")).unwrap_or(false);
                if !is_flag {
                    cover_img = Some(img);
                    break;
                }
            }

            if let Some(img) = cover_img {
                let cover_url = img.attr("src")
                    .or_else(|| img.attr("data-src"))
                    .or_else(|| img.attr("data-lazy-src"))
                    .unwrap_or_default();

                if !cover_url.is_empty() {
                    Some(helper::make_absolute_url(BASE_URL, &cover_url))
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        Some(Manga {
            key: url.clone(),
            cover,
            title,
            authors: None,
            artists: None,
            description: None,
            tags: None,
            status: aidoku::MangaStatus::Unknown,
            content_rating: aidoku::ContentRating::Safe,
            viewer: aidoku::Viewer::LeftToRight,
            chapters: None,
            url: Some(helper::make_absolute_url(BASE_URL, &url)),
            next_update_time: None,
            update_strategy: aidoku::UpdateStrategy::Always,
        })
    }

    fn parse_search_results(&self, html: &Document) -> Vec<Manga> {
        let mut mangas = Vec::new();

        if let Some(items) = html.select("div.unit") {
            for item in items {
                if let Some(manga) = Self::parse_manga_item(&item, "div.info a", "div.poster-image-wrapper > img") {
                    mangas.push(manga);
                }
            }
        }

        mangas
    }

    fn get_popular_manga(&self, _page: i32) -> Result<MangaPageResult> {
        let html = Self::create_html_request(BASE_URL)?;

        let mut mangas = Vec::new();
        let mut seen_urls = Vec::new();

        if let Some(items) = html.select("section#most-viewed div.swiper-slide.unit") {
            for item in items {
                if let Some(manga) = Self::parse_manga_item(&item, "a.c-title", "a.poster div.poster-image-wrapper > img") {
                    if !seen_urls.contains(&manga.key) {
                        seen_urls.push(manga.key.clone());
                        mangas.push(manga);
                    }
                }
            }
        }

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: false,
        })
    }

    fn get_latest_manga(&self, page: i32) -> Result<MangaPageResult> {
        if page == 1 {
            let html = Self::create_html_request(BASE_URL)?;

            let mut mangas = Vec::new();

            if let Some(items) = html.select("section.recently-updated div.unit") {
                for item in items {
                    if let Some(manga) = Self::parse_manga_item(&item, "div.info a", "div.poster-image-wrapper > img") {
                        mangas.push(manga);
                    }
                }
            }

            return Ok(MangaPageResult {
                entries: mangas,
                has_next_page: false,
            });
        }

        Ok(MangaPageResult {
            entries: Vec::new(),
            has_next_page: false,
        })
    }
}

register_source!(RaijinScans, ListingProvider, ImageRequestProvider);
