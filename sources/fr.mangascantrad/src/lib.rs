#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
    MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString};

pub static BASE_URL: &str = "https://manga-scantrad.io";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/605.1.15";

pub struct MangaScantrad;

impl Source for MangaScantrad {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        println!("get_search_manga_list called - page: {}, query: {:?}", page, query);
        
        if let Some(search_query) = query {
            // Use AJAX for search
            self.ajax_search(&search_query, page)
        } else {
            // Use AJAX for manga list
            self.ajax_manga_list(page)
        }
    }

    fn get_manga_update(&self, manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let url = format!("{}/manga/{}/", BASE_URL, manga.key);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_details(html, manga.key, needs_details, needs_chapters)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/{}/", BASE_URL, chapter.key);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_page_list(html)
    }
}

impl ListingProvider for MangaScantrad {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        println!("get_manga_list called - listing: {}, page: {}", listing.id, page);
        
        // Use AJAX for all listings
        self.ajax_manga_list(page)
    }
}

impl ImageRequestProvider for MangaScantrad {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl MangaScantrad {
    fn ajax_manga_list(&self, page: i32) -> Result<MangaPageResult> {
        println!("ajax_manga_list called for page {}", page);
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Madara AJAX payload
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail",
            page - 1, // Madara uses 0-based indexing
            page
        );
        
        println!("AJAX request body: {}", body);
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        println!("AJAX response received");
        
        self.parse_ajax_response(html_doc)
    }
    
    fn ajax_search(&self, query: &str, page: i32) -> Result<MangaPageResult> {
        println!("ajax_search called for query: {}, page: {}", query, page);
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Madara AJAX search payload
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-search&vars%5Bs%5D={}&vars%5Borderby%5D=&vars%5Bpaged%5D={}&vars%5Btemplate%5D=search&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Bmeta_query%5D%5B0%5D%5Brelation%5D=AND",
            page - 1,
            query,
            page
        );
        
        println!("AJAX search body: {}", body);
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        println!("AJAX search response received");
        
        self.parse_ajax_response(html_doc)
    }
    
    fn parse_ajax_response(&self, html: Document) -> Result<MangaPageResult> {
        println!("parse_ajax_response called");
        let mut entries: Vec<Manga> = Vec::new();
        
        // Try multiple selectors for AJAX response
        let selectors = [
            ".page-item-detail",
            ".manga-item",
            ".row .c-tabs-item",
            ".col-12 .manga",
            ".manga-content",
            ".c-tabs-item__content"
        ];
        
        let mut found_items = false;
        for selector in &selectors {
            println!("Trying selector: {}", selector);
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    println!("Found {} items with selector: {}", items_vec.len(), selector);
                    found_items = true;
                    
                    for (idx, item) in items_vec.iter().enumerate() {
                        println!("Processing item {}", idx);
                        
                        // Find the link element
                        let link = if let Some(links) = item.select("a") {
                            if let Some(first_link) = links.first() {
                                first_link
                            } else {
                                println!("  No link found");
                                continue;
                            }
                        } else {
                            println!("  No link found");
                            continue;
                        };
                        
                        let href = link.attr("href").unwrap_or_default();
                        if href.is_empty() {
                            println!("  Empty href");
                            continue;
                        }
                        
                        // Extract title
                        let title = link.attr("title")
                            .or_else(|| {
                                item.select("h3 a, .h5 a, .post-title, .manga-title")
                                    .and_then(|elems| elems.first())
                                    .and_then(|elem| elem.text())
                            })
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        
                        if title.is_empty() {
                            println!("  Empty title");
                            continue;
                        }
                        
                        println!("  Title: {}, URL: {}", title, href);
                        
                        let key = self.extract_manga_id(&href);
                        
                        // Extract cover image
                        let cover = item.select("img")
                            .and_then(|imgs| imgs.first())
                            .and_then(|img| {
                                img.attr("data-src")
                                    .or_else(|| img.attr("src"))
                                    .or_else(|| img.attr("data-lazy-src"))
                            })
                            .unwrap_or_default();
                        
                        println!("  Key: {}, Cover: {}", key, cover);
                        
                        entries.push(Manga {
                            key,
                            title,
                            cover: if cover.is_empty() { None } else { Some(cover) },
                            authors: None,
                            artists: None,
                            description: None,
                            url: Some(href),
                            tags: None,
                            status: MangaStatus::Unknown,
                            content_rating: ContentRating::Safe,
                            viewer: Viewer::RightToLeft,
                            chapters: None,
                            next_update_time: None,
                            update_strategy: UpdateStrategy::Never,
                        });
                    }
                    break; // Stop after finding items with one selector
                }
            }
        }
        
        if !found_items {
            println!("No items found with any selector!");
            // Try to print the HTML for debugging
            if let Some(body) = html.select("body") {
                if let Some(first) = body.first() {
                    let html_text = first.text().unwrap_or_default();
                    println!("Response body text (first 500 chars): {}", &html_text[..html_text.len().min(500)]);
                }
            }
        }
        
        let has_next_page = entries.len() >= 20;
        println!("Total entries parsed: {}, has_next_page: {}", entries.len(), has_next_page);
        
        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }
    
    fn parse_manga_details(&self, html: Document, manga_key: String, _needs_details: bool, _needs_chapters: bool) -> Result<Manga> {
        let title = html.select(".post-title h1, .manga-title")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_default();

        let cover = html.select(".summary_image img")
            .and_then(|elems| elems.first())
            .and_then(|img| {
                img.attr("data-src")
                    .or_else(|| img.attr("src"))
            })
            .unwrap_or_default();

        let description = html.select(".description-summary .summary__content, .summary p")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_default();

        let author = html.select(".author-content a, .manga-authors")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_default();

        let mut tags: Vec<String> = Vec::new();
        if let Some(genre_items) = html.select(".genres-content a, .manga-genres a") {
            for genre in genre_items {
                if let Some(text) = genre.text() {
                    tags.push(text.trim().to_string());
                }
            }
        }

        let status = html.select(".post-status .summary-content, .manga-status")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| {
                let status_text = text.to_lowercase();
                match status_text.as_str() {
                    "en cours" | "ongoing" => MangaStatus::Ongoing,
                    "terminé" | "completed" => MangaStatus::Completed,
                    "annulé" | "cancelled" => MangaStatus::Cancelled,
                    "en pause" | "on hold" => MangaStatus::Hiatus,
                    _ => MangaStatus::Unknown,
                }
            })
            .unwrap_or(MangaStatus::Unknown);

        let authors = if !author.is_empty() {
            Some(vec![author])
        } else {
            None
        };

        Ok(Manga {
            key: manga_key.clone(),
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            authors,
            artists: None,
            description: if description.is_empty() { None } else { Some(description) },
            url: Some(format!("{}/manga/{}/", BASE_URL, manga_key)),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status,
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        })
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        // Try multiple selectors for Madara themes
        if let Some(images) = html.select(".page-break img, .reading-content img, div.page-break > img") {
            for img in images {
                let img_url = img.attr("data-src")
                    .or_else(|| img.attr("src"))
                    .unwrap_or_default();

                if !img_url.is_empty() {
                    pages.push(Page {
                        content: PageContent::Url(img_url, None),
                        thumbnail: None,
                        has_description: false,
                        description: None,
                    });
                }
            }
        }

        Ok(pages)
    }

    fn extract_manga_id(&self, url: &str) -> String {
        url.trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string()
    }
}

register_source!(MangaScantrad, ListingProvider, ImageRequestProvider);