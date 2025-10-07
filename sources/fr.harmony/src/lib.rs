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
        let mut url = if let Some(search_query) = query {
            format!("{}/page/{}/?s={}&post_type=wp-manga", BASE_URL, page, urlencode(&search_query))
        } else {
            format!("{}/manga/page/{}/", BASE_URL, page)
        };

        for filter in filters {
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
        use alloc::string::ToString;

        println!("[HARMONY] ===== get_page_list START =====");
        println!("[HARMONY] Source version: 17");

        // Safe string operations to avoid any potential panic
        let chapter_key_str = chapter.key.to_string();
        println!("[HARMONY] chapter.key length: {}", chapter_key_str.len());
        println!("[HARMONY] chapter.key = '{}'", chapter_key_str);

        if let Some(ref title) = chapter.title {
            println!("[HARMONY] chapter.title = '{}'", title);
        } else {
            println!("[HARMONY] chapter.title = None");
        }

        if let Some(ref url) = chapter.url {
            println!("[HARMONY] chapter.url = '{}'", url);
        } else {
            println!("[HARMONY] chapter.url = None");
        }

        // Build URL very carefully with explicit steps
        println!("[HARMONY] Step 1: Trim trailing slash");
        let chapter_key = if chapter_key_str.ends_with('/') {
            let trimmed = &chapter_key_str[..chapter_key_str.len()-1];
            println!("[HARMONY] Trimmed '{}' -> '{}'", chapter_key_str, trimmed);
            trimmed
        } else {
            println!("[HARMONY] No trailing slash to trim");
            &chapter_key_str
        };

        println!("[HARMONY] Step 2: Build final URL");
        let url = format!("{}{}/?style=list", BASE_URL, chapter_key);
        println!("[HARMONY] Final URL length: {}", url.len());
        println!("[HARMONY] Final URL = '{}'", url);

        println!("[HARMONY] Step 3: Create request");
        let request = match Request::get(&url) {
            Ok(r) => {
                println!("[HARMONY] Request created successfully");
                r
            },
            Err(_) => {
                println!("[HARMONY] ERROR: Failed to create request");
                return Err(aidoku::AidokuError::Message("Failed to create request".to_string()));
            }
        };

        println!("[HARMONY] Step 4: Add headers and fetch");
        let html = match request
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .html()
        {
            Ok(h) => {
                println!("[HARMONY] HTML received successfully");
                h
            },
            Err(_) => {
                println!("[HARMONY] ERROR: Failed to fetch HTML");
                return Err(aidoku::AidokuError::Message("Failed to fetch HTML".to_string()));
            }
        };

        println!("[HARMONY] Step 5: Parse pages");
        let pages = match parse_page_list(&html) {
            Ok(p) => {
                println!("[HARMONY] Pages parsed successfully: {} pages", p.len());
                p
            },
            Err(_) => {
                println!("[HARMONY] ERROR: Failed to parse pages");
                return Err(aidoku::AidokuError::Message("Failed to parse pages".to_string()));
            }
        };

        if pages.is_empty() {
            println!("[HARMONY] WARNING: No pages found!");
        }

        println!("[HARMONY] ===== get_page_list END - SUCCESS =====");

        Ok(pages)
    }
}

impl ImageRequestProvider for Harmony {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        println!("[HARMONY] get_image_request called for: {}", url);

        let request = Request::get(&url)?;
        println!("[HARMONY] Image request created");

        let request_with_headers = request
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL);

        println!("[HARMONY] Headers added to image request");

        Ok(request_with_headers)
    }
}

fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    let has_more = true;

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
    println!("[HARMONY] parse_chapter_list called");
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

                    println!("[HARMONY] Creating chapter: key='{}', url='{}', title='{}'", key, chapter_url, title);

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

    println!("[HARMONY] Created {} chapters", chapters.len());
    Ok(chapters)
}

fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
    println!("[HARMONY] parse_page_list called");
    let mut pages: Vec<Page> = Vec::new();

    // Try wp-manga-chapter-img first (most common)
    if let Some(img_elements) = html.select("img.wp-manga-chapter-img") {
        let count = img_elements.count();
        println!("[HARMONY] Found {} wp-manga-chapter-img elements", count);

        if let Some(img_elements) = html.select("img.wp-manga-chapter-img") {
            for (i, img) in img_elements.enumerate() {
                let img_url = img.attr("data-src")
                    .or_else(|| img.attr("data-lazy-src"))
                    .or_else(|| img.attr("src"))
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                println!("[HARMONY] img[{}] raw url = '{}'", i, img_url);

                if !img_url.is_empty() && !img_url.contains("loader") {
                    println!("[HARMONY] Adding page {}", i);
                    pages.push(Page {
                        content: PageContent::Url(img_url, None),
                        ..Default::default()
                    });
                }
            }
        }
    }

    // Fallback to other selectors if no pages found
    if pages.is_empty() {
        println!("[HARMONY] No wp-manga-chapter-img found, trying fallback selectors");
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

    println!("[HARMONY] Returning {} pages", pages.len());
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
