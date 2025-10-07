#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format, string::ToString},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod helper;
mod parser;

use helper::urlencode;
use parser::{parse_chapter_list, parse_manga_details, parse_manga_list, parse_page_list, has_next_page};

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
        let mut genre_filter = String::new();
        let mut status_filter = String::new();
        let mut type_filter = String::new();

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "genre" && !value.is_empty() {
                        genre_filter = match value.as_str() {
                            "Action" => "action",
                            "Aventure" => "aventure",
                            "Comédie" => "comedie",
                            "Drame" => "drame",
                            "Fantaisie" => "fantaisie",
                            "Horreur" => "horreur",
                            "Romance" => "romance",
                            "Science-Fiction" => "science-fiction",
                            "Tranche de vie" => "tranche-de-vie",
                            "Seinen" => "seinen",
                            "Shōnen" => "shonen",
                            "Shōjo" => "shoujo",
                            _ => "",
                        }
                        .to_string();
                    } else if id == "status" && !value.is_empty() {
                        status_filter = match value.as_str() {
                            "En cours" => "en-cours",
                            "Terminé" => "termine",
                            _ => "",
                        }
                        .to_string();
                    } else if id == "type" && !value.is_empty() {
                        type_filter = match value.as_str() {
                            "Manga" => "manga",
                            "Manhwa" => "manhwa",
                            "Manhua" => "manhua",
                            _ => "",
                        }
                        .to_string();
                    }
                }
                _ => {}
            }
        }

        let mut url = if let Some(q) = query {
            let encoded_query = urlencode(q);
            format!("{}/page/{}/?s={}&post_type=wp-manga", BASE_URL, page, encoded_query)
        } else {
            format!("{}/page/{}/", BASE_URL, page)
        };

        if !genre_filter.is_empty() {
            url.push_str(&format!("&genre={}", genre_filter));
        }
        if !status_filter.is_empty() {
            url.push_str(&format!("&status={}", status_filter));
        }
        if !type_filter.is_empty() {
            url.push_str(&format!("&type={}", type_filter));
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

                let cover = if let Some(imgs) = item.select("img") {
                    if let Some(img) = imgs.first() {
                        let cover_url = img.attr("data-src")
                            .or_else(|| img.attr("data-lazy-src"))
                            .or_else(|| img.attr("src"))
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

                    let cover = if let Some(imgs) = item.select("img") {
                        if let Some(img) = imgs.first() {
                            let cover_url = img.attr("data-src")
                                .or_else(|| img.attr("data-lazy-src"))
                                .or_else(|| img.attr("src"))
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
