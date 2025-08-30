#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, Page, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document},
};

extern crate alloc;
use alloc::vec;

mod parser;
mod helper;

pub static BASE_URL: &str = "https://lelscanfr.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

pub struct LelscanFr;

impl Source for LelscanFr {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        let mut query_params = String::new();
        
        // Add search query if provided
        if let Some(search_query) = query {
            query_params.push_str(&format!("&title={}", helper::urlencode(search_query)));
        }
        
        // Process filters - ignore for now
        let _ = filters;
        
        let url = format!("{}/manga?page={}{}", BASE_URL, page, query_params);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .html()?;
        
        parser::parse_manga_list(html)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{}/manga/{}", BASE_URL, manga.key);
        let html = Request::get(&url)?.html()?;
        
        if needs_details {
            manga = parser::parse_manga_details(manga, &html)?;
        }
        
        if needs_chapters {
            // Check for pagination and handle multi-page chapter lists
            if let Some(_pagination) = html.select(".pagination") {
                if let Some(first_pagination) = _pagination.first() {
                    let pagination_text = first_pagination.text().unwrap_or_default();
                    if !pagination_text.is_empty() {
                        // Extract number of pages and fetch all
                        let pagination_links = html.select(".pagination a");
                        let mut max_page = 1;
                        
                        if let Some(links) = pagination_links {
                            for link in links {
                                if let Some(link_text) = link.text() {
                                    if let Ok(page_num) = link_text.parse::<i32>() {
                                        if page_num > max_page {
                                            max_page = page_num;
                                        }
                                    }
                                }
                            }
                        }
                        
                        let mut all_docs: Vec<Document> = vec![html];
                        for page in 2..=max_page {
                            let page_url = format!("{}/manga/{}?page={}", BASE_URL, manga.key, page);
                            let page_html = Request::get(&page_url)?.html()?;
                            all_docs.push(page_html);
                        }
                        manga.chapters = Some(parser::parse_chapter_list(&manga.key, all_docs)?);
                    } else {
                        manga.chapters = Some(parser::parse_chapter_list(&manga.key, vec![html])?);
                    }
                }
            } else {
                manga.chapters = Some(parser::parse_chapter_list(&manga.key, vec![html])?);
            }
        }
        
        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/{}", BASE_URL, chapter.key);
        let html = Request::get(&url)?.html()?;
        parser::parse_page_list(&html)
    }
}

impl ListingProvider for LelscanFr {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        // Match the listing ID like MangaScantrad does
        match listing.id.as_str() {
            "test" => self.get_test_mangas(page),
            _ => self.get_test_mangas(page), // Default fallback
        }
    }
}

impl LelscanFr {
    fn get_test_mangas(&self, page: i32) -> Result<MangaPageResult> {
        // HARDCODED TEST MANGAS - No HTTP requests, no parsing, just direct return
        let test_mangas = vec![
            Manga {
                key: String::from("test-manga-1"),
                cover: Some(String::from("https://via.placeholder.com/150x200.png?text=Test1")),
                title: format!("TEST: Manga de Test 1 (Page {})", page),
                authors: Some(vec![String::from("Auteur Test")]),
                artists: Some(vec![String::from("Artiste Test")]),
                description: Some(String::from("Ceci est un manga de test pour vérifier si LelscanFR fonctionne correctement.")),
                tags: Some(vec![String::from("Test"), String::from("Debug")]),
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                url: Some(String::from("https://lelscanfr.com/manga/test-manga-1")),
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            },
            Manga {
                key: String::from("test-manga-2"),
                cover: Some(String::from("https://via.placeholder.com/150x200.png?text=Test2")),
                title: String::from("TEST: Manga de Test 2 - Ça marche !"),
                authors: Some(vec![String::from("Claude AI")]),
                artists: Some(vec![String::from("Debug Artist")]),
                description: Some(String::from("Si vous voyez ce manga, c'est que l'implémentation ListingProvider fonctionne.")),
                tags: Some(vec![String::from("Success"), String::from("Working")]),
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                url: Some(String::from("https://lelscanfr.com/manga/test-manga-2")),
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            },
            Manga {
                key: String::from("test-manga-3"),
                cover: None,
                title: String::from("TEST: Manga sans Cover - ListingProvider OK"),
                authors: None,
                artists: None,
                description: Some(String::from("Test pour vérifier les mangas sans métadonnées.")),
                tags: None,
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                url: Some(String::from("https://lelscanfr.com/manga/test-manga-3")),
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            },
        ];

        Ok(MangaPageResult {
            entries: test_mangas,
            has_next_page: page < 3, // Simulate pagination for first 3 pages
        })
    }
}

impl ImageRequestProvider for LelscanFr {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}