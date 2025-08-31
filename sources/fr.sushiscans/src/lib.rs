#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
    MangaStatus, Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec, vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString};

pub static BASE_URL: &str = "https://sushiscan.fr";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub struct SushiScans;

impl Source for SushiScans {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        
        let url = if let Some(search_query) = query {
            if search_query.is_empty() {
                format!("{}/catalogue/?page={}", BASE_URL, page)
            } else {
                format!("{}/?s={}&page={}", BASE_URL, search_query, page)
            }
        } else {
            format!("{}/catalogue/?page={}", BASE_URL, page)
        };

        self.get_manga_from_page(&url)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        let url = format!("{}/manga/{}/", BASE_URL, manga.key);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_manga_details(html, manga.key, _needs_details, needs_chapters)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        let url = format!("{}/{}/", BASE_URL, chapter.key);
        
        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_page_list(html)
    }
}

impl ListingProvider for SushiScans {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        
        let url = match listing.name.as_str() {
            "Dernières" => format!("{}/catalogue/?page={}&order=update", BASE_URL, page),
            "Populaire" => format!("{}/catalogue/?page={}&order=popular", BASE_URL, page),
            "Nouveau" => format!("{}/catalogue/?page={}&order=latest", BASE_URL, page),
            _ => format!("{}/catalogue/?page={}", BASE_URL, page),
        };
        
        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for SushiScans {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        Ok(Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL))
    }
}

impl SushiScans {
    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Accept-Encoding", "gzip, deflate, br")
            .header("DNT", "1")
            .header("Connection", "keep-alive")
            .header("Upgrade-Insecure-Requests", "1")
            .header("Referer", BASE_URL)
            .html()?;

        let mut entries: Vec<Manga> = Vec::new();

        // MangaStream selectors for sushiscan.fr
        if let Some(items) = html.select(".listupd .bsx, .utao .uta .imgu, .page-item-detail") {
            for item in items {
                let link = if let Some(a_element) = item.select("a") {
                    if let Some(first_link) = a_element.first() {
                        first_link
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let href = link.attr("href").unwrap_or_default();
                if href.is_empty() {
                    continue;
                }

                // Extract manga key from URL
                let key = href
                    .replace(BASE_URL, "")
                    .replace("/manga/", "")
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();

                if key.is_empty() {
                    continue;
                }

                let title = link.attr("title")
                    .or_else(|| {
                        item.select("h3 a, h5 a, .post-title, .manga-title")
                            .and_then(|elems| elems.first())
                            .and_then(|elem| elem.text())
                    })
                    .unwrap_or_default()
                    .trim()
                    .to_string();

                if title.is_empty() {
                    continue;
                }

                // Get cover image
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
        }

        // Check for pagination (MangaStream style)
        let has_next_page = html.select(".hpage a.r, .pagination .next").is_some();

        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }

    fn parse_manga_details(&self, html: Document, key: String, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        
        // Extract title
        let title = if let Some(title_elem) = html.select("h1.entry-title, .wp-manga-title, .manga-title") {
            if let Some(first_title) = title_elem.first() {
                first_title.text().unwrap_or_default().trim().to_string()
            } else {
                key.clone()
            }
        } else {
            key.clone()
        };

        // Extract cover 
        let cover = if let Some(cover_elem) = html.select(".infomanga > div[itemprop=image] img, .thumb img") {
            if let Some(first_cover) = cover_elem.first() {
                first_cover.attr("src").unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        // Extract author
        let author = if let Some(author_elem) = html.select(".infotable td:contains(Auteur)+td") {
            if let Some(first_author) = author_elem.first() {
                Some(vec![first_author.text().unwrap_or_default().trim().to_string()])
            } else {
                None
            }
        } else {
            None
        };

        // Extract description
        let description = if let Some(desc_elem) = html.select("div.desc p, div.entry-content p, div[itemprop=description]:not(:has(p))") {
            if let Some(first_desc) = desc_elem.first() {
                let desc_text = first_desc.text().unwrap_or_default().trim().to_string();
                if desc_text.is_empty() { None } else { Some(desc_text) }
            } else {
                None
            }
        } else {
            None
        };

        // Extract status
        let status = if let Some(status_elem) = html.select(".infotable td:contains(Statut)+td") {
            if let Some(first_status) = status_elem.first() {
                let status_str = first_status.text().unwrap_or_default().to_lowercase();
                match status_str.as_str() {
                    s if s.contains("en cours") => MangaStatus::Ongoing,
                    s if s.contains("terminé") => MangaStatus::Completed,
                    s if s.contains("abandonné") => MangaStatus::Cancelled,
                    s if s.contains("en pause") => MangaStatus::Hiatus,
                    _ => MangaStatus::Unknown,
                }
            } else {
                MangaStatus::Unknown
            }
        } else {
            MangaStatus::Unknown
        };

        // Extract tags/genres
        let tags = if let Some(genre_elems) = html.select(".seriestugenre a") {
            let mut genre_list = Vec::new();
            for genre in genre_elems {
                if let Some(genre_text) = genre.text() {
                    genre_list.push(genre_text.trim().to_string());
                }
            }
            if genre_list.is_empty() { None } else { Some(genre_list) }
        } else {
            None
        };

        let mut manga = Manga {
            key: key.clone(),
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            authors: author,
            artists: None,
            description,
            url: Some(format!("{}/manga/{}/", BASE_URL, key)),
            tags,
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

        // MangaStream chapter selectors
        if let Some(items) = html.select("#chapterlist li, .wp-manga-chapter") {
            for item in items {
                let link = if let Some(a_element) = item.select("a") {
                    if let Some(first_link) = a_element.first() {
                        first_link
                    } else {
                        continue;
                    }
                } else {
                    continue;
                };

                let href = link.attr("href").unwrap_or_default();
                if href.is_empty() {
                    continue;
                }

                let chapter_key = href
                    .replace(BASE_URL, "")
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();

                let title = link.text().unwrap_or_default().trim().to_string();
                if title.is_empty() {
                    continue;
                }

                // Extract chapter number
                let chapter_number = self.extract_chapter_number(&title);

                // Extract date
                let date_uploaded = if let Some(date_elem) = item.select(".chapterdate, .dt") {
                    if let Some(first_date) = date_elem.first() {
                        let date_str = first_date.text().unwrap_or_default();
                        self.parse_chapter_date(&date_str)
                    } else {
                        None
                    }
                } else {
                    None
                };

                let url = if href.starts_with("http") {
                    href
                } else if href.starts_with("/") {
                    format!("{}{}", BASE_URL, href)
                } else {
                    format!("{}/{}", BASE_URL, href)
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
        }

        Ok(chapters)
    }

    fn parse_page_list(&self, html: Document) -> Result<Vec<Page>> {
        let mut pages: Vec<Page> = Vec::new();

        // MangaStream page selectors with alt_pages support
        let selectors = [
            "div#readerarea img",
            ".rdminimal img",
            ".reader-area img",
            "#chapter_imgs img",
            ".chapter-content img"
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
        
        // Look for "chapitre" or "ch"
        if let Some(pos) = title_lower.find("chapitre") {
            let after_ch = &title[pos + 8..].trim(); // "chapitre" has 8 chars
            if let Some(num_str) = after_ch.split_whitespace().next() {
                if let Ok(num) = num_str.replace(',', ".").parse::<f32>() {
                    return num;
                }
            }
        }
        
        // Look for numbers in the title
        for word in title.split_whitespace() {
            if let Ok(num) = word.parse::<f32>() {
                return num;
            }
        }
        
        1.0 // Default
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }
        
        // French date parsing: "17 août 2025" format
        let parts: Vec<&str> = date_str.trim().split_whitespace().collect();
        if parts.len() >= 3 {
            if let (Ok(day), Ok(year)) = (parts[0].parse::<u32>(), parts[2].parse::<i32>()) {
                let month = match parts[1].to_lowercase().as_str() {
                    "janvier" => 1,
                    "février" => 2,
                    "mars" => 3,
                    "avril" => 4,
                    "mai" => 5,
                    "juin" => 6,
                    "juillet" => 7,
                    "août" => 8,
                    "septembre" => 9,
                    "octobre" => 10,
                    "novembre" => 11,
                    "décembre" => 12,
                    _ => return None,
                };
                
                // Calculate timestamp (simplified)
                let days_since_epoch = (year - 1970) * 365 + (month - 1) * 30 + day as i32 - 1;
                return Some(days_since_epoch as i64 * 86400);
            }
        }
        
        None
    }
}

register_source!(SushiScans, ListingProvider, ImageRequestProvider);