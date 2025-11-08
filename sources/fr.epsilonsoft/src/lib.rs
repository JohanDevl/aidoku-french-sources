#![no_std]

use aidoku::{
    AidokuError, Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider,
    Manga, MangaPageResult, MangaStatus, Page, PageContent, PageContext, Result, Source,
    UpdateStrategy, Viewer,
    alloc::{String, Vec, vec},
    imports::{net::Request, html::Document, std::send_partial_result},
    prelude::*,
};

extern crate alloc;
use alloc::string::ToString;

pub static BASE_URL: &str = "https://epsilonsoft.to";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

const MAX_RETRIES: u32 = 3;

fn calculate_content_rating(tags: &[String]) -> ContentRating {
    if tags.iter().any(|tag| {
        let lower = tag.to_lowercase();
        matches!(lower.as_str(), "ecchi" | "mature" | "adult" | "hentai" | "smut" | "manhwa r")
    }) {
        ContentRating::NSFW
    } else if tags.iter().any(|tag| {
        let lower = tag.to_lowercase();
        matches!(lower.as_str(), "bl soft" | "boys love" | "yaoi")
    }) {
        ContentRating::Suggestive
    } else {
        ContentRating::Safe
    }
}

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
    let mut attempt = 0;
    loop {
        let request = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html();

        match request {
            Ok(doc) => return Ok(doc),
            Err(e) => {
                if attempt >= MAX_RETRIES {
                    return Err(AidokuError::RequestError(e));
                }
                attempt += 1;
            }
        }
    }
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

pub struct EpsilonSoft;

impl Source for EpsilonSoft {
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

    fn get_manga_update(&self, manga: Manga, needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let url = manga.url.unwrap_or_else(|| format!("{}/manga/{}/", BASE_URL, manga.key));
        let html = create_html_request(&url)?;
        self.parse_manga_details(html, manga.key, needs_details, needs_chapters)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = chapter.url.unwrap_or_else(|| format!("{}/{}", BASE_URL, chapter.key));
        let html = create_html_request(&url)?;
        self.parse_page_list(html)
    }
}

impl ListingProvider for EpsilonSoft {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let mut url_params: Vec<String> = vec![String::from("post_type=wp-manga")];

        if page > 1 {
            url_params.push(format!("paged={}", page));
        }

        match listing.name.as_str() {
            "Populaire" => url_params.push(String::from("m_orderby=views")),
            "Récent" => url_params.push(String::from("m_orderby=latest")),
            "Nouveauté" => url_params.push(String::from("m_orderby=new-manga")),
            _ => {}
        };

        let url = format!("{}/?{}", BASE_URL, url_params.join("&"));
        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for EpsilonSoft {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl EpsilonSoft {
    fn build_search_url(&self, query: Option<String>, page: i32, filters: Vec<FilterValue>) -> String {
        let mut url_params: Vec<String> = Vec::new();
        let mut statuses: Vec<String> = Vec::new();
        let mut orderby_val = String::new();
        let mut genres: Vec<String> = Vec::new();
        let mut condition_val = String::new();

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    match id.as_str() {
                        "orderby" => orderby_val = value,
                        "condition" => condition_val = value,
                        _ => {}
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    match id.as_str() {
                        "genre" => genres = included,
                        "status" => statuses = included,
                        _ => {}
                    }
                }
                _ => {}
            }
        }

        // Add search query if present
        if let Some(search_query) = query {
            if !search_query.is_empty() {
                url_params.push(format!("s={}", urlencode(&search_query)));
            }
        }

        // Always add post_type=wp-manga
        url_params.push(String::from("post_type=wp-manga"));

        // Add genres
        for genre in genres {
            if !genre.is_empty() {
                url_params.push(format!("genre[]={}", genre));
            }
        }

        // Add condition (op parameter) - empty for OR, "1" for AND
        if !condition_val.is_empty() {
            url_params.push(format!("op={}", condition_val));
        }

        // Add statuses
        for status in statuses {
            if !status.is_empty() {
                url_params.push(format!("status[]={}", status));
            }
        }

        // Add orderby
        if !orderby_val.is_empty() {
            url_params.push(format!("m_orderby={}", orderby_val));
        }

        // Add pagination
        if page > 1 {
            url_params.push(format!("paged={}", page));
        }

        format!("{}/?{}", BASE_URL, url_params.join("&"))
    }

    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = create_html_request(url)?;
        self.parse_manga_list(html)
    }

    fn parse_manga_list(&self, html: Document) -> Result<MangaPageResult> {
        let mut mangas: Vec<Manga> = Vec::new();

        // Select all manga items
        if let Some(items) = html.select(".page-item-detail.manga") {
            for item in items {
                // Get title and URL from .post-title h3 a
                let title_elem = item.select(".post-title h3 a, .post-title a")
                    .and_then(|elems| elems.first());

                if let Some(elem) = title_elem {
                    let title = elem.text().unwrap_or_default().trim().to_string();
                    let href = elem.attr("href").unwrap_or_default();

                    if title.is_empty() || href.is_empty() {
                        continue;
                    }

                    let url_abs = make_absolute_url(BASE_URL, &href);

                    // Extract manga key from URL (e.g., "manga/dechu" from "/manga/dechu/")
                    let key = href
                        .trim_start_matches("https://epsilonsoft.to/")
                        .trim_start_matches("http://epsilonsoft.to/")
                        .trim_start_matches('/')
                        .trim_end_matches('/')
                        .to_string();

                    // Get cover image from .item-thumb img
                    let mut cover = None;
                    if let Some(img_elem) = item.select(".item-thumb img").and_then(|imgs| imgs.first()) {
                        let img_url = img_elem.attr("src")
                            .or_else(|| img_elem.attr("data-src"))
                            .or_else(|| img_elem.attr("data-lazy-src"))
                            .unwrap_or_default();
                        if !img_url.is_empty() {
                            cover = Some(make_absolute_url(BASE_URL, &img_url));
                        }
                    }

                    mangas.push(Manga {
                        key,
                        title,
                        cover,
                        authors: None,
                        artists: None,
                        description: None,
                        url: Some(url_abs),
                        tags: None,
                        status: MangaStatus::Unknown,
                        content_rating: ContentRating::Safe,
                        viewer: Viewer::default(),
                        chapters: None,
                        next_update_time: None,
                        update_strategy: UpdateStrategy::Never,
                    });
                }
            }
        }

        // Check for pagination
        let has_more = html.select("a.next, .pagination .next, .nav-previous")
            .and_then(|elems| elems.first())
            .is_some();

        Ok(MangaPageResult {
            entries: mangas,
            has_next_page: has_more,
        })
    }

    fn parse_manga_details(&self, html: Document, key: String, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let title = html.select("div.post-title h1, h1.entry-title, .wp-manga-title")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| key.clone());

        let cover_selectors = [
            "div.summary_image img",
            ".wp-post-image",
            ".manga-poster img",
        ];

        let mut cover = None;
        for selector in &cover_selectors {
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                let img_url = img_elem.attr("data-src")
                    .or_else(|| img_elem.attr("data-lazy-src"))
                    .or_else(|| img_elem.attr("src"))
                    .unwrap_or_default();
                if !img_url.is_empty() {
                    cover = Some(make_absolute_url(BASE_URL, &img_url));
                    break;
                }
            }
        }

        let author_selectors = [
            "div.author-content a",
            "div.manga-authors a",
        ];

        let mut authors = None;
        for selector in &author_selectors {
            if let Some(author_elem) = html.select(selector).and_then(|elems| elems.first()) {
                let author_text = author_elem.text().unwrap_or_default().trim().to_string();
                if !author_text.is_empty() {
                    authors = Some(vec![author_text]);
                    break;
                }
            }
        }

        let description_selectors = [
            "div.description-summary div.summary__content",
            "div.summary_content div.post-content_item > h5 + div",
            "div.summary__content",
        ];

        let mut description = None;
        for selector in &description_selectors {
            if let Some(desc_elem) = html.select(selector).and_then(|elems| elems.first()) {
                let desc_text = desc_elem.text().unwrap_or_default().trim().to_string();
                if !desc_text.is_empty() && desc_text.len() > 10 {
                    description = Some(desc_text);
                    break;
                }
            }
        }

        let status_selectors = [
            "div.summary-heading:contains(Statut) + div.summary-content",
            "div.summary-heading:contains(Status) + div.summary-content",
            ".post-status .summary-content",
        ];

        let mut status = MangaStatus::Unknown;
        for selector in &status_selectors {
            if let Some(status_elem) = html.select(selector).and_then(|elems| elems.first()) {
                let status_text = status_elem.text().unwrap_or_default().trim().to_lowercase();
                status = match status_text.as_str() {
                    s if s.contains("en cours") || s.contains("ongoing") => MangaStatus::Ongoing,
                    s if s.contains("terminé") || s.contains("completed") || s.contains("achevé") => MangaStatus::Completed,
                    s if s.contains("pause") || s.contains("hiatus") => MangaStatus::Hiatus,
                    s if s.contains("abandonné") || s.contains("cancelled") => MangaStatus::Cancelled,
                    _ => MangaStatus::Unknown,
                };
                if status != MangaStatus::Unknown {
                    break;
                }
            }
        }

        let mut tags: Vec<String> = Vec::new();
        if let Some(genre_items) = html.select("div.genres-content a, .wp-manga-genres a") {
            for genre in genre_items {
                let genre_text = genre.text().unwrap_or_default().trim().to_string();
                if !genre_text.is_empty() && !tags.contains(&genre_text) {
                    tags.push(genre_text);
                }
            }
        }

        let content_rating = calculate_content_rating(&tags);
        let viewer = calculate_viewer(&tags);

        let mut manga = Manga {
            key: key.clone(),
            title,
            cover,
            authors,
            artists: None,
            description,
            url: Some(format!("{}/manga/{}/", BASE_URL, key)),
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
            let ajax_url = format!("{}/manga/{}/ajax/chapters", BASE_URL, key);
            let chapters_html = Request::post(&ajax_url)?
                .header("User-Agent", USER_AGENT)
                .header("Accept", "*/*")
                .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
                .header("X-Requested-With", "XMLHttpRequest")
                .header("Origin", BASE_URL)
                .header("Referer", &manga.url.clone().unwrap_or_default())
                .html();

            let chapters = if let Ok(ch_html) = chapters_html {
                self.parse_chapter_list(&ch_html)?
            } else {
                self.parse_chapter_list(&html)?
            };

            manga.chapters = Some(chapters);
        }

        Ok(manga)
    }

    fn parse_chapter_list(&self, html: &Document) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();

        let chapter_selectors = [
            "li.wp-manga-chapter",
            "#chapterlist li",
            ".manga-chapters li",
        ];

        for selector in &chapter_selectors {
            if let Some(items) = html.select(selector) {
                for item in items {
                    let link = item.select("a").and_then(|elems| elems.first());

                    if let Some(link_elem) = link {
                        let href = link_elem.attr("href").unwrap_or_default();
                        if href.is_empty() {
                            continue;
                        }

                        let title = link_elem.text().unwrap_or_default().trim().to_string();
                        if title.is_empty() {
                            continue;
                        }

                        let chapter_key = href.trim_start_matches('/').trim_end_matches('/').to_string();
                        let chapter_number = self.extract_chapter_number(&title);

                        let mut url_with_style = make_absolute_url(BASE_URL, &href);
                        if !url_with_style.contains("?style=") {
                            url_with_style = format!("{}?style=list", url_with_style);
                        }

                        let date_selectors = [
                            "span.chapter-release-date",
                            "img:not(.thumb)",
                        ];

                        let mut date_uploaded = None;
                        for date_sel in &date_selectors {
                            if let Some(date_elem) = item.select(date_sel).and_then(|elems| elems.first()) {
                                let date_text = if date_sel.contains("img") {
                                    date_elem.attr("alt").unwrap_or_default()
                                } else {
                                    date_elem.text().unwrap_or_default()
                                };

                                if !date_text.is_empty() {
                                    date_uploaded = self.parse_chapter_date(&date_text);
                                    break;
                                }
                            }
                        }

                        chapters.push(Chapter {
                            key: chapter_key,
                            title: Some(title),
                            chapter_number: Some(chapter_number),
                            volume_number: None,
                            date_uploaded,
                            scanlators: None,
                            url: Some(url_with_style),
                            language: Some(String::from("fr")),
                            thumbnail: None,
                            locked: false,
                        });
                    }
                }
                if !chapters.is_empty() {
                    break;
                }
            }
        }

        Ok(chapters)
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        let selectors = [
            "div.page-break img",
            ".reading-content img",
            "#readerarea img",
        ];

        for selector in &selectors {
            if let Some(images) = html.select(selector) {
                for img_element in images {
                    let img_url = img_element.attr("data-src")
                        .or_else(|| img_element.attr("data-lazy-src"))
                        .or_else(|| img_element.attr("src"))
                        .unwrap_or_default();

                    if !img_url.is_empty() && !img_url.starts_with("data:") {
                        let absolute_url = make_absolute_url(BASE_URL, &img_url);
                        pages.push(Page {
                            content: PageContent::Url(absolute_url, None),
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

        if cleaned.contains("ago") || cleaned.contains("il y a") || cleaned.contains("heures") || cleaned.contains("jours") {
            let parts: Vec<&str> = cleaned.split_whitespace().collect();
            if parts.len() >= 2 {
                if let Ok(value) = parts[0].parse::<i64>() {
                    let unit = parts[1];
                    let seconds = if unit.contains("second") || unit.contains("seconde") {
                        value
                    } else if unit.contains("minute") {
                        value * 60
                    } else if unit.contains("hour") || unit.contains("heure") {
                        value * 3600
                    } else if unit.contains("day") || unit.contains("jour") {
                        value * 86400
                    } else if unit.contains("week") || unit.contains("semaine") {
                        value * 604800
                    } else if unit.contains("month") || unit.contains("mois") {
                        value * 2592000
                    } else {
                        0
                    };

                    if seconds > 0 {
                        let current_time = aidoku::imports::std::current_date();
                        return Some(current_time - seconds);
                    }
                }
            }
        }

        for separator in &['/', '-'] {
            if cleaned.contains(*separator) {
                let date_parts: Vec<&str> = cleaned.split(*separator).collect();
                if date_parts.len() == 3 {
                    if let (Ok(day), Ok(month), Ok(year)) = (
                        date_parts[0].parse::<i32>(),
                        date_parts[1].parse::<i32>(),
                        date_parts[2].parse::<i32>(),
                    ) {
                        if day >= 1 && day <= 31 && month >= 1 && month <= 12 {
                            let full_year = if year < 100 { year + 2000 } else { year };
                            return Some(self.calculate_timestamp(full_year, month, day));
                        }
                    }
                }
            }
        }

        None
    }

    fn calculate_timestamp(&self, year: i32, month: i32, day: i32) -> i64 {
        let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
        let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

        let mut total_days = 0i64;

        for y in 1970..year {
            if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
                total_days += 366;
            } else {
                total_days += 365;
            }
        }

        for m in 1..month {
            let days = days_in_month[m as usize - 1];
            total_days += if m == 2 && is_leap_year {
                29
            } else {
                days as i64
            };
        }

        total_days += day as i64 - 1;
        total_days * 86400
    }
}

register_source!(EpsilonSoft, ListingProvider, ImageRequestProvider);
