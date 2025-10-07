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
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let mut genre_filters: Vec<String> = Vec::new();
        let mut status_filter = String::new();
        let mut type_filter = String::new();
        let mut sort_filter = String::new();

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "sort" && !value.is_empty() && value != "Défaut" {
                        sort_filter = match value.as_str() {
                            "A-Z" => String::from("title"),
                            "Z-A" => String::from("titlereverse"),
                            "Dernières sorties" => String::from("update"),
                            "Nouveau" => String::from("latest"),
                            "Populaire" => String::from("popular"),
                            _ => String::new(),
                        };
                    } else if id == "status" && !value.is_empty() && value != "Tous" {
                        status_filter = match value.as_str() {
                            "En cours" => String::from("ongoing"),
                            "Terminé" => String::from("completed"),
                            "En pause" => String::from("hiatus"),
                            _ => String::new(),
                        };
                    } else if id == "type" && !value.is_empty() && value != "Tous" {
                        type_filter = match value.as_str() {
                            "Manga" => String::from("manga"),
                            "Manhwa" => String::from("manhwa"),
                            "Manhua" => String::from("manhua"),
                            "Comic" => String::from("comic"),
                            "Novel" => String::from("novel"),
                            _ => String::new(),
                        };
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" {
                        for genre_name in included {
                            if !genre_name.is_empty() {
                                let genre_id = match genre_name.as_str() {
                                    "Académie" => "130",
                                    "Action" => "2",
                                    "Action Aventure" => "22",
                                    "Art Martiaux" => "34",
                                    "Arts martiaux" => "8",
                                    "Arts martiaux Aventure" => "30",
                                    "Aventure" => "3",
                                    "Combat" => "9",
                                    "Demon" => "91",
                                    "Dieu" => "60",
                                    "Donjon" => "116",
                                    "Dragon" => "79",
                                    "Drame" => "42",
                                    "Fantastique" => "27",
                                    "Fantasy" => "125",
                                    "Guilde" => "138",
                                    "Héro" => "96",
                                    "Horreur" => "15",
                                    "Humour" => "101",
                                    "Joueur" => "104",
                                    "Magie" => "54",
                                    "Manga" => "124",
                                    "Murim" => "77",
                                    "Muscu" => "100",
                                    "Mystère" => "43",
                                    "Necromancie" => "132",
                                    "Psychologique" => "32",
                                    "Regression" => "165",
                                    "Réincarnation" => "94",
                                    "Restaurent" => "154",
                                    "Résurrection" => "118",
                                    "Romance" => "156",
                                    "Sang" => "162",
                                    "Shônen" => "10",
                                    "Surnaturel" => "40",
                                    "Tour" => "81",
                                    "Vengeance" => "137",
                                    "Vie scolaire" => "44",
                                    "Webtoons" => "46",
                                    _ => "",
                                };
                                if !genre_id.is_empty() {
                                    genre_filters.push(String::from(genre_id));
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        let search_query = query.unwrap_or_default();
        let has_search = !search_query.is_empty();

        // Build URL - don't include s= parameter if search is empty (it breaks filters)
        let mut url = if has_search {
            let encoded_query = urlencode(search_query);
            if page == 1 {
                format!("{}/manga/?s={}", BASE_URL, encoded_query)
            } else {
                format!("{}/manga/page/{}/?s={}", BASE_URL, page, encoded_query)
            }
        } else {
            if page == 1 {
                format!("{}/manga/?", BASE_URL)
            } else {
                format!("{}/manga/page/{}/?", BASE_URL, page)
            }
        };

        // Track if we've added any parameters (to know whether to use & or not)
        let mut has_params = has_search;

        for genre in genre_filters {
            if has_params {
                url.push_str("&");
            }
            url.push_str(&format!("genre%5B%5D={}", genre));
            has_params = true;
        }

        if !status_filter.is_empty() {
            if has_params {
                url.push_str("&");
            }
            url.push_str(&format!("status={}", status_filter));
            has_params = true;
        }

        if !type_filter.is_empty() {
            if has_params {
                url.push_str("&");
            }
            url.push_str(&format!("type={}", type_filter));
            has_params = true;
        }

        if !sort_filter.is_empty() {
            if has_params {
                url.push_str("&");
            }
            url.push_str(&format!("order={}", sort_filter));
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
