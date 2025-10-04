use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter, UpdateStrategy,
    ContentRating, Viewer,
    alloc::{String, Vec, string::ToString, vec},
    imports::html::Document,
    prelude::*,
};
use core::cmp::Ordering;
use crate::helper;
use crate::BASE_URL;

// Parse manga list from HTML homepage
pub fn parse_manga_list(html: &Document, _page: i32) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();

    // Find manga items - similar structure to what we saw in browser
    let selectors = [
        ".listupd article",
        ".page-item-detail",
        "article",
        "div[class*='manga']",
    ];

    for selector in &selectors {
        if let Some(items) = html.select(selector) {
            for item in items {
                // Find the main link to manga
                if let Some(link) = item.select("a[href*='/lecture-en-ligne/']").and_then(|els| els.first()) {
                    let href = link.attr("href").unwrap_or_default();
                    if href.is_empty() {
                        continue;
                    }

                    // Extract slug from URL: /lecture-en-ligne/manga-slug
                    let key = href
                        .replace(BASE_URL, "")
                        .replace("/lecture-en-ligne/", "")
                        .trim_start_matches('/')
                        .trim_end_matches('/')
                        .to_string();

                    if key.is_empty() {
                        continue;
                    }

                    // Extract title
                    let title = link.attr("title")
                        .or_else(|| item.select(".title, h3, h2").and_then(|els| els.first()).and_then(|el| el.text()))
                        .unwrap_or_default()
                        .trim()
                        .to_string();

                    if title.is_empty() {
                        continue;
                    }

                    // Extract cover image
                    let cover = item.select("img").and_then(|imgs| imgs.first())
                        .and_then(|img| img.attr("data-src")
                            .or_else(|| img.attr("data-lazy-src"))
                            .or_else(|| img.attr("src")))
                        .map(|s| s.to_string());

                    mangas.push(Manga {
                        key,
                        title,
                        cover,
                        authors: None,
                        artists: None,
                        description: None,
                        tags: None,
                        url: Some(href),
                        status: MangaStatus::Unknown,
                        content_rating: ContentRating::Safe,
                        viewer: Viewer::RightToLeft,
                        chapters: None,
                        next_update_time: None,
                        update_strategy: UpdateStrategy::Never,
                    });
                }
            }

            if !mangas.is_empty() {
                break;
            }
        }
    }

    // Check for next page - look for pagination
    let has_next_page = html.select(".pagination .next, a[rel='next']").is_some();

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

                    // Extract chapter number
                    let chapter_text = link.text().unwrap_or_default();
                    let chapter_number = helper::extract_chapter_number(&chapter_text);

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
