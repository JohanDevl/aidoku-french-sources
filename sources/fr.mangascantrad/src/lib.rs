#![no_std]

use aidoku::{
    Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
    MangaStatus, Page, PageContent, Result, Source,
    alloc::{String, Vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString, vec};

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

    fn get_manga_update(&self, manga_id: String) -> Result<Manga> {
        let url = format!("{}/manga/{}/", BASE_URL, manga_id);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_details(html, manga_id)
    }

    fn get_page_list(&self, _manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
        let url = format!("{}/{}/", BASE_URL, chapter_id);
        
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
    fn get_image_request(&self, url: String) -> Result<Request> {
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
                if let (Some(title_elem), Some(link)) = (
                    item.select("h3.h5 a, .post-title h1, .manga-title-badges").first(),
                    item.select("a").first(),
                ) {
                    let title = title_elem.text().read().trim().to_string();
                    let href = link.attr("href").read();
                    
                    if !title.is_empty() && !href.is_empty() {
                        let id = self.extract_manga_id(&href);
                        let cover = item.select("img").first()
                            .map(|img| {
                                img.attr("data-src").read()
                                    .or_else(|| img.attr("src").read())
                                    .unwrap_or_default()
                            })
                            .unwrap_or_default();

                        entries.push(Manga {
                            id,
                            title,
                            cover: if cover.is_empty() { None } else { Some(cover) },
                            author: None,
                            artist: None,
                            description: None,
                            url: Some(href),
                            categories: Vec::new(),
                            status: MangaStatus::Unknown,
                            nsfw: ContentRating::Safe,
                            viewer: ViewerType::Rtl,
                        });
                    }
                }
            }
        }

        Ok(MangaPageResult {
            entries,
            has_next_page: entries.len() >= 20,
        })
    }

    fn parse_manga_details(&self, html: Document, manga_id: String) -> Result<Manga> {
        let title = html.select(".post-title h1, .manga-title")
            .first()
            .map(|elem| elem.text().read().trim().to_string())
            .unwrap_or_default();

        let cover = html.select(".summary_image img")
            .first()
            .map(|img| {
                img.attr("data-src").read()
                    .or_else(|| img.attr("src").read())
                    .unwrap_or_default()
            })
            .unwrap_or_default();

        let description = html.select(".description-summary .summary__content, .summary p")
            .first()
            .map(|elem| elem.text().read().trim().to_string())
            .unwrap_or_default();

        let author = html.select(".author-content a, .manga-authors")
            .first()
            .map(|elem| elem.text().read().trim().to_string())
            .unwrap_or_default();

        let mut categories: Vec<String> = Vec::new();
        if let Some(genre_items) = html.select(".genres-content a, .manga-genres a") {
            for genre in genre_items {
                categories.push(genre.text().read().trim().to_string());
            }
        }

        let status = html.select(".post-status .summary-content, .manga-status")
            .first()
            .map(|elem| {
                let status_text = elem.text().read().to_lowercase();
                match status_text.as_str() {
                    "en cours" | "ongoing" => MangaStatus::Ongoing,
                    "terminé" | "completed" => MangaStatus::Completed,
                    "annulé" | "cancelled" => MangaStatus::Cancelled,
                    "en pause" | "on hold" => MangaStatus::Hiatus,
                    _ => MangaStatus::Unknown,
                }
            })
            .unwrap_or(MangaStatus::Unknown);

        Ok(Manga {
            id: manga_id,
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            author: if author.is_empty() { None } else { Some(author) },
            artist: None,
            description: if description.is_empty() { None } else { Some(description) },
            url: Some(format!("{}/manga/{}/", BASE_URL, manga_id)),
            categories,
            status,
            nsfw: ContentRating::Safe,
            viewer: ViewerType::Rtl,
        })
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        // Try multiple selectors for Madara themes
        if let Some(images) = html.select(".page-break img, .reading-content img, div.page-break > img") {
            for (index, img) in images.enumerate() {
                let img_url = img.attr("data-src").read()
                    .or_else(|| img.attr("src").read())
                    .unwrap_or_default();

                if !img_url.is_empty() {
                    pages.push(Page {
                        index: index as i32,
                        url: img_url,
                        base64: None,
                        text: None,
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