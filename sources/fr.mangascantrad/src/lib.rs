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
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        // Process filters to build search parameters
        let mut status_filters = Vec::new();
        let mut genre_filters = Vec::new();
        let mut genre_op = String::from(""); // Default to OR
        
        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "status" && !value.is_empty() && value != "Tout" {
                        // Map French status names to Madara status codes
                        match value.as_str() {
                            "En cours" => status_filters.push("on-going"),
                            "Terminé" => status_filters.push("end"),
                            "Annulé" => status_filters.push("canceled"), 
                            "En pause" => status_filters.push("on-hold"),
                            _ => {}
                        }
                    } else if id == "op" {
                        // Set genre condition (AND/OR)
                        genre_op = if value == "AND" { "1".to_string() } else { "".to_string() };
                    } else if id == "genres" && !value.is_empty() && value != "Tout" {
                        // Map genre display name to slug using filters.json mapping
                        if let Some(genre_slug) = self.get_genre_slug(value) {
                            genre_filters.push(genre_slug);
                        }
                    }
                }
                _ => {}
            }
        }
        
        // Use filtered search if filters are applied or query is present
        if query.is_some() || !status_filters.is_empty() || !genre_filters.is_empty() {
            self.ajax_filtered_search(query, page, status_filters, genre_filters, &genre_op)
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
        // Use Madara template approach: add ?style=list parameter for better image loading
        let url = format!("{}/{}/?style=list", BASE_URL, chapter.key);
        
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
    fn urlencode(input: &str) -> String {
        input.chars().map(|c| {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                ' ' => "%20".to_string(),
                _ => format!("%{:02X}", c as u8)
            }
        }).collect()
    }
    fn ajax_manga_list(&self, page: i32) -> Result<MangaPageResult> {
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Madara AJAX payload for general listing with more results per page
        let body = format!(
            "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
            page - 1, // Madara uses 0-based indexing
            page
        );
        
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        
        self.parse_ajax_response(html_doc)
    }
    
    fn ajax_manga_listing(&self, listing_type: &str, page: i32) -> Result<MangaPageResult> {
        
        let url = format!("{}/wp-admin/admin-ajax.php", BASE_URL);
        
        // Different payloads for different listing types
        let body = match listing_type {
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
            _ => {
                format!(
                    "action=madara_load_more&page={}&template=madara-core/content/content-archive&vars%5Borderby%5D=post_title&vars%5Bpaged%5D={}&vars%5Btemplate%5D=archive&vars%5Bpost_type%5D=wp-manga&vars%5Bpost_status%5D=publish&vars%5Border%5D=ASC&vars%5Bmanga_archives_item_layout%5D=big_thumbnail&vars%5Bposts_per_page%5D=20&vars%5Bnumberposts%5D=20",
                    page - 1,
                    page
                )
            }
        };
        
        
        let html_doc = Request::post(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Accept", "*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .header("X-Requested-With", "XMLHttpRequest")
            .body(body.as_bytes())
            .html()?;
        
        
        self.parse_ajax_response(html_doc)
    }
    
    fn get_genre_slug(&self, genre_name: &str) -> Option<String> {
        // Map genre display names to their slugs based on filters.json
        match genre_name {
            "4-koma" => Some("4-koma".to_string()),
            "Action" => Some("action".to_string()),
            "Adulte" => Some("adulte".to_string()),
            "Amitié" => Some("amitie".to_string()),
            "Amour" => Some("amour".to_string()),
            "Animation" => Some("animation".to_string()),
            "Arts Martiaux" => Some("arts-martiaux".to_string()),
            "Aventure" => Some("aventure".to_string()),
            "Boxe" => Some("boxe".to_string()),
            "Combat" => Some("combat".to_string()),
            "Comédie" => Some("comedie".to_string()),
            "comedy" => Some("comedy".to_string()),
            "crime" => Some("crime".to_string()),
            "cybernétique" => Some("cybernetique".to_string()),
            "démons" => Some("demons".to_string()),
            "Doujinshi" => Some("doujinshi".to_string()),
            "Drame" => Some("drame".to_string()),
            "E-sport" => Some("e-sport".to_string()),
            "Ecchi" => Some("ecchi".to_string()),
            "Espionnage" => Some("espionnage".to_string()),
            "Famille" => Some("famille".to_string()),
            "Fantaisie" => Some("fantaisie".to_string()),
            "Fantastique" => Some("fantastique".to_string()),
            "Gender Bender" => Some("gender-bender".to_string()),
            "Guerre" => Some("guerre".to_string()),
            "Harcèlement" => Some("harcelement".to_string()),
            "Harem" => Some("harem".to_string()),
            "Hentai" => Some("hentai".to_string()),
            "Historique" => Some("historique".to_string()),
            "Horreur" => Some("horreur".to_string()),
            "isekaï" => Some("isekai".to_string()),
            "Jeux vidéo" => Some("jeux-video".to_string()),
            "Josei" => Some("josei".to_string()),
            "Magical Girls" => Some("magical-girls".to_string()),
            "magie" => Some("magie".to_string()),
            "Mature" => Some("mature".to_string()),
            "Mecha" => Some("mecha".to_string()),
            "Monstres" => Some("monstres".to_string()),
            "Mystère" => Some("mystere".to_string()),
            "One Shot" => Some("one-shot".to_string()),
            "Organisation secrète" => Some("organisation-secrete".to_string()),
            "Parodie" => Some("parodie".to_string()),
            "Policier" => Some("policier".to_string()),
            "Psychologique" => Some("psychologique".to_string()),
            "Realité Virtuel" => Some("realite-virtuel".to_string()),
            "Réincarnation" => Some("reincarnation".to_string()),
            "Returner" => Some("returner".to_string()),
            "Romance" => Some("romance".to_string()),
            "Science-fiction" => Some("science-fiction".to_string()),
            "Seinen" => Some("seinen".to_string()),
            "Shôjo" => Some("shojo".to_string()),
            "Shôjo Ai" => Some("shojo-ai".to_string()),
            "Shonen" => Some("shonen".to_string()),
            "Shônen Ai" => Some("shonen-ai".to_string()),
            "Smut" => Some("smut".to_string()),
            "Sport" => Some("sport".to_string()),
            "Sports" => Some("sports".to_string()),
            "Steampunk" => Some("steampunk".to_string()),
            "Super héros" => Some("super-heros".to_string()),
            "Surnaturel" => Some("surnaturel".to_string()),
            "Technologie" => Some("technologie".to_string()),
            "Tournoi" => Some("tournoi".to_string()),
            "Tragédie" => Some("tragedie".to_string()),
            "Tranches de vie" => Some("tranches-de-vie".to_string()),
            "vampires" => Some("vampires".to_string()),
            "Vengeance" => Some("vengeance".to_string()),
            "Vie scolaire" => Some("vie-scolaire".to_string()),
            "Virtuel world" => Some("virtuel-world".to_string()),
            "Voyage Temporel" => Some("voyage-temporel".to_string()),
            "Webtoons" => Some("webtoons".to_string()),
            "Yaoi" => Some("yaoi".to_string()),
            "Yuri" => Some("yuri".to_string()),
            _ => None
        }
    }
    
    fn ajax_filtered_search(
        &self, 
        query: Option<String>, 
        page: i32, 
        status_filters: Vec<&str>, 
        genre_filters: Vec<String>,
        genre_op: &str
    ) -> Result<MangaPageResult> {
        // Build search URL with parameters like the old Madara implementation
        let mut url = format!("{}/page/{}/", BASE_URL, page);
        let mut params = Vec::new();
        
        // Add search query if present
        if let Some(search_query) = query {
            params.push(format!("s={}", Self::urlencode(&search_query)));
        }
        
        // Add post_type parameter
        params.push("post_type=wp-manga".to_string());
        
        // Add status filters
        for status in &status_filters {
            params.push(format!("status[]={}", status));
        }
        
        // Add genre filters
        for genre in &genre_filters {
            params.push(format!("genre[]={}", Self::urlencode(genre)));
        }
        
        // Add genre condition (AND/OR)
        if !genre_op.is_empty() {
            params.push(format!("op={}", genre_op));
        }
        
        // Construct final URL
        if !params.is_empty() {
            url.push('?');
            url.push_str(&params.join("&"));
        }
        
        let html_doc = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;
            
        self.parse_manga_page(html_doc)
    }
    
    fn ajax_search(&self, query: &str, page: i32) -> Result<MangaPageResult> {
        // Use the new filtered search method with empty filters
        self.ajax_filtered_search(Some(query.to_string()), page, Vec::new(), Vec::new(), "")
    }
    
    fn ajax_chapter_list(&self, manga_key: &str) -> Result<Vec<Chapter>> {
        
        // Step 1: Get the manga page to extract the numeric ID
        let manga_url = format!("{}/manga/{}/", BASE_URL, manga_key);
        
        let manga_page_doc = Request::get(&manga_url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .html()?;
        
        // Step 2: Extract numeric ID from JavaScript (exactly like old Madara implementation)
        let int_id = self.extract_manga_int_id(&manga_page_doc)?;
        
        // Step 3: Use Madara alt_ajax method - POST to /manga/{key}/ajax/chapters  
        let ajax_url = format!("{}/manga/{}/ajax/chapters", BASE_URL, manga_key);
        let body_content = format!("action=manga_get_chapters&manga={}", int_id);
        
        
        let ajax_doc = Request::post(&ajax_url)?
            .header("User-Agent", USER_AGENT)
            .header("Content-Type", "application/x-www-form-urlencoded")
            .header("Referer", &manga_url)
            .body(body_content.as_bytes())
            .html()?;
        
        
        // Parse the response (should contain the chapter HTML fragment)
        match self.parse_ajax_chapters_response(ajax_doc) {
            Ok(chapters) => {
                if !chapters.is_empty() {
                    return Ok(chapters);
                } else {
                }
            }
            Err(_e) => {
            }
        }
        
        // Fallback: try to parse from main page
        if let Ok(chapters) = self.parse_chapter_list(&manga_page_doc) {
            if !chapters.is_empty() {
                return Ok(chapters);
            }
        }
        
        Ok(vec![])
    }
    
    fn extract_manga_int_id(&self, html: &Document) -> Result<String> {
        
        // Look for the wp-manga-js-extra script tag (like in old Madara implementation)
        if let Some(script_element) = html.select("script#wp-manga-js-extra") {
            if let Some(script_content) = script_element.html() {
                let script_text = script_content;
                
                // Look for manga ID in the script (usually in a variable like manga_id or similar)
                if let Some(start) = script_text.find("\"manga_id\":\"") {
                    let after_start = &script_text[start + 12..];
                    if let Some(end) = after_start.find("\"") {
                        let manga_id = &after_start[..end];
                        return Ok(manga_id.to_string());
                    }
                }
                
                // Alternative pattern: look for numeric ID in different formats
                if let Some(start) = script_text.find("manga_id=") {
                    let after_start = &script_text[start + 9..];
                    if let Some(end) = after_start.find(|c: char| !c.is_ascii_digit()) {
                        let manga_id = &after_start[..end];
                        return Ok(manga_id.to_string());
                    }
                }
            }
        }
        
        // If not found in script, try to extract from other common locations
        
        // For now, return a placeholder ID and let the POST request handle it
        Ok("0".to_string())
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }
        
        
        let parts: Vec<&str> = date_str.trim().split_whitespace().collect();
        if parts.len() >= 3 {
            let day_str = parts[0];
            let month_name = parts[1];
            let year_str = parts[2];
            
            if let (Ok(day), Ok(year)) = (day_str.parse::<u32>(), year_str.parse::<i32>()) {
                let month = match month_name.to_lowercase().as_str() {
                    "janvier" => 1,
                    "février" => 2,
                    "mars" => 3,
                    "avril" => 4,
                    "mai" => 5,
                    "juin" => 6,
                    "juillet" => 7,
                    "août" => 8,
                    "septembre" => 9,
                    "octobre" => 10,
                    "novembre" => 11,
                    "décembre" => 12,
                    _ => {
                        return None;
                    }
                };
                
                // Use precise calculation like real calendar libraries
                let timestamp = self.date_to_timestamp(year, month, day);
                
                return Some(timestamp);
            }
        }
        
        None
    }
    
    fn date_to_timestamp(&self, year: i32, month: u32, day: u32) -> i64 {
        // Days cumulated for each month in non-leap year (0-indexed)
        let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
        
        // Years since epoch
        let years_since_epoch = year - 1970;
        
        // Count leap years between 1970 and year (not including current year)
        let leap_days = ((1970..year).filter(|&y| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0).count()) as i32;
        
        // Days for complete years
        let mut days = years_since_epoch * 365 + leap_days;
        
        // Add days for complete months in current year
        days += days_before_month[(month - 1) as usize] as i32;
        
        // Add one day if current year is leap and we're past February
        if ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) && month > 2 {
            days += 1;
        }
        
        // Add days in current month (subtract 1 because we count from day 1)
        days += (day - 1) as i32;
        
        // Convert to seconds
        days as i64 * 86400
    }
    
    fn parse_ajax_chapters_response(&self, html: Document) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();

        // Debug: Print the raw HTML response to see what we're actually getting
        if let Some(body) = html.select("body") {
            if let Some(first) = body.first() {
                let html_text = first.text().unwrap_or_default();
                
                // Check if response contains expected chapter content
                if html_text.contains("Chapitre") {
                } else {
                }
            }
        }

        // Debug: Try finding any li elements first
        if let Some(all_li) = html.select("li") {
            let li_vec: Vec<_> = all_li.collect();
            
            // Show first few li elements
            for (_idx, li) in li_vec.iter().enumerate().take(5) {
                let _class = li.attr("class").unwrap_or_default();
                let _text = li.text().unwrap_or_default();
            }
        }

        // Use the exact structure from the AJAX response we analyzed
        if let Some(chapter_items) = html.select("li.wp-manga-chapter") {
            let items_vec: Vec<_> = chapter_items.collect();
            
            for (_idx, item) in items_vec.iter().enumerate() {
                // Get the chapter link
                if let Some(link) = item.select("a").and_then(|links| links.first()) {
                    let href = link.attr("href").unwrap_or_default();
                    if href.is_empty() {
                        continue;
                    }

                    let title = link.text()
                        .unwrap_or_default()
                        .trim()
                        .to_string();
                    
                    if title.is_empty() {
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

                    // Extract date if available - with detailed debugging
                    let date_uploaded = if let Some(date_elem) = item.select("span.chapter-release-date i")
                        .and_then(|elems| elems.first()) {
                        if let Some(raw_date) = date_elem.text() {
                            let date_str = raw_date.trim();
                            self.parse_chapter_date(date_str)
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // Ensure URL is absolute
                    let url = if href.starts_with("http") {
                        href
                    } else if href.starts_with("/") {
                        format!("{}{}", BASE_URL, href)
                    } else {
                        format!("{}/{}", BASE_URL, href)
                    };

                    let _date_debug = if let Some(ts) = date_uploaded {
                        format!("timestamp={}", ts)
                    } else {
                        "no_date".to_string()
                    };

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
            }
        } else {
        }

        Ok(chapters)
    }
    
    
    fn parse_ajax_response(&self, html: Document) -> Result<MangaPageResult> {
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
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    found_items = true;
                    
                    for (_idx, item) in items_vec.iter().enumerate() {
                        
                        // Find the link element
                        let link = if let Some(links) = item.select("a") {
                            if let Some(first_link) = links.first() {
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
                            continue;
                        }
                        
                        
                        let key = self.extract_manga_id(&href);
                        
                        // Extract cover image using Madara template approach
                        let cover = item.select("img")
                            .and_then(|imgs| imgs.first())
                            .and_then(|img| {
                                // Same attribute priority as Madara template
                                img.attr("data-src")
                                    .or_else(|| img.attr("data-lazy-src"))
                                    .or_else(|| img.attr("src"))
                                    .or_else(|| img.attr("srcset"))
                                    .or_else(|| img.attr("data-cfsrc"))
                            })
                            .map(|src| {
                                // Clean up srcset if needed (take first URL)
                                if src.contains(" ") {
                                    src.split_whitespace().next().unwrap_or("").to_string()
                                } else {
                                    src.to_string()
                                }
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
                    break; // Stop after finding items with one selector
                }
            }
        }
        
        if !found_items {
            // Try to print the HTML for debugging
            if let Some(body) = html.select("body") {
                if let Some(first) = body.first() {
                    let _html_text = first.text().unwrap_or_default();
                }
            }
        }
        
        // Pagination logic: if we got any results, assume there might be more
        // Madara typically returns 10-12 items per page, so we check if we got a reasonable amount
        let has_next_page = entries.len() >= 8; // Conservative threshold
        
        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }
    
    fn parse_manga_page(&self, html: Document) -> Result<MangaPageResult> {
        // For normal HTML pages, use the same parsing logic as AJAX but with broader selectors
        // This handles search/filter results from GET requests
        self.parse_ajax_response(html)
    }
    
    fn parse_manga_details(&self, html: Document, manga_key: String, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        
        // Extract title with multiple selectors
        let title = html.select(".post-title h1, .manga-title, h1.entry-title, .wp-manga-title, .single-title")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| {
                manga_key.clone()
            });
        

        // Extract cover using Madara template approach with more selectors
        let cover_selectors = [
            "div.summary_image img",      // Primary Madara selector
            ".wp-post-image",             // WordPress featured image
            ".manga-poster img",          // Common manga poster
            ".post-thumb img",            // Post thumbnail
            ".series-thumb img",          // Series thumbnail
            ".thumb img",                 // Generic thumb
            "img.attachment-post-thumbnail", // WordPress attachment
            ".infomanga img",             // Info manga section
            "div[itemprop=image] img",    // Schema.org structured data
            ".post-content img:first-child", // First content image
            ".entry-content img:first-child", // Entry content image
            ".manga-summary img",         // Manga summary image
            "article img:first-child",    // First article image
        ];
        
        let mut cover = String::new();
        for selector in &cover_selectors {
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                // Use same attribute priority as Madara template
                if let Some(src) = img_elem.attr("data-src")
                    .or_else(|| img_elem.attr("data-lazy-src"))
                    .or_else(|| img_elem.attr("src"))
                    .or_else(|| img_elem.attr("srcset"))
                    .or_else(|| img_elem.attr("data-cfsrc")) {
                    if !src.is_empty() {
                        // Clean up srcset if needed (take first URL)
                        cover = if src.contains(" ") {
                            src.split_whitespace().next().unwrap_or("").to_string()
                        } else {
                            src.to_string()
                        };
                        break;
                    }
                }
            }
        }
        
        if cover.is_empty() {
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
                    break;
                }
            }
        }

        // Extract status with comprehensive selectors
        let status_selectors = [
            "div.post-content_item:contains(Statut) div.summary-content",   // French Madara primary
            ".post-status .summary-content",                                // Standard Madara
            ".imptdt:contains(Statut) i",                                  // French status info
            ".imptdt:contains(Status) i",                                  // English status info  
            ".manga-status",                                               // Generic manga status
            ".status",                                                     // Generic status
            ".series-status",                                             // Series status
            ".tsinfo .imptdt:contains(Status) i",                         // Theme specific
            ".fmed b:contains(Status) + span",                            // Alternative layout
            ".fmed b:contains(Statut) + span",                            // French alternative
            ".spe span:contains(Status) + span",                          // Special span layout
            ".spe span:contains(Statut) + span",                          // French special span
            "div.summary-content .post-status span",                       // Status in summary
            ".post-content .post-content_item .summary-content:contains(Statut)", // Content item
        ];
        
        let mut status = MangaStatus::Unknown;
        for selector in &status_selectors {
            if let Some(status_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(status_text) = status_elem.text() {
                    let status_str = status_text.trim().to_lowercase()
                        .replace("é", "e")  // Handle French accents
                        .replace("è", "e")
                        .replace("à", "a");
                    
                    
                    status = match status_str.as_str() {
                        "en cours" | "ongoing" | "en_cours" | "en-cours" | "publication" | "publiant" | "continu" => MangaStatus::Ongoing,
                        "termine" | "completed" | "fini" | "acheve" | "complet" | "fin" | "end" => MangaStatus::Completed,
                        "annule" | "cancelled" | "canceled" | "arrete" | "abandon" | "abandonne" => MangaStatus::Cancelled,
                        "en pause" | "hiatus" | "pause" | "en_pause" | "en-pause" | "on hold" | "interrompu" | "suspendu" => MangaStatus::Hiatus,
                        _ => {
                            // Try partial matches for more flexibility
                            if status_str.contains("cours") || status_str.contains("ongoing") || status_str.contains("publication") {
                                MangaStatus::Ongoing
                            } else if status_str.contains("termine") || status_str.contains("completed") || status_str.contains("fini") {
                                MangaStatus::Completed
                            } else if status_str.contains("annule") || status_str.contains("cancelled") {
                                MangaStatus::Cancelled
                            } else if status_str.contains("pause") || status_str.contains("hiatus") {
                                MangaStatus::Hiatus
                            } else {
                                MangaStatus::Unknown
                            }
                        }
                    };
                    
                    if status != MangaStatus::Unknown {
                        break;
                    }
                }
            }
        }

        // Fallback: try broader selectors if no specific status found
        if status == MangaStatus::Unknown {
            
            // Try to find any element containing status-related text
            if let Some(status_elem) = html.select("*").and_then(|elements| {
                elements.into_iter().find(|elem| {
                    if let Some(text) = elem.text() {
                        let text_lower = text.to_lowercase();
                        text_lower.contains("statut") || text_lower.contains("status")
                    } else {
                        false
                    }
                })
            }) {
                if let Some(status_text) = status_elem.text() {
                    let status_str = status_text.trim().to_lowercase()
                        .replace("é", "e")
                        .replace("è", "e")
                        .replace("à", "a");
                    
                    if status_str.contains("en cours") || status_str.contains("ongoing") {
                        status = MangaStatus::Ongoing;
                    } else if status_str.contains("termine") || status_str.contains("completed") {
                        status = MangaStatus::Completed;
                    }
                }
            }
        }
        
        if status == MangaStatus::Unknown {
        }

        let authors = if !author.is_empty() {
            Some(vec![author])
        } else {
            None
        };
        
        // Parse chapters if requested
        let chapters = if needs_chapters {
            
            // First try AJAX approaches
            let ajax_chapters = self.ajax_chapter_list(&manga_key).unwrap_or_else(|_e| {
                vec![]
            });
            
            if !ajax_chapters.is_empty() {
                Some(ajax_chapters)
            } else {
                match self.parse_chapter_list(&html) {
                    Ok(chapter_list) => {
                        if !chapter_list.is_empty() {
                            Some(chapter_list)
                        } else {
                            // Try basic parsing again with different approach
                            match self.parse_chapter_list(&html) {
                                Ok(enhanced_chapters) => {
                                    if !enhanced_chapters.is_empty() {
                                        Some(enhanced_chapters)
                                    } else {
                                        None
                                    }
                                }
                                Err(_e3) => {
                                    None
                                }
                            }
                        }
                    }
                    Err(_e2) => {
                        None
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
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    found_chapters = true;
                    
                    for (_idx, item) in items_vec.iter().enumerate() {
                        // Find the link element
                        let link = if let Some(links) = item.select("a") {
                            if let Some(first_link) = links.first() {
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
        }

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

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();
        

        // Primary selector (same as Madara template default)
        let image_selectors = [
            "div.page-break > img",              // Madara default selector
            ".page-break img",                   // Alternative page break
            ".reading-content img",              // Reading content
            ".wp-manga-chapter-img",             // WordPress manga images
            "img.wp-manga-chapter-img",          // Specific manga chapter images
            ".chapter-content img",              // Chapter content images
            "div.text-left img",                 // Text content images
            "#chapter-content img",              // Chapter content by ID
            ".entry-content img"                 // Entry content images
        ];

        for (_selector_idx, selector) in image_selectors.iter().enumerate() {
            
            if let Some(images) = html.select(selector) {
                
                for (_idx, img) in images.into_iter().enumerate() {
                    let img_url = self.get_image_url(&img);
                    
                    if !img_url.is_empty() {
                        pages.push(Page {
                            content: PageContent::Url(img_url, None),
                            thumbnail: None,
                            has_description: false,
                            description: None,
                        });
                    } else {
                    }
                }
                
                
                if !pages.is_empty() {
                    break;
                }
            } else {
            }
        }
        
        if pages.is_empty() {
            // Debug: print some page content to understand structure
            if let Some(body) = html.select("body").and_then(|b| b.first()) {
                if let Some(body_html) = body.html() {
                    let _preview = if body_html.len() > 500 {
                        &body_html[..500]
                    } else {
                        &body_html
                    };
                }
            }
        }

        Ok(pages)
    }
    
    // Helper function similar to Madara template's get_image_url
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

    fn extract_manga_id(&self, url: &str) -> String {
        url.trim_end_matches('/')
            .split('/')
            .last()
            .unwrap_or("unknown")
            .to_string()
    }
}

register_source!(MangaScantrad, ListingProvider, ImageRequestProvider);