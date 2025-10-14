#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
    MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, vec},
    imports::{net::Request, html::Document, std::send_partial_result},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString};

pub static BASE_URL: &str = "https://sushiscan.fr";

// Calculate content rating based on tags
fn calculate_content_rating(tags: &[String]) -> ContentRating {
	if tags.iter().any(|tag| {
		let lower = tag.to_lowercase();
		matches!(lower.as_str(), "ecchi" | "mature" | "adult" | "hentai" | "smut")
	}) {
		ContentRating::Suggestive
	} else {
		ContentRating::Safe
	}
}

// Calculate viewer type based on tags (Manhwa/Webtoon vs Manga)
fn calculate_viewer(tags: &[String]) -> Viewer {
	if tags.iter().any(|tag| {
		let lower = tag.to_lowercase();
		matches!(lower.as_str(), "manhwa" | "manhua" | "webtoon")
	}) {
		Viewer::Vertical
	} else {
		Viewer::RightToLeft
	}
}
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

fn urlencode(string: &str) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
    let hex = "0123456789abcdef".as_bytes();
    let bytes = string.as_bytes();

    for byte in bytes {
        let curr = *byte;
        if (b'a'..=b'z').contains(&curr)
            || (b'A'..=b'Z').contains(&curr)
            || (b'0'..=b'9').contains(&curr)
            || curr == b'-'
            || curr == b'_'
            || curr == b'.'
            || curr == b'~'
        {
            result.push(curr);
        } else if curr == b' ' {
            result.push(b'+');
        } else {
            result.push(b'%');
            result.push(hex[curr as usize >> 4]);
            result.push(hex[curr as usize & 15]);
        }
    }

    String::from_utf8(result).unwrap_or_default()
}

fn create_html_request(url: &str) -> Result<Document> {
    Ok(Request::get(url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .header("Upgrade-Insecure-Requests", "1")
        .header("Referer", BASE_URL)
        .html()?)
}

fn make_absolute_url(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), url)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), url)
    }
}

pub struct SushiScans;

impl Source for SushiScans {
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
        self.get_manga_from_page(&url)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let url = if manga.key.starts_with("catalogue/") {
            format!("{}/{}/", BASE_URL, manga.key)
        } else {
            format!("{}/catalogue/{}/", BASE_URL, manga.key)
        };

        let html = create_html_request(&url)?;

        self.parse_manga_details(html, manga.key, _needs_details, needs_chapters)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/{}/", BASE_URL, chapter.key);

        let html = create_html_request(&url)?;

        self.parse_page_list(html)
    }
}

impl ListingProvider for SushiScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        
        let url = match listing.name.as_str() {
            "Dernières" => format!("{}/catalogue/?page={}&order=update", BASE_URL, page),
            "Populaire" => format!("{}/catalogue/?page={}&order=popular", BASE_URL, page),
            "Nouveau" => format!("{}/catalogue/?page={}&order=latest", BASE_URL, page),
            _ => format!("{}/catalogue/?page={}", BASE_URL, page),
        };
        
        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for SushiScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl SushiScans {
    fn build_search_url(&self, query: Option<String>, page: i32, filters: Vec<FilterValue>) -> String {
        let mut included_tags: Vec<String> = Vec::new();
        let mut excluded_tags: Vec<String> = Vec::new();
        let mut status = String::new();
        let mut manga_type = String::new();
        
        // Status and type mappings handled directly in the match statements below
        
        // Process filters based on FilterValue structure
        for filter in filters {
            match filter {
                FilterValue::Text { value: _, .. } => {
                    // Title filter is handled by the query parameter, so we can ignore it here
                    continue;
                },
                FilterValue::Select { id, value } => {
                    match id.as_str() {
                        "status" => {
                            // Map French status values to their internal representations
                            status = match value.as_str() {
                                "En Cours" => "ongoing".to_string(),
                                "Terminé" => "completed".to_string(),
                                "Abandonné" => "hiatus".to_string(),
                                "En Pause" => "paused".to_string(),
                                _ => String::new(),
                            };
                        },
                        "type" => {
                            // Map French type values to their internal representations
                            manga_type = match value.as_str() {
                                "Manga" => "manga".to_string(),
                                "Manhwa" => "manhwa".to_string(),
                                "Manhua" => "manhua".to_string(),
                                "Comics" => "comics".to_string(),
                                "Fanfiction" => "fanfiction".to_string(),
                                "Webtoon FR" => "webtoon fr".to_string(),
                                "BD" => "bd".to_string(),
                                "Global-Manga" => "global-manga".to_string(),
                                "Guidebook" => "guidebook".to_string(),
                                "Artbook" => "artbook".to_string(),
                                "Anime-Comics" => "anime-comics".to_string(),
                                _ => String::new(),
                            };
                        },
                        "tags" => {
                            // Handle single select tags (legacy compatibility)
                            if !value.is_empty() && value != "Tout" {
                                included_tags.push(value);
                            }
                        },
                        _ => continue,
                    }
                },
                FilterValue::MultiSelect { id, included, excluded } => {
                    match id.as_str() {
                        "tags" => {
                            // Separate included and excluded genres (don't mix them!)
                            for genre_id in included {
                                if !genre_id.is_empty() && genre_id != "" {
                                    included_tags.push(genre_id);
                                }
                            }
                            for genre_id in excluded {
                                if !genre_id.is_empty() && genre_id != "" {
                                    excluded_tags.push(genre_id);
                                }
                            }
                        },
                        _ => continue,
                    }
                },
                _ => continue,
            }
        }
        
        // Build URL parameters like fr.lelmanga
        let mut url_params = Vec::new();
        
        // Add page parameter if not first page
        if page > 1 {
            url_params.push(format!("page={}", page));
        }
        
        // Add genre parameters with MangaStream logic
        if !included_tags.is_empty() || !excluded_tags.is_empty() {
            if excluded_tags.is_empty() {
                // Only included tags
                for tag in included_tags {
                    if !tag.is_empty() {
                        url_params.push(format!("genre%5B%5D={}", tag));
                    }
                }
            } else if !included_tags.is_empty() && !excluded_tags.is_empty() {
                // Both included and excluded tags
                for tag in included_tags {
                    if !tag.is_empty() {
                        url_params.push(format!("genre%5B%5D={}", tag));
                    }
                }
                for tag in excluded_tags {
                    if !tag.is_empty() {
                        url_params.push(format!("genre%5B%5D=-{}", tag));
                    }
                }
            } else {
                // Only excluded tags
                for tag in excluded_tags {
                    if !tag.is_empty() {
                        url_params.push(format!("genre%5B%5D=-{}", tag));
                    }
                }
            }
        }
        
        // Add status parameter
        if !status.is_empty() {
            url_params.push(format!("status={}", status));
        }
        
        // Add type parameter  
        if !manga_type.is_empty() {
            url_params.push(format!("type={}", manga_type.replace(' ', "%20")));
        }
        
        let url = if let Some(search_query) = query {
            if !search_query.is_empty() {
                let mut search_params = vec![format!("s={}", urlencode(&search_query))];
                search_params.extend(url_params);
                format!("{}/?{}", BASE_URL, search_params.join("&"))
            } else {
                // Empty search query - use catalog with filters
                if url_params.is_empty() {
                    format!("{}/catalogue/", BASE_URL)
                } else {
                    format!("{}/catalogue/?{}", BASE_URL, url_params.join("&"))
                }
            }
        } else {
            // No search query - browse/filter mode
            if url_params.is_empty() {
                format!("{}/catalogue/", BASE_URL)
            } else {
                format!("{}/catalogue/?{}", BASE_URL, url_params.join("&"))
            }
        };
        
        url
    }
    
    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = create_html_request(url)?;

        let mut entries: Vec<Manga> = Vec::new();

        // MangaStream selectors for sushiscan.fr
        if let Some(items) = html.select(".listupd .bsx, .utao .uta .imgu, .page-item-detail") {
            for item in items {
                let link = if let Some(a_element) = item.select("a") {
                    if let Some(first_link) = a_element.first() {
                        first_link
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let href = link.attr("href").unwrap_or_default();
                if href.is_empty() {
                    continue;
                }

                // Extract manga key from URL (catalogue/manga-name format)
                let key = href
                    .replace(BASE_URL, "")
                    .replace("/catalogue/", "")
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();

                if key.is_empty() {
                    continue;
                }

                let title = link.attr("title")
                    .or_else(|| {
                        item.select("h3 a, h5 a, .post-title, .manga-title")
                            .and_then(|elems| elems.first())
                            .and_then(|elem| elem.text())
                    })
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                if title.is_empty() {
                    continue;
                }

                let cover = if let Some(img_elements) = item.select("img") {
                    if let Some(img) = img_elements.first() {
                        img.attr("data-src")
                            .or_else(|| img.attr("data-lazy-src"))
                            .or_else(|| img.attr("src"))
                            .unwrap_or_default()
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

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

        // Check for pagination (MangaStream style)
        let has_next_page = html.select(".hpage a.r, .pagination .next").is_some();

        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }

    fn parse_manga_details(&self, html: Document, key: String, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        
        // Extract title with multiple MangaStream selectors
        let title = html.select("h1.entry-title, .wp-manga-title, .manga-title, .post-title h1, .single-title, h1")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| key.clone());

        // Extract cover with MangaStream template selectors
        let cover_selectors = [
            ".infomanga > div[itemprop=image] img",  // From old config
            ".thumb img",                            // From old config  
            ".wp-post-image",                        // WordPress featured image
            ".manga-poster img",                     // Manga poster
            ".post-thumb img",                       // Post thumbnail
            ".series-thumb img",                     // Series thumbnail
            "div.summary_image img",                 // Madara selector
            ".manga-summary img",                    // Manga summary
            "article img:first-child",               // First article image
        ];
        
        let mut cover = String::new();
        for selector in &cover_selectors {
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(src) = img_elem.attr("data-src")
                    .or_else(|| img_elem.attr("data-lazy-src"))
                    .or_else(|| img_elem.attr("src")) {
                    if !src.is_empty() {
                        cover = src.to_string();
                        break;
                    }
                }
            }
        }

        // Extract author with MangaStream selectors
        let author_selectors = [
            ".infotable td:contains(Auteur)+td",     // From old config
            ".infotable td:contains(Author)+td",     // English version
            ".author-content a",                     // Author content
            ".manga-authors",                        // Manga authors
            ".imptdt:contains(Auteur) i",           // French info table
            ".fmed b:contains(Author) + span",       // Alternative layout
            "span:contains(Author:)",                // Generic author span
        ];
        
        let mut author = None;
        for selector in &author_selectors {
            if let Some(author_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(author_text) = author_elem.text() {
                    let author_str = author_text.trim().to_string();
                    if !author_str.is_empty() {
                        author = Some(vec![author_str]);
                        break;
                    }
                }
            }
        }

        // Extract description with multiple selectors
        let description_selectors = [
            "div.desc p",                            // From old config
            "div.entry-content p",                   // From old config
            "div[itemprop=description]:not(:has(p))",// From old config
            ".summary__content p",                   // Madara summary
            ".description-summary p",                // Description summary
            ".manga-excerpt p",                      // Manga excerpt
            ".post-content p",                       // Post content
            ".synopsis",                             // Synopsis
        ];
        
        let mut description = None;
        for selector in &description_selectors {
            if let Some(desc_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(desc_text) = desc_elem.text() {
                    let desc_str = desc_text.trim().to_string();
                    if !desc_str.is_empty() && desc_str.len() > 10 {
                        description = Some(desc_str);
                        break;
                    }
                }
            }
        }

        // Extract status with French terms
        let status_selectors = [
            ".infotable td:contains(Statut)+td",     // From old config
            ".infotable td:contains(Status)+td",     // English version
            ".post-status .summary-content",         // Madara status
            ".manga-status",                         // Manga status
            ".imptdt:contains(Statut) i",           // French info table
            ".tsinfo .imptdt:contains(Status) i",    // Theme specific
        ];
        
        let mut status = MangaStatus::Unknown;
        for selector in &status_selectors {
            if let Some(status_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(status_text) = status_elem.text() {
                    let status_str = status_text.trim().to_lowercase()
                        .replace("é", "e")  // Handle French accents
                        .replace("è", "e");
                    
                    status = match status_str.as_str() {
                        s if s.contains("en cours") || s.contains("ongoing") || s.contains("publication") => MangaStatus::Ongoing,
                        s if s.contains("termine") || s.contains("completed") || s.contains("fini") || s.contains("complet") => MangaStatus::Completed,
                        s if s.contains("abandonne") || s.contains("cancelled") || s.contains("canceled") => MangaStatus::Cancelled,
                        s if s.contains("en pause") || s.contains("hiatus") || s.contains("pause") => MangaStatus::Hiatus,
                        _ => MangaStatus::Unknown,
                    };
                    
                    if status != MangaStatus::Unknown {
                        break;
                    }
                }
            }
        }

        // Extract tags/genres with multiple selectors
        let genre_selectors = [
            ".seriestugenre a",                      // From old config
            ".genres-content a",                     // Genres content
            ".manga-genres a",                       // Manga genres
            ".gnr a",                                // Short genres
            ".mgen a",                               // Manga genres short
            "span.mgen a",                           // Span manga genres
            ".wp-manga-genres a",                    // WordPress genres
        ];
        
        let mut tags: Vec<String> = Vec::new();
        for selector in &genre_selectors {
            if let Some(genre_items) = html.select(selector) {
                for genre in genre_items {
                    if let Some(genre_text) = genre.text() {
                        let genre_str = genre_text.trim().to_string();
                        if !genre_str.is_empty() && !tags.contains(&genre_str) {
                            tags.push(genre_str);
                        }
                    }
                }
                if !tags.is_empty() {
                    break;
                }
            }
        }

        // Calculate content_rating and viewer based on tags
        let content_rating = calculate_content_rating(&tags);
        let viewer = calculate_viewer(&tags);

        let mut manga = Manga {
            key: key.clone(),
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            authors: author,
            artists: None,
            description,
            url: Some(format!("{}/catalogue/{}/", BASE_URL, key)),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status,
            content_rating,
            viewer,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        };

        if _needs_details {
            send_partial_result(&manga);
        }

        if needs_chapters {
            manga.chapters = Some(self.parse_chapter_list(&html)?);
        }

        Ok(manga)
    }

    fn parse_chapter_list(&self, html: &Document) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();

        // Multiple MangaStream chapter selectors
        let chapter_selectors = [
            "#chapterlist li",                      // From template default
            ".wp-manga-chapter",                    // WordPress manga chapters
            "li.wp-manga-chapter",                  // List item manga chapters
            ".manga-chapters li",                   // Manga chapters list
            ".chapter-list li",                     // Chapter list items
            "div.bxcl li",                         // Box chapter list
            "div.cl li",                           // Chapter list
            ".listing-chapters_wrap li",           // Listing chapters wrapper
        ];
        
        for selector in &chapter_selectors {
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    
                    for item in items_vec {
                        let link = if let Some(a_element) = item.select("a") {
                            if let Some(first_link) = a_element.first() {
                                first_link
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        let href = link.attr("href").unwrap_or_default();
                        if href.is_empty() {
                            continue;
                        }

                        let chapter_key = href
                            .replace(BASE_URL, "")
                            .trim_start_matches('/')
                            .trim_end_matches('/')
                            .to_string();

                        // Extract title with multiple selectors
                        let raw_title = link.text()
                            .or_else(|| {
                                item.select("span.chapternum, .lch a, .chapter-manhwa-title, .chapternum")
                                    .and_then(|elems| elems.first())
                                    .and_then(|elem| elem.text())
                            })
                            .unwrap_or_default()
                            .trim()
                            .to_string();
                        
                        if raw_title.is_empty() {
                            continue;
                        }

                        // Extract date from title and clean title (e.g., "Ch.200 - Chapitre 200 June 23, 2024")
                        let (clean_title, title_date) = self.extract_date_from_title(&raw_title);
                        let title = clean_title;

                        // Extract chapter number from clean title
                        let chapter_number = self.extract_chapter_number(&title);

                        // Extract date with multiple methods: 1) from title, 2) from selectors
                        let mut date_uploaded = title_date; // Use title date first if found
                        
                        // If no date from title, try CSS selectors
                        if date_uploaded.is_none() {
                            let date_selectors = [
                                "span.chapterdate",                 // From template default
                                ".chapterdate",                     // Chapter date class
                                ".dt",                              // Date class
                                ".chapter-release-date",            // Chapter release date
                                ".chapter-date",                    // Chapter date
                                "span.date",                        // Date span
                                "time",                             // Time element
                                ".post-on",                         // Post date
                                ".uploaded-on",                     // Upload date
                            ];
                            
                            for date_selector in &date_selectors {
                                if let Some(date_elem) = item.select(date_selector).and_then(|elems| elems.first()) {
                                    if let Some(date_text) = date_elem.text() {
                                        let date_str = date_text.trim();
                                        if !date_str.is_empty() {
                                            if let Some(parsed_date) = self.parse_chapter_date(date_str) {
                                                date_uploaded = Some(parsed_date);
                                                break;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        let url = make_absolute_url(BASE_URL, &href);

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

        Ok(chapters)
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        // MangaStream page selectors with alt_pages support
        let selectors = [
            "div#readerarea img",
            ".rdminimal img",
            ".reader-area img",
            "#chapter_imgs img",
            ".chapter-content img"
        ];

        for selector in &selectors {
            if let Some(images) = html.select(selector) {
                for img_element in images {
                    let img_url = img_element.attr("data-src")
                        .or_else(|| img_element.attr("data-lazy-src"))
                        .or_else(|| img_element.attr("src"))
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
                if !pages.is_empty() {
                    break;
                }
            }
        }

        Ok(pages)
    }

    fn extract_chapter_number(&self, title: &str) -> f32 {
        let title_lower = title.to_lowercase();

        if let Some(pos) = title_lower.find("chapitre") {
            let after_ch = &title[pos + "chapitre".len()..].trim();
            if let Some(num_str) = after_ch.split_whitespace().next() {
                if let Ok(num) = num_str.replace(',', ".").parse::<f32>() {
                    return num;
                }
            }
        }

        for word in title.split_whitespace() {
            if let Ok(num) = word.parse::<f32>() {
                return num;
            }
        }

        1.0
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }
        
        let cleaned = date_str.trim().to_lowercase();
        
        // French date parsing: "17 août 2025" or "17 aout 2025"
        let parts: Vec<&str> = cleaned.split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(day), Ok(year)) = (parts[0].parse::<u32>(), parts[2].parse::<i32>()) {
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
                
                if month > 0 && day >= 1 && day <= 31 && year >= 1970 {
                    // Accurate timestamp calculation
                    let days_since_epoch = self.calculate_days_since_epoch(year, month, day);
                    return Some(days_since_epoch as i64 * 86400);
                }
            }
        }
        
        // Try parsing other common formats
        // Format: "17/08/2025" or "17-08-2025" 
        for separator in &['/', '-', '.'] {
            if cleaned.contains(*separator) {
                let date_parts: Vec<&str> = cleaned.split(*separator).collect();
                if date_parts.len() == 3 {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        date_parts[0].parse::<u32>(), 
                        date_parts[1].parse::<u32>(), 
                        date_parts[2].parse::<i32>()
                    ) {
                        if day >= 1 && day <= 31 && month >= 1 && month <= 12 && year >= 1970 {
                            let full_year = if year < 100 { year + 2000 } else { year };
                            let days_since_epoch = self.calculate_days_since_epoch(full_year, month, day);
                            return Some(days_since_epoch as i64 * 86400);
                        }
                    }
                }
            }
        }
        
        None
    }
    
    fn calculate_days_since_epoch(&self, year: i32, month: u32, day: u32) -> i32 {
        // Days cumulated for each month in non-leap year (0-indexed)
        let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        
        // Years since epoch
        let years_since_epoch = year - 1970;
        
        // Count leap years between 1970 and year (not including current year)
        let leap_days = (1970..year).filter(|&y| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0).count() as i32;
        
        // Days for complete years
        let mut days = years_since_epoch * 365 + leap_days;
        
        // Add days for complete months in current year
        days += days_before_month[(month - 1) as usize];
        
        // Add one day if current year is leap and we're past February
        if ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) && month > 2 {
            days += 1;
        }
        
        // Add days in current month (subtract 1 because we count from day 1)
        days + (day - 1) as i32
    }
    
    fn extract_date_from_title(&self, title: &str) -> (String, Option<i64>) {
        // Look for English date patterns like "June 23, 2024" at the end of title
        // Common patterns: "Month DD, YYYY" or "Month DD YYYY"
        
        let months = [
            ("january", 1), ("february", 2), ("march", 3), ("april", 4),
            ("may", 5), ("june", 6), ("july", 7), ("august", 8),
            ("september", 9), ("october", 10), ("november", 11), ("december", 12),
            ("jan", 1), ("feb", 2), ("mar", 3), ("apr", 4),
            ("jun", 6), ("jul", 7), ("aug", 8),
            ("sep", 9), ("oct", 10), ("nov", 11), ("dec", 12),
        ];
        
        let title_lower = title.to_lowercase();
        
        // Try to find month name in the title
        for (month_name, month_num) in &months {
            if let Some(month_pos) = title_lower.find(month_name) {
                // Extract the part after month name
                let after_month = &title[month_pos + month_name.len()..];
                
                // Look for pattern: " DD, YYYY" or " DD YYYY"
                let parts: Vec<&str> = after_month.trim().split_whitespace().collect();
                if parts.len() >= 2 {
                    // Try to parse day and year
                    if let Ok(day) = parts[0].trim_end_matches(',').parse::<u32>() {
                        if let Ok(year) = parts[1].parse::<i32>() {
                            if day >= 1 && day <= 31 && year >= 1970 {
                                // Calculate timestamp
                                let timestamp = self.calculate_days_since_epoch(year, *month_num, day) as i64 * 86400;
                                
                                // Clean title by removing the date part
                                let date_start = month_pos;
                                let clean_title = title[..date_start].trim().to_string();
                                
                                return (clean_title, Some(timestamp));
                            }
                        }
                    }
                }
            }
        }
        
        // If no date found, return original title and None
        (title.to_string(), None)
    }
}

register_source!(SushiScans, ListingProvider, ImageRequestProvider);