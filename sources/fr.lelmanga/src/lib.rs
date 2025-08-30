#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, MangaStatus, 
    Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec},
    imports::{net::Request, html::Document},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString, vec};

pub static BASE_URL: &str = "https://www.lelmanga.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Mobile/15E148 Safari/604.1";

pub struct LelManga;

impl Source for LelManga {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        _filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {
        println!("[LelManga] get_search_manga_list called - query: {:?}, page: {}", query, page);

        let url = if let Some(search_query) = query {
            if search_query.is_empty() {
                format!("{}/manga/?page={}", BASE_URL, page)
            } else {
                format!("{}/?s={}&page={}", BASE_URL, search_query, page)
            }
        } else {
            format!("{}/manga/?page={}", BASE_URL, page)
        };

        println!("[LelManga] Search URL: {}", url);
        self.get_manga_from_page(&url)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        println!("[LelManga] get_manga_update called - key: {}", manga.key);

        let url = format!("{}/manga/{}/", BASE_URL, manga.key);
        println!("[LelManga] Details URL: {}", url);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        // Parse manga details
        let mut updated_manga = self.parse_manga_details(manga.key.clone(), &html)?;

        // Parse chapters if needed
        if needs_chapters {
            let chapters = self.parse_chapter_list(manga.key, &html)?;
            updated_manga.chapters = Some(chapters);
        }

        Ok(updated_manga)
    }

    fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
        println!("[LelManga] get_page_list called - manga: {}, chapter: {}", manga.key, chapter.key);

        let url = format!("{}/{}/", BASE_URL, chapter.key);
        println!("[LelManga] Page list URL: {}", url);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        self.parse_page_list(&html)
    }
}

impl ListingProvider for LelManga {
    fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
        println!("[LelManga] get_manga_list called - listing: {}, page: {}", listing.name, page);

        let mut url = format!("{}/manga/", BASE_URL);

        // Add sorting parameter based on listing type
        match listing.name.as_str() {
            "Populaire" => url.push_str("?order=popular"),
            "Tendance" => url.push_str("?order=update"),
            _ => url.push_str("?order=latest"),
        }

        // Add page parameter
        if page > 1 {
            if url.contains('?') {
                url.push_str(&format!("&page={}", page));
            } else {
                url.push_str(&format!("?page={}", page));
            }
        }

        println!("[LelManga] Listing URL: {}", url);
        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for LelManga {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
        println!("[LelManga] get_image_request called for: {}", url);

        Ok(Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Referer", BASE_URL)
            .header("Accept", "image/avif,image/webp,image/png,image/jpeg,*/*")
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Sec-Fetch-Dest", "image")
            .header("Sec-Fetch-Mode", "no-cors")
            .header("Sec-Fetch-Site", "same-origin"))
    }
}

impl LelManga {
    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        let mut entries: Vec<Manga> = Vec::new();

        // Use MangaThemesia selectors
        let selector = ".utao .uta .imgu, .listupd .bs .bsx, .page-listing-item";
        println!("[LelManga] Using selector: {}", selector);

        if let Some(items) = html.select(selector) {
            let items_vec: Vec<_> = items.collect();
            println!("[LelManga] Found {} manga items", items_vec.len());

            for item in items_vec {
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

                // Extract manga ID from URL
                let key = href
                    .replace(BASE_URL, "")
                    .replace("/manga/", "")
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();

                if key.is_empty() {
                    continue;
                }

                let title = if let Some(title_val) = link.attr("title") {
                    if !title_val.is_empty() {
                        title_val
                    } else if let Some(title_elem) = item.select("a .slide-caption h3, .bsx h3, .post-title h3") {
                        if let Some(first_title) = title_elem.first() {
                            first_title.text().unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                if title.is_empty() {
                    continue;
                }

                // Get cover image
                let cover = if let Some(img_elements) = item.select("img") {
                    if let Some(img) = img_elements.first() {
                        if let Some(lazy_src) = img.attr("data-lazy-src") {
                            if !lazy_src.is_empty() {
                                lazy_src
                            } else if let Some(data_src) = img.attr("data-src") {
                                if !data_src.is_empty() {
                                    data_src
                                } else {
                                    img.attr("src").unwrap_or_default()
                                }
                            } else {
                                img.attr("src").unwrap_or_default()
                            }
                        } else {
                            img.attr("src").unwrap_or_default()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                };

                println!("[LelManga] Found manga: key={}, title={}", key, title);

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

        // Check for pagination
        let has_next_page = html.select(".pagination .next, .hpage .r").is_some();
        println!("[LelManga] Found {} manga, has_next_page: {}", entries.len(), has_next_page);

        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }

    fn parse_manga_details(&self, key: String, html: &Document) -> Result<Manga> {
        println!("[LelManga] parse_manga_details called - key: {}", key);

        // Extract title with MangaThemesia selectors
        let title = if let Some(container) = html.select("div.bigcontent, div.animefull, div.main-info, div.postbody") {
            if let Some(title_elem) = container.select("h1.entry-title") {
                if let Some(first_title) = title_elem.first() {
                    let title_text = first_title.text().unwrap_or_default();
                    if !title_text.is_empty() {
                        title_text
                    } else if let Some(breadcrumb) = html.select(".ts-breadcrumb li:last-child span") {
                        if let Some(first_breadcrumb) = breadcrumb.first() {
                            first_breadcrumb.text().unwrap_or_default()
                        } else {
                            String::new()
                        }
                    } else {
                        String::new()
                    }
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        let title = if title.is_empty() {
            if let Some(h1_elem) = html.select("h1") {
                if let Some(first_h1) = h1_elem.first() {
                    first_h1.text().unwrap_or_default()
                } else {
                    String::new()
                }
            } else {
                String::new()
            }
        } else {
            title
        };

        println!("[LelManga] Extracted title: {}", title);

        if title.is_empty() {
            return Err(aidoku::AidokuError::Unimplemented);
        }

        // Extract cover image with multiple selectors
        let cover = if let Some(cover_elem) = html.select(".infomanga > div[itemprop=image] img, .thumb img, .manga-poster img, .post-thumb img, .series-thumb img, img.attachment-post-thumbnail, .wp-post-image, .post-content img:first-child") {
            if let Some(first_cover) = cover_elem.first() {
                let src = first_cover.attr("data-lazy-src")
                    .or_else(|| first_cover.attr("data-src"))
                    .or_else(|| first_cover.attr("src"))
                    .unwrap_or_default();
                src
            } else {
                String::new()
            }
        } else {
            println!("[LelManga] No cover image found");
            String::new()
        };

        println!("[LelManga] Extracted cover: {}", cover);

        // Extract author and artist
        let (authors, artists) = if let Some(container) = html.select("div.bigcontent, div.animefull, div.main-info, div.postbody") {
            let author = if let Some(author_elem) = container.select(".imptdt:contains(Auteur) i") {
                if let Some(first_author) = author_elem.first() {
                    let author_text = first_author.text().unwrap_or_default();
                    if author_text.is_empty() { None } else { Some(vec![author_text]) }
                } else {
                    None
                }
            } else {
                None
            };

            let artist = if let Some(artist_elem) = container.select(".imptdt:contains(Artiste) i") {
                if let Some(first_artist) = artist_elem.first() {
                    let artist_text = first_artist.text().unwrap_or_default();
                    if artist_text.is_empty() { None } else { Some(vec![artist_text]) }
                } else {
                    None
                }
            } else {
                None
            };

            (author, artist)
        } else {
            (None, None)
        };

        // Extract description with multiple selectors
        let description = if let Some(desc_elem) = html.select(".desc, .entry-content[itemprop=description], .summary__content, .manga-summary, .post-content_item .summary-content, .description, .synopsis, .sinopsis, .summary, .post-excerpt") {
            if let Some(first_desc) = desc_elem.first() {
                let desc_text = first_desc.text().unwrap_or_default();
                println!("[LelManga] Found description: {}", if desc_text.len() > 50 { format!("{}...", &desc_text[..50]) } else { desc_text.clone() });
                desc_text
            } else {
                println!("[LelManga] Description element found but no content");
                String::new()
            }
        } else {
            println!("[LelManga] No description element found");
            String::new()
        };

        println!("[LelManga] Extracted description length: {}", description.len());

        // Extract genres
        let mut tags: Vec<String> = Vec::new();
        if let Some(container) = html.select("div.bigcontent, div.animefull, div.main-info, div.postbody") {
            if let Some(genre_elements) = container.select("div.gnr a, .mgen a, .seriestugenre a") {
                for genre_element in genre_elements {
                    let genre = genre_element.text().unwrap_or_default();
                    if !genre.is_empty() {
                        tags.push(genre);
                    }
                }
            }
        }

        println!("[LelManga] Extracted {} tags", tags.len());

        // Extract status
        let status = if let Some(container) = html.select("div.bigcontent, div.animefull, div.main-info, div.postbody") {
            if let Some(status_elem) = container.select("div.post-content_item:contains(Statut) div.summary-content") {
                if let Some(first_status) = status_elem.first() {
                    let status_str = first_status.text().unwrap_or_default().trim().to_lowercase();
                    match status_str.as_str() {
                        "en cours" | "ongoing" => MangaStatus::Ongoing,
                        "terminé" | "completed" => MangaStatus::Completed,
                        "annulé" | "cancelled" => MangaStatus::Cancelled,
                        "en pause" | "hiatus" => MangaStatus::Hiatus,
                        _ => MangaStatus::Unknown,
                    }
                } else {
                    MangaStatus::Unknown
                }
            } else {
                MangaStatus::Unknown
            }
        } else {
            MangaStatus::Unknown
        };

        println!("[LelManga] Extracted status: {:?}", status);

        // Extract content rating
        let suggestive_tags = ["ecchi", "mature", "adult"];
        let content_rating = if tags.iter().any(|v| suggestive_tags.iter().any(|tag| v.to_lowercase() == *tag)) {
            ContentRating::Suggestive
        } else {
            ContentRating::Safe
        };

        Ok(Manga {
            key: key.clone(),
            title,
            cover: if cover.is_empty() { None } else { Some(cover) },
            authors,
            artists,
            description: if description.is_empty() { None } else { Some(description) },
            url: Some(format!("{}/manga/{}/", BASE_URL, key)),
            tags: if tags.is_empty() { None } else { Some(tags) },
            status,
            content_rating,
            viewer: Viewer::RightToLeft,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        })
    }

    fn parse_chapter_list(&self, manga_key: String, html: &Document) -> Result<Vec<Chapter>> {
        println!("[LelManga] parse_chapter_list called - manga_key: {}", manga_key);

        let mut chapters: Vec<Chapter> = Vec::new();

        // Use MangaThemesia selectors for chapters with more alternatives
        let selector = "div.bxcl li, div.cl li, #chapterlist li, ul li:has(div.chbox):has(div.eph-num), .chapter-list li, .wp-manga-chapter, .manga-chapters li, li.wp-manga-chapter";
        println!("[LelManga] Using chapter selector: {}", selector);

        if let Some(items) = html.select(selector) {
            let items_vec: Vec<_> = items.collect();
            println!("[LelManga] Found {} chapter items", items_vec.len());

            for item in items_vec {
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

                // Extract chapter ID from URL
                let chapter_key = href
                    .replace(BASE_URL, "")
                    .trim_start_matches('/')
                    .trim_end_matches('/')
                    .to_string();

                if chapter_key.is_empty() {
                    continue;
                }

                // Get chapter title with fallbacks
                let title = if let Some(lch_elem) = item.select(".lch a") {
                    if let Some(first_lch) = lch_elem.first() {
                        let lch_text = first_lch.text().unwrap_or_default();
                        if !lch_text.is_empty() {
                            lch_text
                        } else {
                            link.text().unwrap_or_default()
                        }
                    } else {
                        link.text().unwrap_or_default()
                    }
                } else if let Some(chnum_elem) = item.select(".chapternum") {
                    if let Some(first_chnum) = chnum_elem.first() {
                        let chnum_text = first_chnum.text().unwrap_or_default();
                        if !chnum_text.is_empty() {
                            chnum_text
                        } else {
                            link.text().unwrap_or_default()
                        }
                    } else {
                        link.text().unwrap_or_default()
                    }
                } else {
                    link.text().unwrap_or_default()
                };

                if title.is_empty() {
                    continue;
                }

                // Clean title and extract date if present
                let (clean_title, extracted_date) = self.clean_chapter_title_and_extract_date(&title);
                println!("[LelManga] Original title: {}", title);
                println!("[LelManga] Clean title: {}", clean_title);

                // Extract chapter number from URL or title
                let chapter_number = self.extract_chapter_number(&chapter_key, &clean_title);
                println!("[LelManga] Chapter: key={}, title={}, number={}", chapter_key, clean_title, chapter_number);

                // Parse chapter date with multiple selectors, prioritizing extracted date from title
                let date_uploaded = if let Some(extracted) = extracted_date {
                    println!("[LelManga] Using date from title: {:?}", extracted);
                    Some(extracted)
                } else if let Some(date_elem) = item.select(".chapterdate, .dt, .chapter-date, .date, span.dt, .chapter-release-date") {
                    if let Some(first_date) = date_elem.first() {
                        let date_str = first_date.text().unwrap_or_default();
                        println!("[LelManga] Found chapter date in element: {}", date_str);
                        self.parse_chapter_date(&date_str)
                    } else {
                        None
                    }
                } else {
                    println!("[LelManga] No date found for chapter: {}", clean_title);
                    None
                };

                // Ensure URL is absolute
                let url = if href.starts_with("http") {
                    href
                } else if href.starts_with("/") {
                    format!("{}{}", BASE_URL, href)
                } else {
                    format!("{}/{}", BASE_URL, href)
                };

                chapters.push(Chapter {
                    key: chapter_key,
                    title: Some(clean_title),
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

        println!("[LelManga] Returning {} chapters", chapters.len());
        Ok(chapters)
    }

    fn extract_chapter_number(&self, chapter_id: &str, title: &str) -> f32 {
        let chapter_id_lower = chapter_id.to_lowercase();

        // Look for "chapitre-" or "chapter-" pattern
        if let Some(chapitre_pos) = chapter_id_lower.find("chapitre-") {
            let after_chapitre = &chapter_id[chapitre_pos + 9..];
            if let Some(num_str) = after_chapitre.split('-').next() {
                if let Ok(num) = num_str.parse::<f32>() {
                    println!("[LelManga] Extracted chapter number from 'chapitre-': {}", num);
                    return num;
                }
            }
        }

        if let Some(chapter_pos) = chapter_id_lower.find("chapter-") {
            let after_chapter = &chapter_id[chapter_pos + 8..];
            if let Some(num_str) = after_chapter.split('-').next() {
                if let Ok(num) = num_str.parse::<f32>() {
                    println!("[LelManga] Extracted chapter number from 'chapter-': {}", num);
                    return num;
                }
            }
        }

        // lelmanga format: manga-name-XXX-Y where XXX is chapter and Y is version
        let parts: Vec<&str> = chapter_id.split('-').collect();
        if parts.len() >= 2 {
            let last_part = parts[parts.len() - 1];
            let second_last_part = parts[parts.len() - 2];

            if let (Ok(last_num), Ok(second_last_num)) = (last_part.parse::<f32>(), second_last_part.parse::<f32>()) {
                if last_num <= 20.0 && second_last_num >= last_num {
                    println!("[LelManga] Extracted chapter number from pattern: {}", second_last_num);
                    return second_last_num;
                }
            }
        }

        // Fallback: last segment
        if let Some(num_str) = chapter_id.split('-').last() {
            if let Ok(num) = num_str.parse::<f32>() {
                println!("[LelManga] Extracted chapter number from last segment: {}", num);
                return num;
            }
        }

        // Extract from title
        let words: Vec<&str> = title.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase().contains("chapitre") || word.to_lowercase().contains("chapter") {
                if i + 1 < words.len() {
                    if let Ok(num) = words[i + 1].parse::<f32>() {
                        println!("[LelManga] Extracted chapter number from title: {}", num);
                        return num;
                    }
                }
            }
        }

        // Last resort: any number in title
        for word in words {
            if let Ok(num) = word.parse::<f32>() {
                println!("[LelManga] Extracted chapter number from any title number: {}", num);
                return num;
            }
        }

        println!("[LelManga] Could not extract chapter number, using -1.0");
        -1.0
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }

        println!("[LelManga] Parsing date: {}", date_str);

        // Simple date parsing for English months
        let months = [
            ("January", 1), ("February", 2), ("March", 3), ("April", 4),
            ("May", 5), ("June", 6), ("July", 7), ("August", 8),
            ("September", 9), ("October", 10), ("November", 11), ("December", 12)
        ];

        let parts: Vec<&str> = date_str.trim().split_whitespace().collect();
        if parts.len() >= 3 {
            let month_name = parts[0];
            let day_str = parts[1].trim_end_matches(',');
            let year_str = parts[2];

            if let Some((_, month)) = months.iter().find(|(name, _)| name.eq_ignore_ascii_case(month_name)) {
                if let (Ok(day), Ok(year)) = (day_str.parse::<i32>(), year_str.parse::<i32>()) {
                    let days_since_1970 = (year - 1970) * 365 + (month - 1) * 30 + day;
                    let timestamp = (days_since_1970 as i64) * 86400; // seconds in a day
                    println!("[LelManga] Parsed date to timestamp: {}", timestamp);
                    return Some(timestamp);
                }
            }
        }

        None
    }

    fn parse_page_list(&self, html: &Document) -> Result<Vec<Page>> {
        println!("[LelManga] parse_page_list called");

        let mut pages: Vec<Page> = Vec::new();

        // First try: HTML images
        println!("[LelManga] Trying HTML image parsing");
        if let Some(img_elements) = html.select("div#readerarea img") {
            for img_element in img_elements {
                let img_url = if let Some(lazy_src) = img_element.attr("data-lazy-src") {
                    if !lazy_src.is_empty() {
                        lazy_src
                    } else if let Some(data_src) = img_element.attr("data-src") {
                        if !data_src.is_empty() {
                            data_src
                        } else {
                            img_element.attr("src").unwrap_or_default()
                        }
                    } else {
                        img_element.attr("src").unwrap_or_default()
                    }
                } else {
                    img_element.attr("src").unwrap_or_default()
                };

                if !img_url.is_empty() {
                    println!("[LelManga] Found HTML image: {}", img_url);
                    pages.push(Page {
                        content: PageContent::url(img_url),
                        thumbnail: None,
                        has_description: false,
                        description: None,
                    });
                }
            }
        }

        if !pages.is_empty() {
            println!("[LelManga] HTML parsing successful, found {} pages", pages.len());
            return Ok(pages);
        }

        // Second try: JavaScript parsing
        println!("[LelManga] Trying JavaScript parsing");
        let html_content = if let Some(script_elem) = html.select("script:contains(ts_reader.run)") {
            if let Some(first_script) = script_elem.first() {
                first_script.text().unwrap_or_default()
            } else {
                String::new()
            }
        } else {
            String::new()
        };

        if let Some(ts_reader_start) = html_content.find("ts_reader.run({") {
            println!("[LelManga] Found ts_reader.run configuration");
            let config_start = ts_reader_start + 15;
            if let Some(config_end) = html_content[config_start..].find("})") {
                let config_content = &html_content[config_start..config_start + config_end];

                if let Some(sources_start) = config_content.find("\"sources\":[") {
                    let sources_section = &config_content[sources_start + 11..];

                    if let Some(images_start) = sources_section.find("\"images\":[") {
                        let images_section = &sources_section[images_start + 10..];
                        if let Some(images_end) = images_section.find("]") {
                            let images_content = &images_section[..images_end];

                            let parts: Vec<&str> = images_content.split('\"').collect();

                            for part in parts {
                                if part.starts_with("https://") && part.contains("/wp-content/uploads/") && 
                                   (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp") || part.ends_with(".jpeg")) {
                                    println!("[LelManga] Found JS image: {}", part);
                                    pages.push(Page {
                                        content: PageContent::url(part.to_string()),
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

        // Fallback: general images pattern
        if pages.is_empty() {
            println!("[LelManga] Trying fallback JavaScript parsing");
            if let Some(images_pattern_start) = html_content.find("\"images\":[") {
                let images_section = &html_content[images_pattern_start + 10..];
                if let Some(images_end) = images_section.find("]") {
                    let images_content = &images_section[..images_end];

                    let parts: Vec<&str> = images_content.split('\"').collect();

                    for part in parts {
                        if part.starts_with("https://") && part.contains("lelmanga.com") && 
                           (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp")) {
                            println!("[LelManga] Found fallback image: {}", part);
                            pages.push(Page {
                                content: PageContent::url(part.to_string()),
                                thumbnail: None,
                                has_description: false,
                                description: None,
                            });
                        }
                    }
                }
            }
        }

        println!("[LelManga] Returning {} pages", pages.len());
        Ok(pages)
    }

    fn clean_chapter_title_and_extract_date(&self, raw_title: &str) -> (String, Option<i64>) {
        println!("[LelManga] Cleaning title: {}", raw_title);
        
        let mut clean_title = raw_title.to_string();
        let mut extracted_date = None;
        
        // Extract date from title - look for patterns like "August 29, 2025"
        let words: Vec<&str> = raw_title.split_whitespace().collect();
        if words.len() >= 3 {
            // Check last 3 words for date pattern
            let last_3_words = &words[words.len()-3..];
            let date_str = last_3_words.join(" ");
            
            if let Some(parsed_date) = self.parse_chapter_date(&date_str) {
                println!("[LelManga] Extracted date from title: {} -> {}", date_str, parsed_date);
                extracted_date = Some(parsed_date);
                
                // Remove the date part from title
                if let Some(date_pos) = raw_title.rfind(&date_str) {
                    clean_title = raw_title[..date_pos].trim().to_string();
                }
            } else {
                // Try different combinations for dates
                for i in (0..words.len().saturating_sub(2)).rev() {
                    let potential_date = words[i..i+3].join(" ");
                    if let Some(parsed_date) = self.parse_chapter_date(&potential_date) {
                        println!("[LelManga] Extracted date from title: {} -> {}", potential_date, parsed_date);
                        extracted_date = Some(parsed_date);
                        
                        if let Some(date_pos) = raw_title.rfind(&potential_date) {
                            clean_title = raw_title[..date_pos].trim().to_string();
                        }
                        break;
                    }
                }
            }
        }
        
        // Additional cleanup: remove trailing punctuation and whitespace
        clean_title = clean_title.trim_end_matches(&['-', '–', '—', ':', ',', '.', ' ']).to_string();
        
        // Remove redundant "Ch.XX -" prefix if it exists
        if let Some(dash_pos) = clean_title.find(" - ") {
            if clean_title[..dash_pos].starts_with("Ch.") {
                clean_title = clean_title[dash_pos + 3..].to_string();
            }
        }
        
        println!("[LelManga] Title cleaned from '{}' to '{}'", raw_title, clean_title);
        (clean_title, extracted_date)
    }
}

register_source!(LelManga, ListingProvider, ImageRequestProvider);