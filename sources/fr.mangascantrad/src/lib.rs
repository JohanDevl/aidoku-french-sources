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
        let mut url = format!("{}/page/{}/", BASE_URL, page);
        
        if let Some(search_query) = query {
            url = format!("{}/?s={}", BASE_URL, search_query);
        }

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_list(html)
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
        let url = match listing.id.as_str() {
            "populaire" => format!("{}/manga/page/{}/", BASE_URL, page),
            "tendance" => format!("{}/tendance/page/{}/", BASE_URL, page),
            _ => format!("{}/page/{}/", BASE_URL, page),
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_list(html)
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
    fn parse_manga_list(&self, html: Document) -> Result<MangaPageResult> {
        let mut entries: Vec<Manga> = Vec::new();

        // Madara theme selectors
        if let Some(items) = html.select(".page-item-detail, .c-tabs-item__content, .manga-item") {
            for item in items {
                if let Some(title_elements) = item.select("h3.h5 a, .post-title h1, .manga-title-badges") {
                    if let Some(title_elem) = title_elements.first() {
                        if let Some(links) = item.select("a") {
                            if let Some(link) = links.first() {
                                let title = title_elem.text().unwrap_or_default().trim().to_string();
                                let href = link.attr("href").unwrap_or_default();
                                
                                if !title.is_empty() && !href.is_empty() {
                                    let key = self.extract_manga_id(&href);
                                    let cover = item.select("img")
                                        .and_then(|imgs| imgs.first())
                                        .and_then(|img| {
                                            img.attr("data-src")
                                                .or_else(|| img.attr("src"))
                                        })
                                        .unwrap_or_default();

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
                            }
                        }
                    }
                }
            }
        }

        let has_next_page = entries.len() >= 20;
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