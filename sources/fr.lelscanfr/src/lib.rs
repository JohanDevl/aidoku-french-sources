#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, Page, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document},
    prelude::*,
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
            // Try real parsing first
            let chapters_result = if let Some(_pagination) = html.select(".pagination") {
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
                        parser::parse_chapter_list(&manga.key, all_docs)?
                    } else {
                        // Get fresh HTML for parsing
                        let fresh_url = format!("{}/manga/{}", BASE_URL, manga.key);
                        let fresh_html = Request::get(&fresh_url)?.html()?;
                        parser::parse_chapter_list(&manga.key, vec![fresh_html])?
                    }
                }
                else {
                    // Get fresh HTML for parsing  
                    let fresh_url = format!("{}/manga/{}", BASE_URL, manga.key);
                    let fresh_html = Request::get(&fresh_url)?.html()?;
                    parser::parse_chapter_list(&manga.key, vec![fresh_html])?
                }
            } else {
                // Get fresh HTML for parsing
                let fresh_url = format!("{}/manga/{}", BASE_URL, manga.key);
                let fresh_html = Request::get(&fresh_url)?.html()?;
                parser::parse_chapter_list(&manga.key, vec![fresh_html])?
            };
            
            // If no chapters found, create debug chapters
            if chapters_result.is_empty() {
                let mut debug_chapters: Vec<Chapter> = Vec::new();
                
                // Fetch fresh HTML for debugging
                let debug_url = format!("{}/manga/{}", BASE_URL, manga.key);
                let debug_html = Request::get(&debug_url)?.html()?;
                
                // Debug info 1: Check if we can find ANY links
                let all_links = debug_html.select("a");
                let link_count = if let Some(links) = all_links {
                    links.count()
                } else {
                    0
                };
                
                debug_chapters.push(Chapter {
                    key: String::from("/debug/1"),
                    title: Some(format!("DEBUG: Found {} total links", link_count)),
                    chapter_number: Some(1.0),
                    volume_number: None,
                    date_uploaded: None,
                    scanlators: None,
                    language: Some(String::from("fr")),
                    locked: false,
                    thumbnail: None,
                    url: Some(format!("{}/debug/1", BASE_URL)),
                });
                
                // Debug info 2: Check manga-specific links
                let manga_links = debug_html.select(&format!("a[href*=\"/manga/{}/\"]", manga.key));
                let manga_link_count = if let Some(links) = manga_links {
                    links.count()
                } else {
                    0
                };
                
                debug_chapters.push(Chapter {
                    key: String::from("/debug/2"),
                    title: Some(format!("DEBUG: Found {} manga-specific links", manga_link_count)),
                    chapter_number: Some(2.0),
                    volume_number: None,
                    date_uploaded: None,
                    scanlators: None,
                    language: Some(String::from("fr")),
                    locked: false,
                    thumbnail: None,
                    url: Some(format!("{}/debug/2", BASE_URL)),
                });
                
                // Debug info 3: Check for "Chapitre" text
                let chapitre_links = debug_html.select("a");
                let mut chapitre_count = 0;
                if let Some(links) = chapitre_links {
                    for link in links {
                        if let Some(text) = link.text() {
                            if text.contains("Chapitre") {
                                chapitre_count += 1;
                            }
                        }
                    }
                }
                
                debug_chapters.push(Chapter {
                    key: String::from("/debug/3"),
                    title: Some(format!("DEBUG: Found {} links with 'Chapitre'", chapitre_count)),
                    chapter_number: Some(3.0),
                    volume_number: None,
                    date_uploaded: None,
                    scanlators: None,
                    language: Some(String::from("fr")),
                    locked: false,
                    thumbnail: None,
                    url: Some(format!("{}/debug/3", BASE_URL)),
                });
                
                // Debug info 4: Show manga key being searched
                debug_chapters.push(Chapter {
                    key: String::from("/debug/4"),
                    title: Some(format!("DEBUG: Searching for key '{}'", manga.key)),
                    chapter_number: Some(4.0),
                    volume_number: None,
                    date_uploaded: None,
                    scanlators: None,
                    language: Some(String::from("fr")),
                    locked: false,
                    thumbnail: None,
                    url: Some(format!("{}/debug/4", BASE_URL)),
                });
                
                manga.chapters = Some(debug_chapters);
            } else {
                manga.chapters = Some(chapters_result);
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
    fn get_manga_list(&self, _listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = format!("{}/manga?page={}", BASE_URL, page);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .html()?;
        
        parser::parse_manga_list(html)
    }
}

impl ImageRequestProvider for LelscanFr {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

register_source!(LelscanFr, ListingProvider, ImageRequestProvider);