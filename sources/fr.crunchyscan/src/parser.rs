use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter, UpdateStrategy,
    ContentRating, Viewer,
    alloc::{String, Vec, string::ToString, vec},
    imports::html::Document,
    prelude::*,
    AidokuError,
};
use core::cmp::Ordering;
use crate::helper;
use crate::BASE_URL;

// Parse manga list from HTML catalog page (legacy, kept for reference)
#[allow(dead_code)]
pub fn parse_manga_list(html: &Document, _page: i32) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    let mut seen_keys: Vec<String> = Vec::new();

    if let Some(links) = html.select("a") {
        for link in links {
            let href = link.attr("href").unwrap_or_default();

            // Only process manga links (not chapter links)
            if !href.contains("/lecture-en-ligne/") || href.contains("/read/") {
                continue;
            }

            let key = href
                .replace(BASE_URL, "")
                .replace("/lecture-en-ligne/", "")
                .trim_start_matches('/')
                .trim_end_matches('/')
                .to_string();

            if key.is_empty() || seen_keys.contains(&key) {
                continue;
            }

            // Get title from link text or title attribute
            let title_text = link.text().unwrap_or_default();
            let title_attr = link.attr("title").unwrap_or_default();

            let title = if !title_text.trim().is_empty() && !title_text.contains("Chapitre") {
                title_text.trim().to_string()
            } else {
                title_attr
                    .replace("Lire le manga ", "")
                    .trim()
                    .to_string()
            };

            if title.is_empty() {
                continue;
            }

            // Try to get cover from img inside link or sibling
            let cover = link.select("img")
                .and_then(|imgs| imgs.first())
                .and_then(|img| {
                    img.attr("src")
                        .or_else(|| img.attr("data-src"))
                })
                .map(|s| s.to_string());

            seen_keys.push(key.clone());
            mangas.push(Manga {
                key,
                title,
                cover,
                authors: None,
                artists: None,
                description: None,
                tags: None,
                url: Some(href.to_string()),
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::RightToLeft,
                chapters: None,
                next_update_time: None,
                update_strategy: UpdateStrategy::Never,
            });
        }
    }

    let has_next_page = html.select("a.next, button.next").is_some();

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page,
    })
}

// Parse manga details from HTML
pub fn parse_manga_details(html: &Document, key: &str) -> Result<Manga> {
    let mut title = String::new();
    let mut cover = String::new();
    let mut description = String::new();
    let mut tags: Vec<String> = Vec::new();
    let mut status = MangaStatus::Unknown;
    let mut authors: Option<Vec<String>> = None;

    // Extract title
    let title_selectors = [
        "h1.entry-title",
        ".manga-title",
        ".series-title",
        "h1",
    ];
    for selector in &title_selectors {
        if let Some(elem) = html.select(selector).and_then(|els| els.first()) {
            if let Some(text) = elem.text() {
                title = text.trim().to_string();
                if !title.is_empty() {
                    break;
                }
            }
        }
    }

    // Extract cover
    let cover_selectors = [
        "img[class*='cover']",
        ".manga-cover img",
        ".series-image img",
        "img[alt*='cover']",
        ".thumb img",
    ];
    for selector in &cover_selectors {
        if let Some(img) = html.select(selector).and_then(|els| els.first()) {
            if let Some(src) = img.attr("data-src")
                .or_else(|| img.attr("data-lazy-src"))
                .or_else(|| img.attr("src")) {
                cover = if src.starts_with("http") {
                    src.to_string()
                } else {
                    format!("{}/{}", BASE_URL, src.trim_start_matches('/'))
                };
                if !cover.is_empty() {
                    break;
                }
            }
        }
    }

    // Extract description
    let desc_selectors = [
        ".description",
        ".synopsis",
        "[class*='desc']",
        ".manga-description",
        ".summary",
    ];
    for selector in &desc_selectors {
        if let Some(elem) = html.select(selector).and_then(|els| els.first()) {
            if let Some(text) = elem.text() {
                description = text.trim().to_string();
                if !description.is_empty() {
                    break;
                }
            }
        }
    }

    // Extract genres/tags
    let genre_selectors = [
        ".genre",
        ".tag",
        "[class*='genre'] a",
        ".genres a",
    ];
    for selector in &genre_selectors {
        if let Some(els) = html.select(selector) {
            for elem in els {
                if let Some(text) = elem.text() {
                    let tag = text.trim().to_string();
                    if !tag.is_empty() {
                        tags.push(tag);
                    }
                }
            }
            if !tags.is_empty() {
                break;
            }
        }
    }

    // Extract status
    let status_selectors = [
        ".status",
        "[class*='status']",
        ".manga-status",
    ];
    for selector in &status_selectors {
        if let Some(elem) = html.select(selector).and_then(|els| els.first()) {
            if let Some(text) = elem.text() {
                let status_str = text.to_lowercase();
                status = if status_str.contains("en cours") || status_str.contains("ongoing") {
                    MangaStatus::Ongoing
                } else if status_str.contains("terminé") || status_str.contains("completed") {
                    MangaStatus::Completed
                } else if status_str.contains("abandonné") || status_str.contains("cancelled") {
                    MangaStatus::Cancelled
                } else if status_str.contains("en pause") || status_str.contains("hiatus") {
                    MangaStatus::Hiatus
                } else {
                    MangaStatus::Unknown
                };
                break;
            }
        }
    }

    // Extract author
    let author_selectors = [
        ".author-content",
        "[class*='author']",
        ".manga-author",
    ];
    for selector in &author_selectors {
        if let Some(elem) = html.select(selector).and_then(|els| els.first()) {
            if let Some(text) = elem.text() {
                let author_str = text.trim().to_string();
                if !author_str.is_empty() {
                    authors = Some(vec![author_str]);
                    break;
                }
            }
        }
    }

    Ok(Manga {
        key: key.to_string(),
        title,
        cover: if cover.is_empty() { None } else { Some(cover) },
        authors,
        artists: None,
        description: if description.is_empty() { None } else { Some(description) },
        tags: if tags.is_empty() { None } else { Some(tags) },
        url: Some(format!("{}/lecture-en-ligne/{}", BASE_URL, key)),
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::RightToLeft,
        chapters: None,
        next_update_time: None,
        update_strategy: UpdateStrategy::Never,
    })
}

// Parse chapter list from HTML
pub fn parse_chapter_list(html: &Document) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();
    let mut seen_urls: Vec<String> = Vec::new();

    // Try to find chapter links
    let chapter_selectors = [
        "a[href*='/read/']",
        ".chapter-link",
        ".chapter a",
        "li a[href*='chapitre']",
    ];

    for selector in &chapter_selectors {
        if let Some(links) = html.select(selector) {
            for link in links {
                if let Some(href) = link.attr("href") {
                    let url = if href.starts_with("http") {
                        href.to_string()
                    } else if href.starts_with("/") {
                        format!("{}{}", BASE_URL, href)
                    } else {
                        format!("{}/{}", BASE_URL, href)
                    };

                    // Skip duplicates
                    if seen_urls.contains(&url) {
                        continue;
                    }

                    // Extract chapter number
                    let chapter_text = link.text().unwrap_or_default();

                    // Skip chapters with time info in title (duplicates from other sections)
                    let chapter_text_lower = chapter_text.to_lowercase();
                    if chapter_text_lower.contains("heures") || chapter_text_lower.contains("heure") ||
                       chapter_text_lower.contains("jours") || chapter_text_lower.contains("jour") ||
                       chapter_text_lower.contains("années") || chapter_text_lower.contains("année") ||
                       chapter_text_lower.contains("mois") || chapter_text_lower.contains("semaines") ||
                       chapter_text_lower.contains("semaine") || chapter_text_lower.contains("minutes") ||
                       chapter_text_lower.contains("minute") {
                        continue;
                    }

                    let chapter_number = helper::extract_chapter_number(&chapter_text);

                    seen_urls.push(url.clone());
                    chapters.push(Chapter {
                        key: url.clone(),
                        title: Some(chapter_text.trim().to_string()),
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
            }

            if !chapters.is_empty() {
                break;
            }
        }
    }

    // Sort chapters by number (descending)
    chapters.sort_by(|a, b| {
        let a_num = a.chapter_number.unwrap_or(0.0);
        let b_num = b.chapter_number.unwrap_or(0.0);
        b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal)
    });

    Ok(chapters)
}

// Parse page list from chapter HTML
pub fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();

    // Try multiple selectors for images
    let image_selectors = [
        "img[class*='page']",
        ".page img",
        "#reader img",
        ".reader-container img",
        "[class*='reader'] img",
    ];

    for selector in &image_selectors {
        if let Some(imgs) = html.select(selector) {
            for img in imgs {
                if let Some(src) = img.attr("data-src")
                    .or_else(|| img.attr("data-lazy-src"))
                    .or_else(|| img.attr("src")) {

                    let url = if src.starts_with("http") {
                        src.to_string()
                    } else {
                        format!("{}/{}", BASE_URL, src.trim_start_matches('/'))
                    };

                    pages.push(Page {
                        content: PageContent::Url(url, None),
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

// Parse manga list from JSON API response
pub fn parse_manga_list_json(data: &str, _page: i32) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();

    // Parse JSON
    let json: serde_json::Value = serde_json::from_str(data)
        .map_err(|_| AidokuError::JsonParseError)?;

    // Get data array from JSON
    let data_array = json.get("data")
        .and_then(|v| v.as_array())
        .ok_or(AidokuError::JsonParseError)?;

    for item in data_array {
        let obj = item.as_object().ok_or(AidokuError::JsonParseError)?;

        // Extract fields
        let name = obj.get("name")
            .and_then(|v| v.as_str())
            .ok_or(AidokuError::JsonParseError)?;
        let slug = obj.get("slug")
            .and_then(|v| v.as_str())
            .ok_or(AidokuError::JsonParseError)?;
        let cover_url = obj.get("cover_url")
            .and_then(|v| v.as_str());
        let synopsis = obj.get("synopsis")
            .and_then(|v| v.as_str());

        mangas.push(Manga {
            key: slug.to_string(),
            title: name.to_string(),
            cover: cover_url.map(|s| s.to_string()),
            authors: None,
            artists: None,
            description: synopsis.map(|s| s.to_string()),
            tags: None,
            url: Some(format!("{}/lecture-en-ligne/{}", BASE_URL, slug)),
            status: MangaStatus::Unknown,
            content_rating: ContentRating::Safe,
            viewer: Viewer::RightToLeft,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        });
    }

    // Check for next page - if we have 24 items, there might be more
    let has_next_page = data_array.len() >= 24;

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page,
    })
}
