#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Manga, MangaPageResult,
    MangaStatus, Page, PageContent, PageContext, Result, Source, Viewer,
    alloc::{String, Vec, format},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::string::ToString;

pub static BASE_URL: &str = "https://harmony-scan.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36";

pub struct Harmony;

impl Source for Harmony {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        use aidoku::println;

        // Check if query is non-empty
        let has_query = query.as_ref().map_or(false, |q| !q.trim().is_empty());

        // Check if filters have meaningful values (not default values)
        let has_meaningful_filters = filters.iter().any(|filter| {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "orderby" {
                        !value.is_empty() && value != "Pertinence"
                    } else if id == "status" {
                        !value.is_empty() && value != "Tous"
                    } else {
                        false
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" {
                        included.iter().any(|g| !g.is_empty() && g != "Tous")
                    } else {
                        false
                    }
                }
                _ => false
            }
        });

        println!("Query: {:?}", query);
        println!("Filters count: {}", filters.len());
        println!("Has query: {}", has_query);
        println!("Has meaningful filters: {}", has_meaningful_filters);

        let mut url = if has_query || has_meaningful_filters {
            // Search/filter format: /?s=query&post_type=wp-manga
            let search_query = query.unwrap_or_default();
            if page > 1 {
                format!("{}/page/{}/?s={}&post_type=wp-manga", BASE_URL, page, urlencode(&search_query))
            } else {
                format!("{}/?s={}&post_type=wp-manga", BASE_URL, urlencode(&search_query))
            }
        } else {
            // Normal listing format: /manga/page/X/
            if page > 1 {
                format!("{}/manga/page/{}/", BASE_URL, page)
            } else {
                format!("{}/manga/", BASE_URL)
            }
        };

        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "orderby" && !value.is_empty() {
                        let order_value = match value.as_str() {
                            "Dernières sorties" => "latest",
                            "Alphabétique" => "alphabet",
                            "Note" => "rating",
                            "Tendances" => "trending",
                            "Vues" => "views",
                            "Nouveautés" => "new-manga",
                            _ => continue,
                        };
                        url.push_str(&format!("&m_orderby={}", order_value));
                    } else if id == "status" && !value.is_empty() && value != "Tous" {
                        let status_value = match value.as_str() {
                            "En cours" => "on-going",
                            "Terminé" => "end",
                            "En pause" => "on-hold",
                            "Annulé" => "canceled",
                            _ => continue,
                        };
                        url.push_str(&format!("&status[]={}", status_value));
                    }
                }
                FilterValue::MultiSelect { id, included, excluded: _ } => {
                    if id == "genre" {
                        for genre in included {
                            if !genre.is_empty() && genre != "Tous" {
                                url.push_str(&format!("&genre[]={}", urlencode(&genre.to_lowercase())));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        println!("URL: {}", url);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        parse_manga_list(html)
    }

    fn get_manga_update(
        &self,
        mut manga: Manga,
        needs_details: bool,
        needs_chapters: bool,
    ) -> Result<Manga> {
        let url = format!("{}{}", BASE_URL, manga.key);
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .html()?;

        if needs_details {
            manga = parse_manga_details(manga, &html)?;
        }

        if needs_chapters {
            // Try to fetch chapters from AJAX endpoint first
            let manga_key = manga.key.trim_end_matches('/');
            let ajax_url = format!("{}{}/ajax/chapters/", BASE_URL, manga_key);

            let chapters = match Request::post(&ajax_url) {
                Ok(request) => {
                    match request
                        .header("User-Agent", USER_AGENT)
                        .header("Referer", &url)
                        .body(b"")
                        .html()
                    {
                        Ok(ajax_html) => parse_chapter_list(&ajax_html),
                        Err(_) => parse_chapter_list(&html)
                    }
                }
                Err(_) => parse_chapter_list(&html)
            }?;

            manga.chapters = Some(chapters);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let chapter_key = chapter.key.trim_end_matches('/');
        let url = format!("{}{}/?style=list", BASE_URL, chapter_key);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .html()?;

        parse_page_list(&html)
    }
}

impl ImageRequestProvider for Harmony {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    let has_more = true;

    // Try search/filter format first (div.c-tabs-item__content)
    if let Some(manga_items) = html.select("div.c-tabs-item__content") {
        for item in manga_items {
            let link = if let Some(links) = item.select("div.post-title h3 a, div.tab-summary div.post-title h3 a") {
                if let Some(first_link) = links.first() {
                    first_link
                } else {
                    continue;
                }
            } else {
                continue;
            };

            let url = link.attr("abs:href").or_else(|| link.attr("href")).unwrap_or_default();
            let title = link.text().unwrap_or_default().trim().to_string();

            if url.is_empty() || title.is_empty() {
                continue;
            }

            let key = url.replace(BASE_URL, "");

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

            mangas.push(Manga {
                key,
                title,
                cover: if cover.is_empty() { None } else { Some(cover) },
                url: Some(url),
                ..Default::default()
            });
        }
    }

    // Fallback to normal listing format (div.page-item-detail)
    if mangas.is_empty() {
        if let Some(manga_items) = html.select("div.page-item-detail") {
            for item in manga_items {
                let link = if let Some(links) = item.select("div.post-title a") {
                    if let Some(first_link) = links.first() {
                        first_link
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let url = link.attr("abs:href").unwrap_or_default();
                let title = link.own_text().unwrap_or_default().trim().to_string();

                if url.is_empty() || title.is_empty() {
                    continue;
                }

                let key = url.replace(BASE_URL, "");

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

                mangas.push(Manga {
                    key,
                    title,
                    cover: if cover.is_empty() { None } else { Some(cover) },
                    url: Some(url),
                    ..Default::default()
                });
            }
        }
    }

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: has_more,
    })
}

fn parse_manga_details(mut manga: Manga, html: &Document) -> Result<Manga> {
    if let Some(cover_images) = html.select("div.summary_image img") {
        if let Some(cover_img) = cover_images.first() {
            let cover_url = cover_img.attr("data-src")
                .or_else(|| cover_img.attr("src"))
                .unwrap_or_default();
            if !cover_url.is_empty() {
                manga.cover = Some(cover_url);
            }
        }
    }

    if let Some(desc_elements) = html.select("div.description-summary div.summary__content") {
        if let Some(desc) = desc_elements.first() {
            if let Some(text) = desc.text() {
                manga.description = Some(text.trim().to_string());
            }
        }
    }

    let mut authors = Vec::new();
    if let Some(author_elements) = html.select("div.author-content a") {
        for author in author_elements {
            if let Some(text) = author.text() {
                authors.push(text.trim().to_string());
            }
        }
    }
    if !authors.is_empty() {
        manga.authors = Some(authors);
    }

    let mut artists = Vec::new();
    if let Some(artist_elements) = html.select("div.artist-content a") {
        for artist in artist_elements {
            if let Some(text) = artist.text() {
                artists.push(text.trim().to_string());
            }
        }
    }
    if !artists.is_empty() {
        manga.artists = Some(artists);
    }

    if let Some(status_elements) = html.select("div.post-status div.summary-content") {
        if let Some(status_elem) = status_elements.first() {
            if let Some(status_text) = status_elem.text() {
                let status_lower = status_text.trim().to_lowercase();
                manga.status = if status_lower.contains("complet") || status_lower.contains("terminé") {
                    MangaStatus::Completed
                } else if status_lower.contains("en cours") || status_lower.contains("ongoing") {
                    MangaStatus::Ongoing
                } else if status_lower.contains("pause") || status_lower.contains("hiatus") {
                    MangaStatus::Hiatus
                } else if status_lower.contains("annulé") || status_lower.contains("canceled") {
                    MangaStatus::Cancelled
                } else {
                    MangaStatus::Unknown
                };
            }
        }
    }

    let mut tags = Vec::new();
    if let Some(genre_elements) = html.select("div.genres-content a") {
        for genre in genre_elements {
            if let Some(text) = genre.text() {
                tags.push(text.trim().to_string());
            }
        }
    }
    manga.tags = if tags.is_empty() { None } else { Some(tags) };

    manga.content_rating = ContentRating::NSFW;
    manga.viewer = Viewer::default();
    manga.url = Some(format!("{}{}", BASE_URL, manga.key));

    Ok(manga)
}

fn parse_chapter_list(html: &Document) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();

    if let Some(chapter_items) = html.select("li.wp-manga-chapter") {
        for item in chapter_items {
            if let Some(links) = item.select("a") {
                if let Some(link) = links.first() {
                    let url = link.attr("abs:href").or_else(|| link.attr("href")).unwrap_or_default();

                    if url.is_empty() || !url.contains("chapitre") {
                        continue;
                    }

                    let key = url.replace(BASE_URL, "").replace("/?style=list", "");
                    let title = link.text().unwrap_or_default().trim().to_string();

                    if title.is_empty() {
                        continue;
                    }

                    let chapter_num = extract_chapter_number(&title);
                    let chapter_url = format!("{}{}", BASE_URL, key);

                    chapters.push(Chapter {
                        key,
                        title: Some(title),
                        chapter_number: if chapter_num > 0.0 { Some(chapter_num as f32) } else { None },
                        url: Some(chapter_url),
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(chapters)
}

fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();

    // Try wp-manga-chapter-img first (most common)
    if let Some(img_elements) = html.select("img.wp-manga-chapter-img") {
        for img in img_elements {
            let img_url = img.attr("data-src")
                .or_else(|| img.attr("data-lazy-src"))
                .or_else(|| img.attr("src"))
                .unwrap_or_default()
                .trim()
                .to_string();

            if !img_url.is_empty() && !img_url.contains("loader") {
                pages.push(Page {
                    content: PageContent::Url(img_url, None),
                    ..Default::default()
                });
            }
        }
    }

    // Fallback to other selectors if no pages found
    if pages.is_empty() {
        if let Some(img_elements) = html.select("div.page-break img, li.blocks-gallery-item img") {
            for img in img_elements {
                let img_url = img.attr("data-src")
                    .or_else(|| img.attr("data-lazy-src"))
                    .or_else(|| img.attr("src"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                if !img_url.is_empty() && !img_url.contains("loader") {
                    pages.push(Page {
                        content: PageContent::Url(img_url, None),
                        ..Default::default()
                    });
                }
            }
        }
    }

    Ok(pages)
}

fn extract_chapter_number(title: &str) -> f64 {
    let lower = title.to_lowercase();

    let patterns = ["chapitre ", "chapter ", "ch. ", "ch ", "chap "];
    for pattern in patterns {
        if let Some(pos) = lower.find(pattern) {
            let after = &lower[pos + pattern.len()..];
            if let Some(num_str) = after.split_whitespace().next() {
                if let Ok(num) = num_str.replace(",", ".").parse::<f64>() {
                    return num;
                }
            }
        }
    }

    for word in lower.split_whitespace() {
        if let Ok(num) = word.replace(",", ".").parse::<f64>() {
            if num > 0.0 && num < 10000.0 {
                return num;
            }
        }
    }

    -1.0
}

fn urlencode(text: &str) -> String {
    let mut result = String::new();
    for byte in text.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                result.push(byte as char);
            }
            b' ' => result.push_str("+"),
            _ => {
                result.push('%');
                result.push_str(&format!("{:02X}", byte));
            }
        }
    }
    result
}

register_source!(Harmony, ImageRequestProvider);
