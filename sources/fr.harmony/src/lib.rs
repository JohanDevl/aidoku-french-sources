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
            println!("[HARMONY] needs_chapters = true");
            println!("[HARMONY] manga.key = {}", manga.key);

            // Try to fetch chapters from AJAX endpoint first
            let manga_key = manga.key.trim_end_matches('/');
            let ajax_url = format!("{}{}/ajax/chapters/", BASE_URL, manga_key);
            println!("[HARMONY] ajax_url = {}", ajax_url);

            let chapters = match Request::post(&ajax_url) {
                Ok(request) => {
                    println!("[HARMONY] AJAX POST request created");
                    match request
                        .header("User-Agent", USER_AGENT)
                        .header("Referer", &url)
                        .body(b"")
                        .html()
                    {
                        Ok(ajax_html) => {
                            println!("[HARMONY] AJAX HTML received");

                            // Debug: check what's in the HTML
                            if let Some(all_lis) = ajax_html.select("li") {
                                println!("[HARMONY] Total <li> elements: {}", all_lis.count());
                            }

                            // Debug: print first few li elements with their class and content
                            if let Some(all_lis) = ajax_html.select("li") {
                                let mut count = 0;
                                for li in all_lis {
                                    if count >= 3 { break; }
                                    let class_attr = li.attr("class").unwrap_or_default();
                                    let text = li.text().unwrap_or_default();
                                    println!("[HARMONY] li[{}] class='{}' text='{}'", count, class_attr, text.chars().take(50).collect::<String>());

                                    if let Some(a_tags) = li.select("a") {
                                        if let Some(first_a) = a_tags.first() {
                                            let href = first_a.attr("href").unwrap_or_default();
                                            println!("[HARMONY]   -> has <a> with href='{}'", href);
                                        }
                                    }
                                    count += 1;
                                }
                            }

                            let result = parse_chapter_list(&ajax_html);
                            if let Ok(ref chapters) = result {
                                println!("[HARMONY] Parsed {} chapters from AJAX", chapters.len());
                            } else {
                                println!("[HARMONY] Failed to parse chapters from AJAX");
                            }
                            result
                        },
                        Err(_) => {
                            println!("[HARMONY] AJAX html() failed, falling back to main page");
                            parse_chapter_list(&html)
                        }
                    }
                }
                Err(_) => {
                    println!("[HARMONY] AJAX request failed, falling back to main page");
                    parse_chapter_list(&html)
                }
            }?;

            println!("[HARMONY] Final chapter count = {}", chapters.len());
            manga.chapters = Some(chapters);
        }

        Ok(manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}{}/?style=list", BASE_URL, chapter.key);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .html()?;

        parse_page_list(&html)
    }
}

impl ImageRequestProvider for Harmony {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
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

    // Look for wp-manga-chapter list items
    println!("[HARMONY] Searching for li.wp-manga-chapter");
    if let Some(chapter_items) = html.select("li.wp-manga-chapter") {
        let count = chapter_items.count();
        println!("[HARMONY] Found {} chapter items", count);

        if let Some(chapter_items) = html.select("li.wp-manga-chapter") {
            for item in chapter_items {
                if let Some(links) = item.select("a") {
                    if let Some(link) = links.first() {
                        let url = link.attr("abs:href").or_else(|| link.attr("href")).unwrap_or_default();
                        println!("[HARMONY] Found chapter URL: {}", url);

                        if url.is_empty() || !url.contains("chapitre") {
                            println!("[HARMONY] URL rejected");
                            continue;
                        }

                        let key = url.replace(BASE_URL, "").replace("/?style=list", "");
                        let title = link.text().unwrap_or_default().trim().to_string();

                        if title.is_empty() {
                            println!("[HARMONY] Title is empty, skipping");
                            continue;
                        }

                        println!("[HARMONY] Adding chapter: {}", title);
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
    }

    chapters.reverse();
    println!("[HARMONY] Returning {} chapters", chapters.len());
    Ok(chapters)
}

fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();

    if let Some(img_elements) = html.select("div.page-break img, li.blocks-gallery-item img") {
        for img in img_elements {
            let img_url = img.attr("data-src")
                .or_else(|| img.attr("data-lazy-src"))
                .or_else(|| img.attr("src"))
                .unwrap_or_default();

            if !img_url.is_empty() && !img_url.contains("loader") {
                pages.push(Page {
                    content: PageContent::Text(img_url),
                    ..Default::default()
                });
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
