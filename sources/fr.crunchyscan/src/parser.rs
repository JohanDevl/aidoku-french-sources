use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter, UpdateStrategy,
    ContentRating, Viewer,
    alloc::{String, Vec, string::ToString},
    imports::html::Document,
    prelude::*,
};
extern crate serde_json;
use core::cmp::Ordering;
extern crate alloc;
use crate::helper;
use crate::BASE_URL;

// Parse JSON API response
pub fn parse_api_manga_list(json_str: &str, page: i32) -> Result<MangaPageResult> {
    let json: serde_json::Value = serde_json::from_str(json_str)
        .map_err(|_| aidoku::AidokuError::JsonParseError)?;
    let mut mangas: Vec<Manga> = Vec::new();

    if let Some(data) = json.get("data").and_then(|d| d.as_array()) {
        for item in data {
            let slug = item.get("slug")
                .and_then(|s| s.as_str())
                .unwrap_or("")
                .to_string();

            let title = item.get("title")
                .and_then(|t| t.as_str())
                .unwrap_or("")
                .to_string();

            let cover = if let Some(cover_path) = item.get("cover_path").and_then(|c| c.as_str()) {
                if cover_path.starts_with("http") {
                    Some(cover_path.to_string())
                } else {
                    Some(format!("{}/{}", BASE_URL, cover_path.trim_start_matches('/')))
                }
            } else {
                None
            };

            mangas.push(Manga {
                key: slug,
                title,
                cover,
                authors: None,
                artists: None,
                description: None,
                tags: None,
                url: None,
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::RightToLeft,
                chapters: None,
                next_update_time: None,
                update_strategy: UpdateStrategy::Never,
            });
        }
    }

    let has_next_page = if let Some(last_page) = json.get("last_page").and_then(|l| l.as_i64()) {
        page < last_page as i32
    } else {
        false
    };

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

    Ok(Manga {
        key: key.to_string(),
        title,
        cover: if cover.is_empty() { None } else { Some(cover) },
        authors: None,
        artists: None,
        description: if description.is_empty() { None } else { Some(description) },
        tags: if tags.is_empty() { None } else { Some(tags) },
        url: Some(helper::build_manga_url(key)),
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::RightToLeft,
        chapters: None,
        next_update_time: None,
        update_strategy: UpdateStrategy::Never,
    })
}

// Parse chapter list from HTML
pub fn parse_chapter_list(html: &Document, _manga_key: &str) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();

    // Try to find chapter links
    let chapter_selectors = [
        "a[href*='/read/']",
        ".chapter-link",
        ".chapter a",
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

                    // Extract chapter number
                    let chapter_text = link.text().unwrap_or_default();
                    let chapter_number = helper::extract_chapter_number(&chapter_text);

                    // Parse date if available
                    let date_uploaded = if let Some(parent) = link.parent() {
                        // Look for date elements
                        if let Some(date_els) = parent.select(".date, .chapter-date, time") {
                            if let Some(date_el) = date_els.first() {
                                if let Some(date_text) = date_el.text() {
                                    Some(helper::parse_relative_time(&date_text) as i64)
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    chapters.push(Chapter {
                        key: url,
                        title: Some(chapter_text.trim().to_string()),
                        chapter_number: Some(chapter_number),
                        volume_number: None,
                        date_uploaded,
                        scanlators: None,
                        url: None,
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

    // Fallback: try to extract from script tags
    if pages.is_empty() {
        if let Some(scripts) = html.select("script") {
            for script in scripts {
                if let Some(script_text) = script.text() {
                    if script_text.contains("http") && (script_text.contains(".jpg") || script_text.contains(".png") || script_text.contains(".webp")) {
                        // Try to extract image URLs from script
                        for line in script_text.lines() {
                            if line.contains("http") && (line.contains(".jpg") || line.contains(".png") || line.contains(".webp")) {
                                // Simple extraction - look for URLs in quotes
                                let parts: Vec<&str> = line.split('"').collect();
                                for part in parts {
                                    if part.starts_with("http") && (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp") || part.contains(".jpg?") || part.contains(".png?") || part.contains(".webp?")) {
                                        pages.push(Page {
                                            content: PageContent::Url(part.to_string(), None),
                                            thumbnail: None,
                                            has_description: false,
                                            description: None,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(pages)
}
