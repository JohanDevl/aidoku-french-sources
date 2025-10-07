#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
    MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString};

pub static BASE_URL: &str = "https://starboundscans.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36";

pub struct StarBoundScans;

impl Source for StarBoundScans {
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
        let url = if manga.key.starts_with("http") {
            manga.key.clone()
        } else {
            format!("{}/{}/", BASE_URL, manga.key.trim_start_matches('/'))
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_details(html, manga.key, _needs_details, needs_chapters)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = if chapter.key.starts_with("http") {
            chapter.key.clone()
        } else {
            format!("{}/{}", BASE_URL, chapter.key.trim_start_matches('/'))
        };

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_page_list(html)
    }
}

impl ListingProvider for StarBoundScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        let url = match listing.name.as_str() {
            "Populaire" => format!("{}/manga/page/{}/?m_orderby=views", BASE_URL, page),
            "Dernières" => format!("{}/manga/page/{}/?m_orderby=latest", BASE_URL, page),
            _ => format!("{}/manga/page/{}/", BASE_URL, page),
        };

        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for StarBoundScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl StarBoundScans {
    fn build_search_url(&self, query: Option<String>, page: i32, filters: Vec<FilterValue>) -> String {
        let mut params: Vec<String> = Vec::new();
        let mut manga_type = String::new();
        let mut status = String::new();

        for filter in filters {
            match filter {
                FilterValue::Select { id, value } => {
                    match id.as_str() {
                        "type" => {
                            if !value.is_empty() {
                                manga_type = value;
                            }
                        },
                        "status" => {
                            if !value.is_empty() {
                                status = value;
                            }
                        },
                        _ => {}
                    }
                },
                _ => {}
            }
        }

        if let Some(search_query) = query {
            if !search_query.is_empty() {
                params.push(format!("s={}", search_query.replace(' ', "+")));
            }
        }

        if !manga_type.is_empty() {
            params.push(format!("type[]={}", manga_type));
        }

        if !status.is_empty() {
            params.push(format!("status[]={}", status));
        }

        if params.is_empty() {
            format!("{}/manga/page/{}/", BASE_URL, page)
        } else {
            params.push("post_type=wp-manga".to_string());
            params.push(format!("page={}", page));
            format!("{}/?{}", BASE_URL, params.join("&"))
        }
    }

    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        let mut entries: Vec<Manga> = Vec::new();

        let manga_selectors = [
            ".page-item-detail",
            ".manga-item",
            ".c-tabs-item__content",
            "div.row.c-tabs-item__content",
        ];

        for selector in &manga_selectors {
            if let Some(items) = html.select(selector) {
                for item in items {
                    let href = item.select(".item-thumb a, a")
                        .and_then(|links| links.first())
                        .and_then(|link| link.attr("href"))
                        .unwrap_or_default();

                    if href.is_empty() {
                        continue;
                    }

                    let key = href
                        .replace(BASE_URL, "")
                        .trim_start_matches('/')
                        .trim_end_matches('/')
                        .to_string();

                    if key.is_empty() {
                        continue;
                    }

                    let title = item.select(".post-title h3 a, h3 a, .item-summary h3 a, h5 a")
                        .and_then(|elems| elems.first())
                        .and_then(|elem| elem.text())
                        .or_else(|| {
                            item.select(".item-thumb a, a")
                                .and_then(|links| links.first())
                                .and_then(|link| link.attr("title"))
                        })
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    if title.is_empty() {
                        continue;
                    }

                    let cover = item.select(".item-thumb img, img")
                        .and_then(|imgs| imgs.first())
                        .and_then(|img| {
                            img.attr("src")
                                .or_else(|| img.attr("data-src"))
                                .or_else(|| img.attr("data-lazy-src"))
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

                if !entries.is_empty() {
                    break;
                }
            }
        }

        let has_next_page = html.select(".nav-previous, .next.page-numbers, a.next").is_some();

        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }

    fn parse_manga_details(&self, html: Document, key: String, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let title = html.select("h1")
            .and_then(|elems| elems.first())
            .and_then(|elem| elem.text())
            .map(|text| text.trim().to_string())
            .unwrap_or_else(|| key.clone());

        let cover_selectors = [
            "img.project-cover",
            ".project-cover",
            ".summary_image img",
            ".tab-summary img",
            "div.summary_image img",
        ];

        let mut cover = String::new();
        for selector in &cover_selectors {
            if let Some(img_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(src) = img_elem.attr("src")
                    .or_else(|| img_elem.attr("data-src"))
                    .or_else(|| img_elem.attr("data-lazy-src")) {
                    if !src.is_empty() {
                        cover = src.to_string();
                        break;
                    }
                }
            }
        }

        let description_selectors = [
            ".card-body .black-orion-article-content p",
            ".summary__content p",
            ".description-summary p",
            "div[itemprop=description] p",
        ];

        let mut description = None;
        for selector in &description_selectors {
            if let Some(desc_elems) = html.select(selector) {
                let mut desc_parts: Vec<String> = Vec::new();
                for desc_elem in desc_elems {
                    if let Some(text) = desc_elem.text() {
                        let trimmed = text.trim();
                        if !trimmed.is_empty() {
                            desc_parts.push(trimmed.to_string());
                        }
                    }
                }
                if !desc_parts.is_empty() {
                    description = Some(desc_parts.join("\n"));
                    break;
                }
            }
        }

        let status_selectors = [
            ".info-item:contains(Statut) .info-value",
            ".post-status .summary-content",
            ".post-content_item:contains(Statut) .summary-content",
        ];

        let mut status = MangaStatus::Unknown;
        for selector in &status_selectors {
            if let Some(status_elem) = html.select(selector).and_then(|elems| elems.first()) {
                if let Some(status_text) = status_elem.text() {
                    let status_str = status_text.trim().to_lowercase();

                    status = match status_str.as_str() {
                        s if s.contains("en cours") || s.contains("ongoing") => MangaStatus::Ongoing,
                        s if s.contains("terminé") || s.contains("termine") || s.contains("completed") => MangaStatus::Completed,
                        s if s.contains("abandonné") || s.contains("abandonne") || s.contains("cancelled") => MangaStatus::Cancelled,
                        s if s.contains("pause") || s.contains("hiatus") => MangaStatus::Hiatus,
                        _ => MangaStatus::Unknown,
                    };

                    if status != MangaStatus::Unknown {
                        break;
                    }
                }
            }
        }

        let author_selectors = [
            ".info-item:contains(Auteur) a",
            ".author-content a",
            ".artist-content a",
        ];

        let mut author = None;
        for selector in &author_selectors {
            if let Some(author_elems) = html.select(selector) {
                let mut authors: Vec<String> = Vec::new();
                for elem in author_elems {
                    if let Some(text) = elem.text() {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            authors.push(trimmed);
                        }
                    }
                }
                if !authors.is_empty() {
                    author = Some(authors);
                    break;
                }
            }
        }

        let genre_selectors = [
            ".info-item:contains(Genres) .genre-tag",
            ".genres-content a",
            ".wp-manga-tags-list a",
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

        let mut manga = Manga {
            key: key.clone(),
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            authors: author,
            artists: None,
            description,
            url: Some(format!("{}/{}/", BASE_URL, key.trim_start_matches('/'))),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status,
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        };

        if needs_chapters {
            manga.chapters = Some(self.parse_chapter_list(&html)?);
        }

        Ok(manga)
    }

    fn parse_chapter_list(&self, html: &Document) -> Result<Vec<Chapter>> {
        let mut chapters: Vec<Chapter> = Vec::new();

        let chapter_selectors = [
            ".chapter-item",
            "li.wp-manga-chapter",
            ".version-chap li",
        ];

        for selector in &chapter_selectors {
            if let Some(items) = html.select(selector) {
                let items_vec: Vec<_> = items.collect();
                if !items_vec.is_empty() {
                    for item in items_vec {
                        let link = if let Some(a_element) = item.select("a.chapter-link, a") {
                            if let Some(first_link) = a_element.first() {
                                first_link
                            } else {
                                continue;
                            }
                        } else {
                            continue;
                        };

                        let href = link.attr("href").unwrap_or_default();
                        if href.is_empty() || href == "#" {
                            continue;
                        }

                        let title_elem = item.select("h3, .chapter-title, span.chapter-title")
                            .and_then(|elems| elems.first());

                        let raw_title = if let Some(elem) = title_elem {
                            elem.text().unwrap_or_default()
                        } else {
                            link.text().unwrap_or_default()
                        };

                        let title_lower = raw_title.to_lowercase();
                        if title_lower.contains("vip") || title_lower.contains("réservé") || title_lower.contains("reserve") {
                            continue;
                        }

                        let title = raw_title.replace("NEW", "").trim().to_string();

                        if title.is_empty() {
                            continue;
                        }

                        let mut chapter_key = href
                            .replace(BASE_URL, "")
                            .trim_start_matches('/')
                            .trim_end_matches('/')
                            .to_string();

                        if !chapter_key.contains("?style=list") {
                            if chapter_key.contains('?') {
                                chapter_key.push_str("&style=list");
                            } else {
                                chapter_key.push_str("?style=list");
                            }
                        }

                        let chapter_number = self.extract_chapter_number(&title);

                        let date_selectors = [
                            ".chapter-meta",
                            ".chapter-release-date",
                            "span.chapter-release-date",
                        ];

                        let mut date_uploaded = None;
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

                        let url = if href.starts_with("http") {
                            if href.contains("?style=list") {
                                href
                            } else if href.contains('?') {
                                format!("{}&style=list", href)
                            } else {
                                format!("{}?style=list", href)
                            }
                        } else {
                            format!("{}/{}", BASE_URL, chapter_key)
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
                    break;
                }
            }
        }

        Ok(chapters)
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        let selectors = [
            "img.wp-manga-chapter-img",
            ".reading-content img",
            "div.page-break img",
            "#readerarea img",
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
            let after_ch = &title[pos + 8..].trim();
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
                    let days_since_epoch = self.calculate_days_since_epoch(year, month, day);
                    return Some(days_since_epoch as i64 * 86400);
                }
            }
        }

        None
    }

    fn calculate_days_since_epoch(&self, year: i32, month: u32, day: u32) -> i32 {
        let days_before_month = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];

        let years_since_epoch = year - 1970;

        let leap_days = (1970..year).filter(|&y| (y % 4 == 0 && y % 100 != 0) || y % 400 == 0).count() as i32;

        let mut days = years_since_epoch * 365 + leap_days;

        days += days_before_month[(month - 1) as usize];

        if ((year % 4 == 0 && year % 100 != 0) || year % 400 == 0) && month > 2 {
            days += 1;
        }

        days + (day - 1) as i32
    }
}

register_source!(StarBoundScans, ListingProvider, ImageRequestProvider);
