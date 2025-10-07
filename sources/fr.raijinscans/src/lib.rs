#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document},
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

        for genre in genre_filters {
            url.push_str(&format!("&genre%5B%5D={}", genre));
        }
        for status in status_filters {
            url.push_str(&format!("&status%5B%5D={}", status));
        }
        for type_val in type_filters {
            url.push_str(&format!("&type%5B%5D={}", type_val));
        }
        for release in release_filters {
            url.push_str(&format!("&release%5B%5D={}", release));
        }

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
    fn parse_search_results(&self, html: &Document) -> Vec<Manga> {
        let mut mangas = Vec::new();

        if let Some(items) = html.select("div.unit") {
            for item in items {
                let link = if let Some(links) = item.select("div.info a") {
                    if let Some(l) = links.first() {
                        l
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let url = link.attr("href").unwrap_or_default();
                let title = link.text().unwrap_or_default();

                if url.is_empty() || title.is_empty() {
                    continue;
                }

                let key = url.clone();

                let cover = if let Some(imgs) = item.select("div.poster-image-wrapper > img") {
                    if let Some(img) = imgs.first() {
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

                mangas.push(Manga {
                    key: key.clone(),
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
                });
            }
        }

        mangas
    }

    fn get_popular_manga(&self, _page: i32) -> Result<MangaPageResult> {
        let html = Request::get(BASE_URL)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        let mut mangas = Vec::new();

        if let Some(items) = html.select("section#most-viewed div.swiper-slide.unit") {
            for item in items {
                let link = if let Some(links) = item.select("a.c-title") {
                    if let Some(l) = links.first() {
                        l
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let url = link.attr("href").unwrap_or_default();
                let title = link.text().unwrap_or_default();

                if url.is_empty() || title.is_empty() {
                    continue;
                }

                let key = url.clone();

                let cover = if let Some(imgs) = item.select("a.poster div.poster-image-wrapper > img") {
                    if let Some(img) = imgs.first() {
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

                mangas.push(Manga {
                    key: key.clone(),
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
                });
            }
        }

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: false,
        })
    }

    fn get_latest_manga(&self, page: i32) -> Result<MangaPageResult> {
        if page == 1 {
            let html = Request::get(BASE_URL)?
                .header("User-Agent", USER_AGENT)
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("DNT", "1")
                .header("Connection", "keep-alive")
                .header("Upgrade-Insecure-Requests", "1")
                .header("Referer", BASE_URL)
                .html()?;

            let mut mangas = Vec::new();

            if let Some(items) = html.select("section.recently-updated div.unit") {
                for item in items {
                    let link = if let Some(links) = item.select("div.info a") {
                        if let Some(l) = links.first() {
                            l
                        } else {
                            continue;
                        }
                    } else {
                        continue;
                    };

                    let url = link.attr("href").unwrap_or_default();
                    let title = link.text().unwrap_or_default();

                    if url.is_empty() || title.is_empty() {
                        continue;
                    }

                    let key = url.clone();

                    let cover = if let Some(imgs) = item.select("div.poster-image-wrapper > img") {
                        if let Some(img) = imgs.first() {
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

                    mangas.push(Manga {
                        key: key.clone(),
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
                    });
                }
            }

            let has_more = false;

            return Ok(MangaPageResult {
                entries: mangas,
                has_next_page: has_more,
            });
        }

        Ok(MangaPageResult {
            entries: Vec::new(),
            has_next_page: false,
        })
    }
}

register_source!(RaijinScans, ListingProvider, ImageRequestProvider);
