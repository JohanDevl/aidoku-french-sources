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
        
        match listing.id.as_str() {
            "populaire" => self.ajax_manga_listing("popular", page),
            "tendance" => self.ajax_manga_listing("trending", page),
            _ => self.ajax_manga_list(page),
        }
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
        
        // Madara AJAX payload for general listing with more results per page
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
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
    
    fn ajax_manga_listing(&self, listing_type: &str, page: i32) -> Result<MangaPageResult> {
        println!("ajax_manga_listing called for type: {}, page: {}", listing_type, page);
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Different payloads for different listing types
        let body = match listing_type {
            "popular" => {
                println!("Using popular/populaire AJAX payload");
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=meta_value_num&vars%5Bmeta_key%5D=_wp_manga_views&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=DESC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            },
            "trending" => {
                println!("Using trending/tendance AJAX payload");
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=trending&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=DESC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            },
            _ => {
                println!("Using default AJAX payload");
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            }
        };
        
        println!("AJAX {} request body: {}", listing_type, body);
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        println!("AJAX {} response received", listing_type);
        
        self.parse_ajax_response(html_doc)
    }
    
    fn ajax_search(&self, query: &str, page: i32) -> Result<MangaPageResult> {
        println!("ajax_search called for query: {}, page: {}", query, page);
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Madara AJAX search payload with more results per page
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-search&vars%5Bs%5D={}&vars%5Borderby%5D=&vars%5Bpaged%5D={}&vars%5Btemplate%5D=search&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Bmeta_query%5D%5B0%5D%5Brelation%5D=AND&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
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
    
    fn ajax_chapter_list(&self, manga_key: &str) -> Result<Vec<Chapter>> {
        println!("ajax_chapter_list called for manga: {}", manga_key);
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Madara AJAX payload for chapter list
        let body = format!(
            "action=manga_get_chapters&manga={}&nonce=1",
            manga_key
        );
        
        println!("AJAX chapter request body: {}", body);
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", &format!("{}/manga/{}/", BASE_URL, manga_key))
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        println!("AJAX chapter response received");
        
        self.parse_ajax_chapter_response(html_doc)
    }
    
    fn parse_ajax_chapter_response(&self, html: Document) -> Result<Vec<Chapter>> {
        println!("parse_ajax_chapter_response called");
        let mut chapters: Vec<Chapter> = Vec::new();

        // Try selectors specifically for AJAX chapter responses
        let chapter_selectors = [
            "li.wp-manga-chapter",
            ".wp-manga-chapter",
            "li a",
            ".version-chap li",
            ".listing-chapters_wrap li",
            ".manga-chapters li",
            "li.chapter-item",
            ".chapters li"
        ];
        
        let mut found_chapters = false;
        for selector in &chapter_selectors {
            println!("Trying AJAX chapter selector: {}", selector);
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    println!("Found {} chapter items with AJAX selector: {}", items_vec.len(), selector);
                    found_chapters = true;
                    
                    for (idx, item) in items_vec.iter().enumerate() {
                        // Find the link element
                        let link = if let Some(links) = item.select("a") {
                            if let Some(first_link) = links.first() {
                                first_link
                            } else {
                                println!("  AJAX Chapter {}: No link found", idx);
                                continue;
                            }
                        } else {
                            println!("  AJAX Chapter {}: No link found", idx);
                            continue;
                        };

                        let href = link.attr("href").unwrap_or_default();
                        if href.is_empty() {
                            println!("  AJAX Chapter {}: Empty href", idx);
                            continue;
                        }

                        // Extract chapter title
                        let title = link.text()
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        
                        if title.is_empty() {
                            println!("  AJAX Chapter {}: Empty title", idx);
                            continue;
                        }

                        // Extract chapter ID from URL
                        let chapter_key = href
                            .replace(BASE_URL, "")
                            .trim_start_matches('/')
                            .trim_end_matches('/')
                            .to_string();

                        // Extract chapter number from title or URL
                        let chapter_number = self.extract_chapter_number(&chapter_key, &title);

                        // Ensure URL is absolute
                        let url = if href.starts_with("http") {
                            href
                        } else if href.starts_with("/") {
                            format!("{}{}", BASE_URL, href)
                        } else {
                            format!("{}/{}", BASE_URL, href)
                        };

                        println!("  AJAX Chapter {}: title='{}', number={}, url={}", idx, title, chapter_number, url);

                        chapters.push(Chapter {
                            key: chapter_key,
                            title: Some(title),
                            chapter_number: Some(chapter_number),
                            volume_number: None,
                            date_uploaded: None,
                            scanlators: None,
                            url: Some(url),
                            language: Some(String::from("fr")),
                            thumbnail: None,
                            locked: false,
                        });
                    }
                    break; // Stop after finding chapters with one selector
                }
            }
        }
        
        if !found_chapters {
            println!("No chapters found with any AJAX selector!");
            // Try to print the response for debugging
            if let Some(body) = html.select("body") {
                if let Some(first) = body.first() {
                    let response_text = first.text().unwrap_or_default();
                    println!("AJAX response body text (first 300 chars): {}", &response_text[..response_text.len().min(300)]);
                }
            }
        }

        println!("Total AJAX chapters parsed: {}", chapters.len());
        Ok(chapters)
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
        
        // Pagination logic: if we got any results, assume there might be more
        // Madara typically returns 10-12 items per page, so we check if we got a reasonable amount
        let has_next_page = entries.len() >= 8; // Conservative threshold
        println!("Total entries parsed: {}, has_next_page: {} (threshold >= 8)", entries.len(), has_next_page);
        
        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }
    
    fn parse_manga_details(&self, html: Document, manga_key: String, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        println!("parse_manga_details called - key: {}, needs_details: {}, needs_chapters: {}", manga_key, needs_details, needs_chapters);
        
        // Extract title with multiple selectors
        let title = html.select(".post-title h1, .manga-title, h1.entry-title, .wp-manga-title, .single-title")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| {
                println!("No title found with selectors");
                manga_key.clone()
            });
        
        println!("Found title: {}", title);

        // Extract cover with extensive selectors
        let cover_selectors = [
            ".summary_image img",
            ".wp-post-image",
            ".manga-poster img",
            ".post-thumb img",
            ".series-thumb img",
            ".thumb img",
            "img.attachment-post-thumbnail",
            ".infomanga img",
            "div[itemprop=image] img",
            ".post-content img:first-child"
        ];
        
        let mut cover = String::new();
        for selector in &cover_selectors {
            println!("Trying cover selector: {}", selector);
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(src) = img_elem.attr("data-lazy-src")
                    .or_else(|| img_elem.attr("data-src"))
                    .or_else(|| img_elem.attr("src")) {
                    if !src.is_empty() {
                        cover = src;
                        println!("Found cover with {}: {}", selector, cover);
                        break;
                    }
                }
            }
        }
        
        if cover.is_empty() {
            println!("No cover found with any selector");
        }

        // Extract description with multiple selectors
        let description_selectors = [
            ".description-summary .summary__content",
            ".summary p",
            ".desc",
            ".entry-content[itemprop=description]",
            ".manga-summary",
            ".post-content_item .summary-content",
            ".description",
            ".synopsis",
            ".post-excerpt"
        ];
        
        let mut description = String::new();
        for selector in &description_selectors {
            if let Some(desc_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(desc_text) = desc_elem.text() {
                    if !desc_text.trim().is_empty() {
                        description = desc_text.trim().to_string();
                        println!("Found description with {}: {} chars", selector, description.len());
                        break;
                    }
                }
            }
        }

        // Extract author
        let author = html.select(".author-content a, .manga-authors, .imptdt:contains(Auteur) i, .fmed b:contains(Author) + span")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_default();
            
        println!("Found author: {}", if author.is_empty() { "none" } else { &author });

        // Extract tags/genres
        let mut tags: Vec<String> = Vec::new();
        let genre_selectors = [
            ".genres-content a",
            ".manga-genres a", 
            ".gnr a",
            ".mgen a",
            ".seriestugenre a"
        ];
        
        for selector in &genre_selectors {
            if let Some(genre_items) = html.select(selector) {
                for genre in genre_items {
                    if let Some(text) = genre.text() {
                        let genre_text = text.trim().to_string();
                        if !genre_text.is_empty() && !tags.contains(&genre_text) {
                            tags.push(genre_text);
                        }
                    }
                }
                if !tags.is_empty() {
                    println!("Found {} tags with {}", tags.len(), selector);
                    break;
                }
            }
        }

        // Extract status
        let status_selectors = [
            ".post-status .summary-content",
            ".manga-status",
            ".imptdt:contains(Statut) i",
            ".status",
            ".series-status",
            ".tsinfo .imptdt:contains(Status) i"
        ];
        
        let mut status = MangaStatus::Unknown;
        for selector in &status_selectors {
            if let Some(status_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(status_text) = status_elem.text() {
                    let status_str = status_text.trim().to_lowercase();
                    status = match status_str.as_str() {
                        "en cours" | "ongoing" | "en_cours" | "en-cours" => MangaStatus::Ongoing,
                        "terminé" | "completed" | "termine" | "fini" | "achevé" => MangaStatus::Completed,
                        "annulé" | "cancelled" | "annule" | "canceled" => MangaStatus::Cancelled,
                        "en pause" | "hiatus" | "pause" | "en_pause" | "en-pause" | "on hold" => MangaStatus::Hiatus,
                        _ => MangaStatus::Unknown,
                    };
                    if status != MangaStatus::Unknown {
                        println!("Found status with {}: {:?}", selector, status);
                        break;
                    }
                }
            }
        }

        let authors = if !author.is_empty() {
            Some(vec![author])
        } else {
            None
        };
        
        // Parse chapters if requested
        let chapters = if needs_chapters {
            println!("Fetching chapters via AJAX...");
            match self.ajax_chapter_list(&manga_key) {
                Ok(chapter_list) => {
                    println!("Found {} chapters via AJAX", chapter_list.len());
                    Some(chapter_list)
                }
                Err(e) => {
                    println!("Error fetching chapters via AJAX: {:?}", e);
                    // Fallback to HTML parsing
                    println!("Falling back to HTML parsing...");
                    match self.parse_chapter_list(&html) {
                        Ok(chapter_list) => {
                            println!("Found {} chapters via HTML", chapter_list.len());
                            Some(chapter_list)
                        }
                        Err(e2) => {
                            println!("Error parsing chapters from HTML: {:?}", e2);
                            None
                        }
                    }
                }
            }
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
            chapters,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        })
    }
    
    fn parse_chapter_list(&self, html: &Document) -> Result<Vec<Chapter>> {
        println!("parse_chapter_list called");
        let mut chapters: Vec<Chapter> = Vec::new();

        // Madara/WordPress chapter selectors
        let chapter_selectors = [
            "li.wp-manga-chapter",
            ".wp-manga-chapter",
            ".manga-chapters li",
            ".chapter-list li",
            "#chapterlist li",
            "div.bxcl li",
            "div.cl li",
            ".listing-chapters_wrap li",
            ".main .eph-num",
            ".chbox"
        ];
        
        let mut found_chapters = false;
        for selector in &chapter_selectors {
            println!("Trying chapter selector: {}", selector);
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    println!("Found {} chapter items with {}", items_vec.len(), selector);
                    found_chapters = true;
                    
                    for (idx, item) in items_vec.iter().enumerate() {
                        // Find the link element
                        let link = if let Some(links) = item.select("a") {
                            if let Some(first_link) = links.first() {
                                first_link
                            } else {
                                println!("  Chapter {}: No link found", idx);
                                continue;
                            }
                        } else {
                            println!("  Chapter {}: No link found", idx);
                            continue;
                        };

                        let href = link.attr("href").unwrap_or_default();
                        if href.is_empty() {
                            println!("  Chapter {}: Empty href", idx);
                            continue;
                        }

                        // Extract chapter title
                        let title = link.text()
                            .or_else(|| {
                                item.select(".chapternum, .lch a, .chapter-manhwa-title")
                                    .and_then(|elems| elems.first())
                                    .and_then(|elem| elem.text())
                            })
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        
                        if title.is_empty() {
                            println!("  Chapter {}: Empty title", idx);
                            continue;
                        }

                        // Extract chapter ID from URL
                        let chapter_key = href
                            .replace(BASE_URL, "")
                            .trim_start_matches('/')
                            .trim_end_matches('/')
                            .to_string();

                        // Extract chapter number from title or URL
                        let chapter_number = self.extract_chapter_number(&chapter_key, &title);
                        
                        // Extract date if available
                        let date_uploaded = item.select(".chapterdate, .chapter-release-date, .dt")
                            .and_then(|elems| elems.first())
                            .and_then(|elem| elem.text())
                            .and_then(|date_str| self.parse_chapter_date(&date_str));

                        // Ensure URL is absolute
                        let url = if href.starts_with("http") {
                            href
                        } else if href.starts_with("/") {
                            format!("{}{}", BASE_URL, href)
                        } else {
                            format!("{}/{}", BASE_URL, href)
                        };

                        println!("  Chapter {}: title='{}', number={}, url={}", idx, title, chapter_number, url);

                        chapters.push(Chapter {
                            key: chapter_key,
                            title: Some(title),
                            chapter_number: Some(chapter_number),
                            volume_number: None,
                            date_uploaded,
                            scanlators: None,
                            url: Some(url),
                            language: Some(String::from("fr")),
                            thumbnail: None,
                            locked: false,
                        });
                    }
                    break; // Stop after finding chapters with one selector
                }
            }
        }
        
        if !found_chapters {
            println!("No chapters found with any selector!");
        }

        println!("Total chapters parsed: {}", chapters.len());
        Ok(chapters)
    }
    
    fn extract_chapter_number(&self, chapter_id: &str, title: &str) -> f32 {
        // Try to extract from title first - check for "chapitre" or "ch"
        let title_lower = title.to_lowercase();
        if let Some(pos) = title_lower.find("chapitre") {
            let after_ch = &title[pos + 8..].trim(); // "chapitre" has 8 chars
            if let Some(num_str) = after_ch.split_whitespace().next() {
                if let Ok(num) = num_str.replace(',', ".").parse::<f32>() {
                    return num;
                }
            }
        } else if let Some(pos) = title_lower.find("ch") {
            let after_ch = &title[pos + 2..].trim(); // "ch" has 2 chars
            if let Some(num_str) = after_ch.split_whitespace().next() {
                if let Ok(num) = num_str.replace(',', ".").parse::<f32>() {
                    return num;
                }
            }
        }
        
        // Try to extract from URL
        let parts: Vec<&str> = chapter_id.split('/').filter(|s| !s.is_empty()).collect();
        for part in parts.iter().rev() {
            if let Ok(num) = part.parse::<f32>() {
                return num;
            }
            // Try to extract number from part like "chapitre-123"
            if let Some(dash_pos) = part.rfind('-') {
                let after_dash = &part[dash_pos + 1..];
                if let Ok(num) = after_dash.parse::<f32>() {
                    return num;
                }
            }
        }
        
        1.0 // Default
    }
    
    fn parse_chapter_date(&self, _date_str: &str) -> Option<i64> {
        // Date parsing disabled for now due to no_std limitations
        // TODO: Implement using Aidoku's date utilities if available
        None
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