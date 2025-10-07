use aidoku::{
    alloc::{String, Vec, format, string::ToString},
    imports::html::Document,
    AidokuError, Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent,
    Result, UpdateStrategy, Viewer,
};
use serde_json::Value;

use crate::BASE_URL;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas = Vec::new();

    if let Some(elem_list) = html.select(".mangas-list .manga-block:not(:has(a[href='']))") {
        for elem in elem_list {
            if let Some(link_list) = elem.select("a") {
                if let Some(link) = link_list.first() {
                    let url = link.attr("href").unwrap_or_default();
                    let title = link.text().unwrap_or_default();
                    let thumbnail = link.select("img")
                        .and_then(|imgs| imgs.first())
                        .and_then(|img| img.attr("abs:data-src"));

                    if !url.is_empty() && !title.is_empty() {
                        mangas.push(Manga {
                            key: url.clone(),
                            title,
                            cover: thumbnail,
                            url: Some(url),
                            authors: None,
                            artists: None,
                            description: None,
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
            }
        }
    }

    let has_next_page = html.select(".pagination > li:last-child:not(.disabled)")
        .and_then(|l| l.first())
        .is_some();

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page,
    })
}

pub fn parse_search_json(json_str: String) -> Result<MangaPageResult> {
    let json: Value = serde_json::from_str(&json_str)
        .map_err(|_| AidokuError::message("JSON parsing failed"))?;

    let mut mangas = Vec::new();

    if let Some(array) = json.as_array() {
        for item in array {
            if let (Some(url), Some(name), Some(image)) = (
                item["url"].as_str(),
                item["name"].as_str(),
                item["image"].as_str(),
            ) {
                mangas.push(Manga {
                    key: String::from(url),
                    title: String::from(name),
                    cover: Some(format!("{}{}", BASE_URL, image)),
                    url: Some(String::from(url)),
                    authors: None,
                    artists: None,
                    description: None,
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
    }

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: false,
    })
}

pub fn parse_browse_list(html: Document, base_host: &str) -> Result<MangaPageResult> {
    let mut mangas = Vec::new();

    if let Some(elem_list) = html.select("div.card div.p-2") {
        for elem in elem_list {
            if let Some(link) = elem.select("p a").and_then(|links| links.first()) {
                let url = link.attr("abs:href").unwrap_or_default();

                if url.contains(base_host) && !url.is_empty() {
                    let title = link.text().unwrap_or_default();
                    let thumbnail = elem.select("img")
                        .and_then(|imgs| imgs.first())
                        .and_then(|img| img.attr("abs:src"));

                    mangas.push(Manga {
                        key: url.clone(),
                        title,
                        cover: thumbnail,
                        url: Some(url),
                        authors: None,
                        artists: None,
                        description: None,
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
        }
    }

    let has_next_page = html.select(".pagination > li:last-child:not(.disabled)")
        .and_then(|l| l.first())
        .is_some();

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page,
    })
}

pub fn parse_manga_details(html: Document) -> Result<Manga> {
    let info_element = html.select("#main .card-body")
        .and_then(|elems| elems.first())
        .ok_or_else(|| AidokuError::message("Manga info not found"))?;

    let thumbnail = info_element.select("img")
        .and_then(|imgs| imgs.first())
        .and_then(|img| img.attr("abs:src"));

    let mut authors = Vec::new();
    let mut artists = Vec::new();
    let mut genres = Vec::new();
    let mut status = MangaStatus::Unknown;

    if let Some(rows) = info_element.select(".row, .d-flex") {
        for elem in rows {
            if let Some(p) = elem.select("p").and_then(|ps| ps.first()) {
                let text = p.text().unwrap_or_default();
                let span_text = p.select("span")
                    .and_then(|spans| spans.first())
                    .and_then(|s| s.text())
                    .unwrap_or_default();

                if span_text.contains("Auteur(s):") {
                    let author_str = text.replace("Auteur(s):", "").trim().to_string();
                    if !author_str.is_empty() {
                        authors.push(author_str);
                    }
                } else if span_text.contains("Artiste(s):") {
                    let artist_str = text.replace("Artiste(s):", "").trim().to_string();
                    if !artist_str.is_empty() {
                        artists.push(artist_str);
                    }
                } else if span_text.contains("Genre(s):") {
                    let genre_str = text.replace("Genre(s):", "").trim().to_string();
                    genres = genre_str.split(',').map(|g| String::from(g.trim())).collect();
                } else if span_text.contains("Statut:") {
                    let status_str = text.replace("Statut:", "").trim().to_lowercase();
                    status = if status_str.contains("en cours") {
                        MangaStatus::Ongoing
                    } else if status_str.contains("terminé") {
                        MangaStatus::Completed
                    } else {
                        MangaStatus::Unknown
                    };
                }
            }
        }
    }

    let description = info_element.select("div:contains(Synopsis) + p")
        .and_then(|ps| ps.first())
        .and_then(|p| p.own_text());

    let title = html.select("#main h1, #main .card-title")
        .and_then(|elems| elems.first())
        .and_then(|h| h.text())
        .unwrap_or_else(|| String::from("Unknown"));

    Ok(Manga {
        key: String::new(),
        title,
        cover: thumbnail,
        authors: if authors.is_empty() { None } else { Some(authors) },
        artists: if artists.is_empty() { None } else { Some(artists) },
        description,
        url: None,
        tags: if genres.is_empty() { None } else { Some(genres) },
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::RightToLeft,
        chapters: None,
        next_update_time: None,
        update_strategy: UpdateStrategy::Never,
    })
}

pub fn parse_chapter_list(html: Document, hide_spoilers: bool) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();

    let selector = if hide_spoilers {
        "#list_chapters > div.collapse > div.list_chapters:not(:has(.badge:contains(SPOILER),.badge:contains(RAW),.badge:contains(VUS)))"
    } else {
        "#list_chapters > div.collapse > div.list_chapters"
    };

    if let Some(elem_list) = html.select(selector) {
        for elem in elem_list {
            let mut chapter_url = String::new();
            let mut chapter_title = String::new();

            if let Some(link_list) = elem.select("a") {
                for link in link_list {
                    for attr_name in ["href", "data-href", "data-url", "data-link"].iter() {
                        let url_val = link.attr(attr_name).unwrap_or_default();
                        if url_val.starts_with("/manga/") || url_val.starts_with("/manhua/") || url_val.starts_with("/manhwa/") {
                            chapter_url = url_val;
                            chapter_title = link.own_text().unwrap_or_default();
                            if chapter_title.is_empty() {
                                chapter_title = link.text().unwrap_or_default();
                            }
                            break;
                        }
                    }

                    if !chapter_url.is_empty() {
                        break;
                    }
                }
            }

            if !chapter_url.is_empty() {
                let chapter_number = extract_chapter_number(&chapter_title);

                chapters.push(Chapter {
                    key: chapter_url.clone(),
                    title: Some(chapter_title),
                    volume_number: None,
                    chapter_number: if chapter_number > 0.0 { Some(chapter_number) } else { None },
                    date_uploaded: None,
                    scanlators: None,
                    url: Some(chapter_url),
                    language: Some(String::from("fr")),
                    thumbnail: None,
                    locked: false,
                });
            }
        }
    }

    Ok(chapters)
}

fn extract_chapter_number(title: &str) -> f32 {
    let words: Vec<&str> = title.split_whitespace().collect();
    for word in words {
        let cleaned: String = word.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        if let Ok(num) = cleaned.parse::<f32>() {
            return num;
        }
    }
    -1.0
}

pub fn parse_page_list(html_content: String, base_url: &str) -> Result<Vec<Page>> {
    let urls = crate::helper::extract_images_from_script(&html_content)
        .ok_or_else(|| AidokuError::message("Failed to extract image URLs from JavaScript"))?;

    let base_host = if let Some(domain_start) = base_url.find("://") {
        let after_protocol = &base_url[domain_start + 3..];
        if let Some(slash) = after_protocol.find('/') {
            &after_protocol[..slash]
        } else {
            after_protocol
        }
    } else {
        base_url
    };

    let filtered_urls: Vec<String> = urls.into_iter()
        .filter(|url| {
            if let Some(domain_start) = url.find("://") {
                let after_protocol = &url[domain_start + 3..];
                let host = if let Some(slash) = after_protocol.find('/') {
                    &after_protocol[..slash]
                } else {
                    after_protocol
                };
                host.contains(base_host.trim_start_matches("www."))
            } else {
                false
            }
        })
        .collect();

    let pages = filtered_urls.into_iter()
        .map(|url| Page {
            content: PageContent::url(url),
            thumbnail: None,
            has_description: false,
            description: None,
        })
        .collect();

    Ok(pages)
}
