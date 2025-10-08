#![no_std]

use aidoku::{
    Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult,
    MangaStatus, ContentRating, Viewer, Page, Result, Source,
    alloc::{String, Vec, format},
    imports::{net::{Request, HttpMethod}, html::Document},
    prelude::*,
};

extern crate alloc;

mod parser;
mod helper;

pub const BASE_URL: &str = "https://epsilonscan.to";
pub const AJAX_URL: &str = "https://epsilonscan.to/wp-admin/admin-ajax.php";

struct EpsilonScan;

impl EpsilonScan {
    fn make_request(&self, url: &str, method: HttpMethod) -> Request {
        Request::new(url, method)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("Referer", BASE_URL)
            .header("x-requested-with", "app.notMihon")
    }

    fn get_ajax_manga_list(&self, page: i32, meta_key: &str) -> Result<MangaPageResult> {
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core%2Fcontent%2Fcontent-archive&vars%5Bpaged%5D=1&vars%5Borderby%5D=meta_value_num&vars%5Btemplate%5D=archive&vars%5Bsidebar%5D=full&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Bmeta_key%5D={}&vars%5Border%5D=desc&vars%5Bmeta_query%5D%5Brelation%5D=OR&vars%5Bmanga_archives_item_layout%5D=big_thumbnail",
            page - 1,
            meta_key
        );

        let response = self.make_request(AJAX_URL, HttpMethod::Post)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.as_bytes())
            .send()?;

        let html = Document::from(response.body_string()?);

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
        let response = self.make_request(&url, HttpMethod::Get).send()?;
        let html = Document::from(response.body_string()?);
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
            let response = self.make_request(&url, HttpMethod::Get).send()?;
            let html = Document::from(response.body_string()?);

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
        let response = self.make_request(&url, HttpMethod::Get).send()?;
        let html = Document::from(response.body_string()?);
        parser::parse_page_list(html)
    }
}

impl EpsilonScan {
    fn get_ajax_chapters(&self, manga_id: &str, post_id: &str) -> Result<Vec<Chapter>> {
        let body = format!("action=manga_get_chapters&manga={}", post_id);

        let response = self.make_request(AJAX_URL, HttpMethod::Post)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .body(body.as_bytes())
            .send()?;

        let html = Document::from(response.body_string()?);

        parser::parse_chapter_list(manga_id, html)
    }
}

impl ListingProvider for EpsilonScan {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.name.as_str() {
            "Populaire" => self.get_ajax_manga_list(page, "_wp_manga_views"),
            _ => self.get_ajax_manga_list(page, "_latest_update"),
        }
    }
}

register_source!(EpsilonScan, ListingProvider);
