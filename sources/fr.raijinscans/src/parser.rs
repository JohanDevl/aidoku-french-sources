use aidoku::{
    Result, Manga, MangaStatus, Chapter, Page, PageContent,
    ContentRating, Viewer, UpdateStrategy,
    alloc::{String, Vec, vec, string::ToString},
    imports::html::Document,
};
use crate::helper::{decode_base64, make_absolute_url, parse_relative_date};

extern crate alloc;

pub fn parse_manga_list(html: &Document, base_url: &str) -> Vec<Manga> {
    let mut mangas = Vec::new();

    if let Some(items) = html.select("div.unit") {
        for item in items {
            let link = if let Some(links) = item.select("div.info a, a.c-title") {
                if let Some(l) = links.first() {
                    l
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let url = link.attr("href").unwrap_or_default();
            let title_attr = link.attr("title").unwrap_or_default();
            let title_text = link.text().unwrap_or_default();
            let title = if !title_attr.is_empty() { title_attr } else { title_text };

            if url.is_empty() || title.is_empty() {
                continue;
            }

            let key = url.clone();

            let cover = if let Some(imgs) = item.select("div.poster-image-wrapper img, a.poster img") {
                if let Some(img) = imgs.first() {
                    let data_src = img.attr("data-src").unwrap_or_default();
                    let src = img.attr("src").unwrap_or_default();
                    let cover_url = if !data_src.is_empty() { data_src } else { src };
                    if !cover_url.is_empty() {
                        Some(make_absolute_url(base_url, &cover_url))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            mangas.push(Manga {
                key: key.clone(),
                cover,
                title,
                authors: None,
                artists: None,
                description: None,
                tags: None,
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                url: Some(make_absolute_url(base_url, &url)),
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            });
        }
    }

    mangas
}

pub fn parse_manga_details(html: &Document, manga_key: String, base_url: &str) -> Result<Manga> {
    let title = if let Some(title_elems) = html.select("h1.serie-title") {
        if let Some(elem) = title_elems.first() {
            elem.text().unwrap_or_default()
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    let author = if let Some(author_elems) = html.select("div.stat-item:has(span:contains(Auteur)) span.stat-value") {
        if let Some(elem) = author_elems.first() {
            let text = elem.text().unwrap_or_default();
            if !text.is_empty() { Some(vec![text]) } else { None }
        } else {
            None
        }
    } else {
        None
    };

    let artist = if let Some(artist_elems) = html.select("div.stat-item:has(span:contains(Artiste)) span.stat-value") {
        if let Some(elem) = artist_elems.first() {
            let text = elem.text().unwrap_or_default();
            if !text.is_empty() { Some(vec![text]) } else { None }
        } else {
            None
        }
    } else {
        None
    };

    let mut description = None;
    if let Some(scripts) = html.select("script") {
        for script in scripts {
            if let Some(html_content) = script.html() {
                if html_content.contains("content.innerHTML") {
                    if let Some(start) = html_content.find("content.innerHTML = `") {
                        let desc_start = start + 21;
                        if let Some(end) = html_content[desc_start..].find("`;") {
                            description = Some(html_content[desc_start..desc_start + end].to_string());
                            break;
                        }
                    }
                }
            }
        }
    }

    if description.is_none() {
        if let Some(desc_elems) = html.select("div.description-content") {
            if let Some(elem) = desc_elems.first() {
                let text = elem.text().unwrap_or_default();
                if !text.is_empty() {
                    description = Some(text);
                }
            }
        }
    }

    let tags = if let Some(genre_elems) = html.select("div.genre-list div.genre-link") {
        let mut tags_vec = Vec::new();
        for elem in genre_elems {
            let tag = elem.text().unwrap_or_default();
            if !tag.is_empty() {
                tags_vec.push(tag);
            }
        }
        if !tags_vec.is_empty() { Some(tags_vec) } else { None }
    } else {
        None
    };

    let cover = if let Some(cover_elems) = html.select("img.cover") {
        if let Some(elem) = cover_elems.first() {
            let src = elem.attr("src").unwrap_or_default();
            if !src.is_empty() {
                Some(make_absolute_url(base_url, &src))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let status = if let Some(status_elems) = html.select("div.stat-item:has(span:contains(État)) span.manga") {
        if let Some(elem) = status_elems.first() {
            let status_text = elem.text().unwrap_or_default().to_lowercase();
            if status_text.contains("en cours") {
                MangaStatus::Ongoing
            } else if status_text.contains("terminé") || status_text.contains("termine") {
                MangaStatus::Completed
            } else {
                MangaStatus::Unknown
            }
        } else {
            MangaStatus::Unknown
        }
    } else {
        MangaStatus::Unknown
    };

    Ok(Manga {
        key: manga_key.clone(),
        cover,
        title,
        authors: author,
        artists: artist,
        description,
        tags,
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::LeftToRight,
        chapters: None,
        url: Some(manga_key),
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    })
}

pub fn parse_chapter_list(html: &Document) -> Vec<Chapter> {
    let mut chapters = Vec::new();

    if let Some(items) = html.select("ul.scroll-sm li.item") {
        for item in items {
            let link = if let Some(links) = item.select("a") {
                if let Some(l) = links.first() {
                    l
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let url = link.attr("href").unwrap_or_default();
            let title = link.attr("title").unwrap_or_default();

            if url.is_empty() {
                continue;
            }

            let date_uploaded = if let Some(spans) = item.select("a span:nth-of-type(2)") {
                if let Some(span) = spans.first() {
                    let date_text = span.text().unwrap_or_default().to_lowercase();
                    parse_relative_date(&date_text)
                } else {
                    None
                }
            } else {
                None
            };

            chapters.push(Chapter {
                key: url.clone(),
                title: if !title.is_empty() { Some(title) } else { None },
                date_uploaded,
                url: Some(url),
                chapter_number: None,
                volume_number: None,
                scanlators: None,
                language: None,
                thumbnail: None,
                locked: false,
            });
        }
    }

    chapters
}

pub fn parse_page_list(html: &Document) -> Vec<Page> {
    let mut pages = Vec::new();

    if let Some(items) = html.select("div.protected-image-data") {
        for item in items {
            let encoded = item.attr("data-src").unwrap_or_default();

            if !encoded.is_empty() {
                let url = if let Some(decoded) = decode_base64(&encoded) {
                    decoded
                } else {
                    encoded
                };

                pages.push(Page {
                    content: PageContent::Url(url, None),
                    thumbnail: None,
                    has_description: false,
                    description: None,
                });
            }
        }
    }

    pages
}

pub fn has_next_page(html: &Document) -> bool {
    if let Some(next_elems) = html.select("li.page-item:not(.disabled) a[rel=next]") {
        if !next_elems.is_empty() {
            return true;
        }
    }

    if let Some(load_more) = html.select("a#load-more-manga") {
        if !load_more.is_empty() {
            return true;
        }
    }

    false
}
