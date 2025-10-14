#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
    MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer, AidokuError,
    alloc::{String, Vec, vec},
    imports::{net::Request, html::Document, std::send_partial_result},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString};

pub static BASE_URL: &str = "https://mangas-origines.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604";

const MAX_RETRIES: u32 = 3;

pub struct MangasOrigines;

fn calculate_content_rating(tags: &Option<Vec<String>>) -> ContentRating {
    if let Some(tags) = tags {
        for tag in tags {
            let tag_lower = tag.to_lowercase();
            match tag_lower.as_str() {
                "adult" | "adulte" | "mature" | "hentai" | "smut" | "érotique" => {
                    return ContentRating::NSFW;
                }
                "ecchi" | "suggestif" | "suggestive" => {
                    return ContentRating::Suggestive;
                }
                _ => {}
            }
        }
    }
    ContentRating::Safe
}

fn calculate_viewer(tags: &Option<Vec<String>>) -> Viewer {
    if let Some(tags) = tags {
        for tag in tags {
            let tag_lower = tag.to_lowercase();
            match tag_lower.as_str() {
                "manhwa" | "manhua" | "webtoon" | "scroll" | "vertical" => {
                    return Viewer::Vertical;
                }
                "manga" => {
                    return Viewer::RightToLeft;
                }
                _ => {}
            }
        }
    }
    Viewer::RightToLeft
}

impl Source for MangasOrigines {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        // Process filters to extract search parameters
        let mut genre_filters = Vec::new();
        let mut genre_op = String::from(""); // Default to OR
        
        for filter in filters.iter() {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "op" {
                        // Set genre condition (AND/OR)
                        genre_op = if value == "AND" { "1".to_string() } else { "".to_string() };
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" && !included.is_empty() {
                        for genre in included {
                            if !genre.is_empty() {
                                genre_filters.push(genre.clone());
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Use filtered search if filters are applied or query is present
        if query.is_some() || !genre_filters.is_empty() {
            self.ajax_filtered_search(query, page, genre_filters, &genre_op)
        } else {
            self.get_manga_listing_page(page)
        }
    }

    fn get_manga_update(&self, manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        println!("[mangasorigines] get_manga_update START - manga_id: {}, needs_details: {}, needs_chapters: {}",
            manga.key, needs_details, needs_chapters);

        let url = format!("{}/oeuvre/{}/", BASE_URL, manga.key);

        let headers = vec![
            ("User-Agent", USER_AGENT),
            ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"),
            ("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8"),
            ("Referer", BASE_URL),
        ];

        let html = Self::request_with_retry(&url, headers)?;

        let mut result_manga = self.parse_manga_details(&html, manga.key.clone(), false)?;

        if needs_details {
            println!("[mangasorigines] Metadata fetched successfully - title: {}", result_manga.title);
            send_partial_result(&result_manga);
        }

        if needs_chapters {
            let ajax_chapters = self.ajax_chapter_list(&manga.key).unwrap_or_else(|_| vec![]);

            if !ajax_chapters.is_empty() {
                result_manga.chapters = Some(ajax_chapters);
            } else {
                result_manga.chapters = Some(self.parse_chapter_list(&manga.key, &html).unwrap_or_else(|_| vec![]));
            }

            if let Some(ref chapters) = result_manga.chapters {
                println!("[mangasorigines] Chapters fetched successfully - count: {}", chapters.len());
            }
        }

        println!("[mangasorigines] get_manga_update COMPLETE");
        Ok(result_manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}?style=list", chapter.url.unwrap_or_else(|| format!("{}/{}/", BASE_URL, chapter.key)));

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_page_list(&html)
    }
}

impl ListingProvider for MangasOrigines {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        match listing.id.as_str() {
            "populaire" => self.get_manga_listing("popular", page),
            "tendance" => self.get_manga_listing("trending", page),
            _ => self.get_manga_listing_page(page),
        }
    }
}

impl ImageRequestProvider for MangasOrigines {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl MangasOrigines {

    fn request_with_retry(url: &str, headers: Vec<(&str, &str)>) -> Result<Document> {
        let mut attempt = 0;
        loop {
            let mut request = Request::get(url)?;
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            let response = match request.send() {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::RequestError(e));
                    }
                    attempt += 1;
                    continue;
                }
            };

            match response.status_code() {
                200..=299 => {
                    return response.get_html().map_err(|e| AidokuError::RequestError(e));
                }
                408 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Request timeout"));
                    }
                    attempt += 1;
                    continue;
                }
                502 | 503 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                }
                403 | 429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Access blocked or rate limited"));
                    }
                    attempt += 1;
                    continue;
                }
                _ => {
                    return Err(AidokuError::message("Request failed"));
                }
            }
        }
    }

    fn post_request_with_retry(url: &str, headers: Vec<(&str, &str)>, body: &[u8]) -> Result<Document> {
        let mut attempt = 0;
        loop {
            let mut request = Request::post(url)?;
            for (key, value) in &headers {
                request = request.header(key, value);
            }

            let response = match request.body(body).send() {
                Ok(resp) => resp,
                Err(e) => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::RequestError(e));
                    }
                    attempt += 1;
                    continue;
                }
            };

            match response.status_code() {
                200..=299 => {
                    return response.get_html().map_err(|e| AidokuError::RequestError(e));
                }
                408 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Request timeout"));
                    }
                    attempt += 1;
                    continue;
                }
                502 | 503 | 504 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Server error"));
                    }
                    attempt += 1;
                    continue;
                }
                403 | 429 => {
                    if attempt >= MAX_RETRIES {
                        return Err(AidokuError::message("Access blocked or rate limited"));
                    }
                    attempt += 1;
                    continue;
                }
                _ => {
                    return Err(AidokuError::message("Request failed"));
                }
            }
        }
    }

    fn get_manga_listing_page(&self, page: i32) -> Result<MangaPageResult> {
        let url = format!("{}/oeuvre/?page={}", BASE_URL, page);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_list(html, page)
    }

    fn get_manga_listing(&self, list_type: &str, page: i32) -> Result<MangaPageResult> {
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Different payloads for different listing types (like mangascantrad)
        let body = match list_type {
            "popular" => {
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=meta_value_num&vars%5Bmeta_key%5D=_wp_manga_views&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=DESC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            },
            "trending" => {
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=trending&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=DESC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            },
            _ => return self.get_manga_listing_page(page)
        };
        
        let html = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;

        self.parse_manga_list(html, page)
    }

    fn ajax_filtered_search(
        &self,
        query: Option<String>,
        page: i32,
        genre_filters: Vec<String>,
        genre_op: &str,
    ) -> Result<MangaPageResult> {
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        let mut body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
            page - 1,
            page
        );
        
        // Add search query if present
        if let Some(search_query) = &query {
            if !search_query.is_empty() {
                body.push_str(&format!("&vars%5Bs%5D={}", self.url_encode(search_query)));
            }
        }
        
        // Add genre filters using tax_query format
        let mut tax_query_index = 0;
        for genre in &genre_filters {
            let genre_param = format!(
                "&vars%5Btax_query%5D%5B{}%5D%5Btaxonomy%5D=wp-manga-genre&vars%5Btax_query%5D%5B{}%5D%5Bfield%5D=slug&vars%5Btax_query%5D%5B{}%5D%5Bterms%5D={}",
                tax_query_index, tax_query_index, tax_query_index, self.url_encode(genre)
            );
            body.push_str(&genre_param);
            tax_query_index += 1;
        }
        
        // Add operator if multiple genre filters
        if tax_query_index > 1 {
            let operator = if genre_op == "1" { "AND" } else { "OR" };
            let relation_param = format!("&vars%5Btax_query%5D%5Brelation%5D={}", operator);
            body.push_str(&relation_param);
        }
        
        
        let html = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;

        self.parse_manga_list(html, page)
    }

    fn parse_manga_list(&self, html: Document, page: i32) -> Result<MangaPageResult> {
        let mut manga_list = Vec::new();
        
        if let Some(items) = html.select("div.page-item-detail, div.row.c-tabs-item__content, .manga-item") {
            for item in items {
                if let Some(link_elements) = item.select("h3 a, h5 a, h4 a, .post-title a, .manga-title a") {
                    if let Some(link) = link_elements.first() {
                        let title = link.text().unwrap_or_default().trim().to_string();
                        let url = link.attr("href").unwrap_or_default();
                        
                        if !title.is_empty() && !url.is_empty() {
                            let key = self.extract_manga_key(&url);
                            if !key.is_empty() {
                                let cover_selectors = [
                                    "div.item-thumb img",       // Main manga item thumb
                                    ".post-thumb img",          // Post thumbnail  
                                    ".manga-poster img",        // Manga poster
                                    ".wp-post-image",          // WordPress featured image
                                    ".item-summary img",        // Item summary image
                                    ".c-image-hover img",       // Image hover container
                                    ".tab-thumb img",           // Tab thumbnail
                                    "img",                      // Fallback generic img
                                ];
                                
                                let mut cover_url: Option<String> = None;
                                for selector in &cover_selectors {
                                    if let Some(img_elem) = item.select(selector).and_then(|elems| elems.first()) {
                                        // Use same attribute priority as other Madara sources
                                        if let Some(src) = img_elem.attr("data-src")
                                            .or_else(|| img_elem.attr("data-lazy-src"))
                                            .or_else(|| img_elem.attr("src"))
                                            .or_else(|| img_elem.attr("srcset"))
                                            .or_else(|| img_elem.attr("data-cfsrc")) {
                                            if !src.is_empty() {
                                                // Clean up srcset if needed (take first URL)
                                                let clean_src = if src.contains(" ") {
                                                    src.split_whitespace().next().unwrap_or("").to_string()
                                                } else {
                                                    src.to_string()
                                                };
                                                cover_url = Some(clean_src);
                                                break;
                                            }
                                        }
                                    }
                                }

                                manga_list.push(Manga {
                                    key: key.clone(),
                                    title,
                                    cover: cover_url,
                                    url: Some(format!("{}/oeuvre/{}/", BASE_URL, key)),
                                    status: MangaStatus::Unknown,
                                    content_rating: ContentRating::Safe,
                                    viewer: Viewer::RightToLeft,
                                    authors: None,
                                    artists: None,
                                    description: None,
                                    tags: None,
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

        Ok(MangaPageResult {
            entries: manga_list,
            has_next_page: page < 50,
        })
    }

    fn parse_manga_details(&self, html: &Document, key: String, needs_chapters: bool) -> Result<Manga> {
        let title = if let Some(title_elements) = html.select("div.post-title h1, .wp-manga-title, .manga-title") {
            title_elements.text().unwrap_or_default().trim().to_string()
        } else {
            String::new()
        };

        let cover = self.get_cover_url(&html);
        let author = self.get_manga_author(&html);
        let description = self.get_manga_description(&html);
        let status = self.get_manga_status(&html);
        let tags = self.get_manga_tags(&html);

        let content_rating = calculate_content_rating(&tags);
        let viewer = calculate_viewer(&tags);

        let mut manga = Manga {
            key: key.clone(),
            title,
            cover,
            url: Some(format!("{}/oeuvre/{}/", BASE_URL, key)),
            status,
            content_rating,
            viewer,
            authors: author,
            artists: None,
            description,
            tags,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        };

        if needs_chapters {
            let ajax_chapters = self.ajax_chapter_list(&key).unwrap_or_else(|_| vec![]);

            if !ajax_chapters.is_empty() {
                manga.chapters = Some(ajax_chapters);
            } else {
                manga.chapters = Some(self.parse_chapter_list(&key, &html).unwrap_or_else(|_| vec![]));
            }
        }

        Ok(manga)
    }

    fn parse_chapter_elements_from_html(&self, html: &Document) -> Vec<Chapter> {
        let mut chapters = Vec::new();

        if let Some(chapter_elements) = html.select("li.wp-manga-chapter, .wp-manga-chapter, .chapter-item") {
            for chapter_element in chapter_elements {
                if let Some(link_elements) = chapter_element.select("a") {
                    if let Some(link) = link_elements.first() {
                        let chapter_url = link.attr("href").unwrap_or_default();
                        let chapter_title = link.text().unwrap_or_default().trim().to_string();

                        if !chapter_url.is_empty() && !chapter_title.is_empty() {
                            let chapter_key = self.extract_chapter_key(&chapter_url);
                            if !chapter_key.is_empty() {
                                let chapter_number = self.extract_chapter_number(&chapter_title);
                                let date_published = {
                                    let date_selectors = [
                                        "span.chapter-release-date i",
                                        ".chapter-release-date",
                                        ".chapterdate",
                                        ".chapter-date",
                                        ".dt",
                                        "span.date",
                                        "time",
                                        "i",
                                        ".post-on",
                                        ".release-date",
                                        ".uploaded-on",
                                    ];

                                    let mut found_date = None;
                                    for selector in &date_selectors {
                                        if let Some(date_elem) = chapter_element.select(selector).and_then(|elems| elems.first()) {
                                            if let Some(date_text) = date_elem.text() {
                                                let date_str = date_text.trim();
                                                if !date_str.is_empty() {
                                                    if let Some(parsed_date) = self.parse_chapter_date(date_str) {
                                                        found_date = Some(parsed_date);
                                                        break;
                                                    }
                                                }
                                            }

                                            if let Some(title_attr) = date_elem.attr("title") {
                                                let title_str = title_attr.trim();
                                                if !title_str.is_empty() {
                                                    if let Some(parsed_date) = self.parse_chapter_date(title_str) {
                                                        found_date = Some(parsed_date);
                                                        break;
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    found_date
                                };

                                chapters.push(Chapter {
                                    key: chapter_key,
                                    title: Some(chapter_title),
                                    url: Some(chapter_url),
                                    language: Some("fr".to_string()),
                                    volume_number: None,
                                    chapter_number: Some(chapter_number),
                                    date_uploaded: date_published,
                                    scanlators: None,
                                    thumbnail: None,
                                    locked: false,
                                });
                            }
                        }
                    }
                }
            }
        }

        chapters
    }

    fn parse_chapter_list(&self, _manga_key: &str, html: &Document) -> Result<Vec<Chapter>> {
        Ok(self.parse_chapter_elements_from_html(html))
    }
    
    fn ajax_chapter_list(&self, manga_key: &str) -> Result<Vec<Chapter>> {
        let manga_url = format!("{}/oeuvre/{}/", BASE_URL, manga_key);

        let headers = vec![
            ("User-Agent", USER_AGENT),
            ("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8"),
            ("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8"),
            ("Referer", BASE_URL),
        ];

        let manga_page_doc = Self::request_with_retry(&manga_url, headers)?;

        let int_id = self.extract_manga_int_id(&manga_page_doc)?;

        let ajax_url = format!("{}/oeuvre/{}/ajax/chapters", BASE_URL, manga_key);
        let body_content = format!("action=manga_get_chapters&manga={}", int_id);

        let ajax_headers = vec![
            ("User-Agent", USER_AGENT),
            ("Content-Type", "application/x-www-form-urlencoded"),
            ("Accept", "*/*"),
            ("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8"),
            ("Referer", manga_url.as_str()),
            ("X-Requested-With", "XMLHttpRequest"),
        ];

        let ajax_doc = Self::post_request_with_retry(&ajax_url, ajax_headers, body_content.as_bytes())?;

        self.parse_ajax_chapters_response(ajax_doc)
    }
    
    fn extract_manga_int_id(&self, html: &Document) -> Result<String> {
        // Look for the wp-manga-js-extra script tag (like in mangascantrad)
        if let Some(script_element) = html.select("script#wp-manga-js-extra") {
            if let Some(script_content) = script_element.html() {
                let script_text = script_content;
                
                // Look for manga ID in the script
                if let Some(start) = script_text.find("\"manga_id\":\"") {
                    let after_start = &script_text[start + 12..];
                    if let Some(end) = after_start.find("\"") {
                        let manga_id = &after_start[..end];
                        return Ok(manga_id.to_string());
                    }
                }
                
                // Alternative pattern
                if let Some(start) = script_text.find("manga_id=") {
                    let after_start = &script_text[start + 9..];
                    if let Some(end) = after_start.find(|c: char| !c.is_ascii_digit()) {
                        let manga_id = &after_start[..end];
                        return Ok(manga_id.to_string());
                    }
                }
            }
        }
        
        if let Some(script_elements) = html.select("script") {
            for script in script_elements {
                if let Some(script_content) = script.html() {
                    if script_content.contains("manga_id") {
                        if let Some(start) = script_content.find("\"manga_id\":\"") {
                            let after_start = &script_content[start + 12..];
                            if let Some(end) = after_start.find("\"") {
                                let manga_id = &after_start[..end];
                                return Ok(manga_id.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok("0".to_string())
    }
    
    fn parse_ajax_chapters_response(&self, html: Document) -> Result<Vec<Chapter>> {
        Ok(self.parse_chapter_elements_from_html(&html))
    }

    fn parse_page_list(&self, html: &Document) -> Result<Vec<Page>> {
        let mut pages = Vec::new();
        
        // Multiple selectors for different page layouts
        let image_selectors = [
            "div.page-break > img",              // Madara default selector
            ".page-break img",                   // Alternative page break
            ".reading-content img",              // Reading content
            ".wp-manga-chapter-img",             // WordPress manga images
            "img.wp-manga-chapter-img",          // Specific manga chapter images
            ".chapter-content img",              // Chapter content images
            "div.text-left img",                 // Text content images
            "#chapter-content img",              // Chapter content by ID
            ".entry-content img",                // Entry content images
            "div.reading-content img",           // Reading content div
            ".wp-manga-chapter-img img",         // WordPress manga chapter images
            "div.page-break img",                // Page break div
            ".chapter-wrap img",                 // Chapter wrapper
            ".chapter img",                      // Chapter class
            ".pages img",                        // Pages container
        ];

        for selector in &image_selectors {
            if let Some(images) = html.select(selector) {
                for img_element in images {
                    let image_url = self.get_image_url(&img_element);
                    
                    if !image_url.is_empty() {
                        pages.push(Page {
                            content: PageContent::Url(image_url, None),
                            thumbnail: None,
                            has_description: false,
                            description: None,
                        });
                    }
                }
                
                // If we found images with this selector, no need to try others
                if !pages.is_empty() {
                    break;
                }
            }
        }

        Ok(pages)
    }
    
    fn get_image_url(&self, img_elem: &aidoku::imports::html::Element) -> String {
        // Try different attributes in same priority as Madara template
        let mut img_url = img_elem.attr("data-src").unwrap_or_default();
        if img_url.is_empty() {
            img_url = img_elem.attr("data-lazy-src").unwrap_or_default();
        }
        if img_url.is_empty() {
            img_url = img_elem.attr("src").unwrap_or_default();
        }
        if img_url.is_empty() {
            img_url = img_elem.attr("srcset").unwrap_or_default();
        }
        if img_url.is_empty() {
            img_url = img_elem.attr("data-cfsrc").unwrap_or_default();
        }
        
        let img_url = img_url.trim().to_string();
        
        // Clean up srcset if needed (take first URL)
        if img_url.contains(" ") {
            img_url.split_whitespace().next().unwrap_or("").to_string()
        } else {
            img_url
        }
    }

    fn extract_manga_key(&self, url: &str) -> String {
        if let Some(start) = url.find("/oeuvre/") {
            let key_part = &url[start + 8..];
            if let Some(end) = key_part.find('/') {
                key_part[..end].to_string()
            } else {
                key_part.to_string()
            }
        } else {
            String::new()
        }
    }

    fn extract_chapter_key(&self, url: &str) -> String {
        if let Some(start) = url.rfind('/') {
            let end_part = &url[..start];
            if let Some(start) = end_part.rfind('/') {
                end_part[start + 1..].to_string()
            } else {
                url.to_string()
            }
        } else {
            url.to_string()
        }
    }

    fn extract_chapter_number(&self, title: &str) -> f32 {
        let title_lower = title.to_lowercase();
        
        if let Some(ch_pos) = title_lower.find("chapitre") {
            let after_ch = &title_lower[ch_pos + 8..];
            self.extract_number_from_string(after_ch)
        } else if let Some(ch_pos) = title_lower.find("ch") {
            let after_ch = &title_lower[ch_pos + 2..];
            self.extract_number_from_string(after_ch)
        } else {
            self.extract_number_from_string(title)
        }
    }

    fn extract_number_from_string(&self, s: &str) -> f32 {
        let mut number_str = String::new();
        let mut has_dot = false;
        
        for ch in s.chars() {
            if ch.is_ascii_digit() {
                number_str.push(ch);
            } else if ch == '.' && !has_dot {
                number_str.push(ch);
                has_dot = true;
            } else if !number_str.is_empty() {
                break;
            }
        }
        
        number_str.trim().parse().unwrap_or(0.0)
    }

    fn get_cover_url(&self, html: &Document) -> Option<String> {
        let cover_selectors = [
            "div.summary_image img",      // Primary Madara selector
            ".wp-post-image",             // WordPress featured image
            ".manga-poster img",          // Common manga poster
            ".post-thumb img",            // Post thumbnail
            ".series-thumb img",          // Series thumbnail
            "div.tab-summary img",        // Tab summary image
            ".manga-summary img",         // Manga summary image
            "div.manga-detail img",       // Manga detail image
            "img.wp-post-image",          // WordPress post image
            "article img:first-child",    // First article image
        ];

        for selector in &cover_selectors {
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                // Use same attribute priority as other Madara sources
                if let Some(src) = img_elem.attr("data-src")
                    .or_else(|| img_elem.attr("data-lazy-src"))
                    .or_else(|| img_elem.attr("src"))
                    .or_else(|| img_elem.attr("srcset"))
                    .or_else(|| img_elem.attr("data-cfsrc")) {
                    if !src.is_empty() {
                        // Clean up srcset if needed (take first URL)
                        let clean_src = if src.contains(" ") {
                            src.split_whitespace().next().unwrap_or("").to_string()
                        } else {
                            src.to_string()
                        };
                        return Some(clean_src);
                    }
                }
            }
        }
        None
    }

    fn get_manga_author(&self, html: &Document) -> Option<Vec<String>> {
        let author_selectors = [
            "div.manga-authors a",
            "div.author-content a",
            "span.author a",
            "div.manga-authors",
            "div.author-content"
        ];

        for selector in author_selectors.iter() {
            if let Some(author_elements) = html.select(selector) {
                let author_text = author_elements.text().unwrap_or_default().trim().to_string();
                if !author_text.is_empty() {
                    return Some(vec![author_text]);
                }
            }
        }
        None
    }

    fn get_manga_description(&self, html: &Document) -> Option<String> {
        let description_selectors = [
            "div.summary__content p",
            "div.description-summary p",
            "div.manga-excerpt p",
            "div.summary__content",
            "div.description-summary"
        ];

        for selector in description_selectors.iter() {
            if let Some(desc_elements) = html.select(selector) {
                let desc_text = desc_elements.text().unwrap_or_default().trim().to_string();
                if !desc_text.is_empty() && desc_text.len() > 10 {
                    return Some(desc_text);
                }
            }
        }
        None
    }

    fn get_manga_status(&self, html: &Document) -> MangaStatus {
        let status_selectors = [
            "div.post-status div.summary-content",
            "div.manga-status",
            "span.manga-status",
            "div.post-content_item:contains('Statut')",
            "div.summary-heading:contains('Statut') + div.summary-content"
        ];

        for selector in status_selectors.iter() {
            if let Some(status_elements) = html.select(selector) {
                let status_text = status_elements.text().unwrap_or_default().to_lowercase();
                
                if status_text.contains("en cours") || status_text.contains("ongoing") || status_text.contains("publication") {
                    return MangaStatus::Ongoing;
                } else if status_text.contains("terminé") || status_text.contains("completed") || status_text.contains("fini") || status_text.contains("complet") {
                    return MangaStatus::Completed;
                } else if status_text.contains("annulé") || status_text.contains("cancelled") || status_text.contains("canceled") {
                    return MangaStatus::Cancelled;
                } else if status_text.contains("en pause") || status_text.contains("hiatus") || status_text.contains("pause") {
                    return MangaStatus::Hiatus;
                }
            }
        }
        
        MangaStatus::Unknown
    }

    fn get_manga_tags(&self, html: &Document) -> Option<Vec<String>> {
        let mut tags: Vec<String> = Vec::new();
        
        let genre_selectors = [
            ".genres-content a",       // Primary Madara genres selector
            ".manga-genres a",         // Alternative genres selector
            ".gnr a",                  // Short genres selector
            ".mgen a",                 // Manga genres selector
            ".seriestugenre a",        // Series genres selector
            "div.tags a",              // Tags container
            "div.manga-tags a",        // Manga tags
            ".wp-manga-genres a",      // WordPress manga genres
            ".post-content_item:contains('Genres') a", // Post content genres
            ".summary-heading:contains('Genres') + .summary-content a" // Summary genres
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
                    break;
                }
            }
        }
        
        if tags.is_empty() { None } else { Some(tags) }
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }
        
        let cleaned = date_str.trim().to_lowercase();
        
        // Try different date formats
        
        // Format: "17 août 2025" or "17 aout 2025"
        let parts: Vec<&str> = cleaned.split(' ').collect();
        if parts.len() == 3 {
            if let (Ok(day), Ok(year)) = (parts[0].parse::<i32>(), parts[2].parse::<i32>()) {
                let month = match parts[1] {
                    "janvier" => 1,
                    "février" | "fevrier" => 2,
                    "mars" => 3,
                    "avril" => 4,
                    "mai" => 5,
                    "juin" => 6,
                    "juillet" => 7,
                    "août" | "aout" => 8,
                    "septembre" => 9,
                    "octobre" => 10,
                    "novembre" => 11,
                    "décembre" | "decembre" => 12,
                    _ => 0,
                };
                
                if month > 0 {
                    let days_since_epoch = (year - 1970) * 365 + ((year - 1969) / 4) + self.days_in_months(month - 1, year) + day - 1;
                    return Some(days_since_epoch as i64 * 86400);
                }
            }
        }
        
        // Format: "17/08/2025" or "17-08-2025"
        for separator in &['/', '-', '.'] {
            if cleaned.contains(*separator) {
                let date_parts: Vec<&str> = cleaned.split(*separator).collect();
                if date_parts.len() == 3 {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        date_parts[0].parse::<i32>(), 
                        date_parts[1].parse::<i32>(), 
                        date_parts[2].parse::<i32>()
                    ) {
                        if day >= 1 && day <= 31 && month >= 1 && month <= 12 {
                            let full_year = if year < 100 { year + 2000 } else { year };
                            let days_since_epoch = (full_year - 1970) * 365 + ((full_year - 1969) / 4) + self.days_in_months(month - 1, full_year) + day - 1;
                            return Some(days_since_epoch as i64 * 86400);
                        }
                    }
                }
            }
        }
        
        // Format: "2025-08-17" (ISO format)
        if cleaned.len() == 10 && cleaned.chars().nth(4) == Some('-') && cleaned.chars().nth(7) == Some('-') {
            let iso_parts: Vec<&str> = cleaned.split('-').collect();
            if iso_parts.len() == 3 {
                if let (Ok(year), Ok(month), Ok(day)) = (
                    iso_parts[0].parse::<i32>(), 
                    iso_parts[1].parse::<i32>(), 
                    iso_parts[2].parse::<i32>()
                ) {
                    if day >= 1 && day <= 31 && month >= 1 && month <= 12 {
                        let days_since_epoch = (year - 1970) * 365 + ((year - 1969) / 4) + self.days_in_months(month - 1, year) + day - 1;
                        return Some(days_since_epoch as i64 * 86400);
                    }
                }
            }
        }
        
        None
    }

    fn days_in_months(&self, month: i32, year: i32) -> i32 {
        if month < 0 || month >= 12 {
            return 0;
        }

        let days = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        let mut total_days = days[month as usize];

        if month > 1 && self.is_leap_year(year) {
            total_days += 1;
        }

        total_days
    }

    fn is_leap_year(&self, year: i32) -> bool {
        (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
    }

    fn url_encode(&self, s: &str) -> String {
        s.chars()
            .map(|c| match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "+".to_string(),
                _ => c.to_string().bytes().map(|b| format!("%{:02X}", b)).collect(),
            })
            .collect()
    }
}

register_source!(MangasOrigines, ListingProvider, ImageRequestProvider);