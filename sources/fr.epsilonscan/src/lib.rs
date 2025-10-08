#![no_std]

use aidoku::{
    Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult,
    Page, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;

mod parser;
mod helper;

pub const BASE_URL: &str = "https://epsilonscan.to";
pub const AJAX_URL: &str = "https://epsilonscan.to/wp-admin/admin-ajax.php";

struct EpsilonScan;

impl EpsilonScan {
    fn get_catalogue_page(&self, page: i32, order: &str) -> Result<MangaPageResult> {
        let url = if page > 1 {
            format!("{}/manga/page/{}/?m_orderby={}", BASE_URL, page, order)
        } else {
            format!("{}/manga/?m_orderby={}", BASE_URL, order)
        };

        let html = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Referer", BASE_URL)
            .html()?;

        parser::parse_manga_list(html)
    }

    fn build_search_url(&self, query: Option<String>, page: i32, filters: Vec<FilterValue>) -> String {
        let mut url = format!("{}/page/{}/", BASE_URL, page);
        let mut params = Vec::new();

        if let Some(q) = query {
            params.push(format!("s={}&post_type=wp-manga", helper::urlencode(q)));
        }

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if value.is_empty() || value == "Tout" {
                        continue;
                    }

                    match id.as_str() {
                        "status" => {
                            let status_value = match value.as_str() {
                                "En cours" => "on-going",
                                "Terminé" => "end",
                                "En pause" => "on-hold",
                                "Annulé" => "canceled",
                                _ => continue,
                            };
                            params.push(format!("status[]={}", status_value));
                        }
                        "type" => {
                            let type_value = value.to_lowercase();
                            if type_value != "tout" {
                                params.push(format!("wp-manga-type={}", type_value));
                            }
                        }
                        _ => {}
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" {
                        for genre in included {
                            if !genre.is_empty() {
                                params.push(format!("genre[]={}", helper::urlencode(genre.to_lowercase())));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }

        url
    }
}

impl Source for EpsilonScan {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let url = self.build_search_url(query, page, filters);
        let html = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Referer", BASE_URL)
            .html()?;
        parser::parse_manga_list(html)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        if needs_details || needs_chapters {
            let url = format!("{}/manga/{}/", BASE_URL, manga.key);
            let html = Request::get(&url)?
                .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
                .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("Accept-Encoding", "gzip, deflate, br")
                .header("DNT", "1")
                .header("Connection", "keep-alive")
                .header("Referer", BASE_URL)
                .html()?;

            if needs_details {
                if let Ok(detailed) = parser::parse_manga_details(&manga.key, &html) {
                    manga.title = detailed.title;
                    manga.cover = detailed.cover;
                    manga.authors = detailed.authors;
                    manga.artists = detailed.artists;
                    manga.description = detailed.description;
                    manga.url = detailed.url;
                    manga.tags = detailed.tags;
                    manga.status = detailed.status;
                    manga.content_rating = detailed.content_rating;
                    manga.viewer = detailed.viewer;
                }
            }

            if needs_chapters {
                let post_id = parser::extract_post_id(&html)?;
                manga.chapters = Some(self.get_ajax_chapters(&manga.key, &post_id)?);
            }
        }

        Ok(manga)
    }

    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/manga/{}/{}/", BASE_URL, manga.key, chapter.key);
        let html = Request::get(&url)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Referer", BASE_URL)
            .html()?;
        parser::parse_page_list(html)
    }
}

impl EpsilonScan {
    fn get_ajax_chapters(&self, manga_id: &str, post_id: &str) -> Result<Vec<Chapter>> {
        let body = format!("action=manga_get_chapters&manga={}", post_id);

        let html = Request::post(AJAX_URL)?
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Referer", BASE_URL)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("x-requested-with", "app.notMihon")
            .body(body.as_bytes())
            .html()?;

        parser::parse_chapter_list(manga_id, html)
    }
}

impl ListingProvider for EpsilonScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.name.as_str() {
            "Populaire" => self.get_catalogue_page(page, "views"),
            _ => self.get_catalogue_page(page, "latest"),
        }
    }
}

register_source!(EpsilonScan, ListingProvider);
