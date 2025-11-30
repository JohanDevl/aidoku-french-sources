#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider,
    Manga, MangaPageResult, Page, PageContext, Result, Source,
    alloc::{String, Vec, format},
    imports::net::Request,
    prelude::*,
};

extern crate alloc;
extern crate serde_json;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://crunchyscan.fr";
pub static API_URL: &str = "https://crunchyscan.fr/api/manga/search/advance";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

pub struct CrunchyScan;

impl Source for CrunchyScan {
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

        // Process filters
        let mut included_status: Vec<String> = Vec::new();
        let mut excluded_status: Vec<String> = Vec::new();
        let mut included_types: Vec<String> = Vec::new();
        let mut excluded_types: Vec<String> = Vec::new();
        let mut included_genres: Vec<String> = Vec::new();
        let mut excluded_genres: Vec<String> = Vec::new();
        let mut order_with = String::from("Récent");
        let mut order_by = String::from("desc");

        for filter in filters {
            match filter {
                FilterValue::MultiSelect { id, included, excluded } => {
                    match id.as_str() {
                        "status" => {
                            for status in included {
                                if !status.is_empty() {
                                    included_status.push(status);
                                }
                            }
                            for status in excluded {
                                if !status.is_empty() {
                                    excluded_status.push(status);
                                }
                            }
                        }
                        "type" => {
                            for t in included {
                                if !t.is_empty() {
                                    included_types.push(t);
                                }
                            }
                            for t in excluded {
                                if !t.is_empty() {
                                    excluded_types.push(t);
                                }
                            }
                        }
                        "genres" => {
                            for g in included {
                                if !g.is_empty() {
                                    included_genres.push(g);
                                }
                            }
                            for g in excluded {
                                if !g.is_empty() {
                                    excluded_genres.push(g);
                                }
                            }
                        }
                        _ => {}
                    }
                }
                FilterValue::Select { id, value } => {
                    match id.as_str() {
                        "orderwith" => {
                            if !value.is_empty() {
                                order_with = value;
                            }
                        }
                        "orderby" => {
                            if !value.is_empty() {
                                order_by = value;
                            }
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Step 1: Get the catalog page to extract CSRF token
        let catalog_url = format!("{}/catalog", BASE_URL);
        let catalog_response = Request::get(&catalog_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .send()?;

        let html_string = catalog_response.get_string()?;

        // Extract CSRF token from meta tag: <meta name="csrf-token" content="...">
        let csrf_token = helper::extract_csrf_token(&html_string).unwrap_or_default();

        // Build status params
        let status_params = if !included_status.is_empty() {
            // User selected specific status to include
            included_status.iter()
                .map(|s| format!("status%5B%5D={}", helper::urlencode(s)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            // Default: include all status
            String::from("status%5B%5D=En+cours&status%5B%5D=Termin%C3%A9&status%5B%5D=Abandonn%C3%A9")
        };

        // Build exclude_status params
        let exclude_status_params = if !excluded_status.is_empty() {
            excluded_status.iter()
                .map(|s| format!("exclude_status%5B%5D={}", helper::urlencode(s)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        // Build types params
        let types_params = if !included_types.is_empty() {
            included_types.iter()
                .map(|t| format!("types%5B%5D={}", helper::urlencode(t)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        // Build exclude_types params
        let exclude_types_params = if !excluded_types.is_empty() {
            excluded_types.iter()
                .map(|t| format!("exclude_types%5B%5D={}", helper::urlencode(t)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        // Build genres params
        let genres_params = if !included_genres.is_empty() {
            included_genres.iter()
                .map(|g| format!("genres%5B%5D={}", helper::urlencode(g)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        // Build exclude_genres params
        let exclude_genres_params = if !excluded_genres.is_empty() {
            excluded_genres.iter()
                .map(|g| format!("exclude_genres%5B%5D={}", helper::urlencode(g)))
                .collect::<Vec<_>>()
                .join("&")
        } else {
            String::new()
        };

        // Step 2: Make the API POST request with CSRF token
        let mut body = format!(
            "affichage=grid&team=&artist=&author=&page={}&chapters%5B%5D=0&chapters%5B%5D=9999&searchTerm={}&orderWith={}&orderBy={}&{}",
            page,
            helper::urlencode(&search_query),
            helper::urlencode(&order_with),
            helper::urlencode(&order_by),
            status_params
        );

        // Add exclude_status if any
        if !exclude_status_params.is_empty() {
            body = format!("{}&{}", body, exclude_status_params);
        }

        // Add types if any
        if !types_params.is_empty() {
            body = format!("{}&{}", body, types_params);
        }

        // Add exclude_types if any
        if !exclude_types_params.is_empty() {
            body = format!("{}&{}", body, exclude_types_params);
        }

        // Add genres if any
        if !genres_params.is_empty() {
            body = format!("{}&{}", body, genres_params);
        }

        // Add exclude_genres if any
        if !exclude_genres_params.is_empty() {
            body = format!("{}&{}", body, exclude_genres_params);
        }

        let response = Request::post(API_URL)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "application/json, text/javascript, */*; q=0.01")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Content-Type", "application/x-www-form-urlencoded; charset=UTF-8")
            .header("X-Requested-With", "XMLHttpRequest")
            .header("X-Csrf-Token", &csrf_token)
            .header("Origin", BASE_URL)
            .header("Referer", &catalog_url)
            .body(body.as_bytes())
            .send()?;

        let json_string = response.get_string()?;

        // Debug: log the response
        let preview = if json_string.len() > 500 {
            &json_string[..500]
        } else {
            &json_string
        };
        println!("[CrunchyScan] API Response (first 500 chars): {}", preview);
        println!("[CrunchyScan] Response length: {}", json_string.len());

        parser::parse_manga_list_json(&json_string, page)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{}/lecture-en-ligne/{}", BASE_URL, manga.key);
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

        if needs_details {
            let details = parser::parse_manga_details(&html, &manga.key)?;
            manga.title = details.title;
            manga.description = details.description;
            manga.tags = details.tags;
            manga.cover = details.cover;
            manga.status = details.status;
            manga.authors = details.authors;
        }

        if needs_chapters {
            manga.chapters = Some(parser::parse_chapter_list(&html)?);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let html = Request::get(&chapter.key)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        parser::parse_page_list(&html)
    }
}

impl ImageRequestProvider for CrunchyScan {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

register_source!(CrunchyScan, ImageRequestProvider);
