use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter,
    ContentRating, Viewer, UpdateStrategy,
    alloc::{String, Vec, vec, string::ToString},
    imports::html::Document,
};
use core::cmp::Ordering;
extern crate alloc;
use crate::helper;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    // Based on site analysis: find all manga links directly
    if let Some(manga_links) = html.select("a[href*='/lecture-en-ligne/']") {
        for link in manga_links {
            let url = link.attr("href").unwrap_or_default();
            if url.is_empty() {
                continue;
            }
            
            let slug = helper::extract_slug_from_url(&url);
            if slug.is_empty() {
                continue;
            }
            
            // Extract title from link text, handle "Lire le manga" prefix
            let raw_title = link.text().unwrap_or_default();
            let title = if raw_title.starts_with("Lire le manga ") {
                raw_title.replace("Lire le manga ", "").trim().to_string()
            } else {
                raw_title.trim().to_string()
            };
            
            // If title is empty, try to extract from URL slug as fallback
            let title = if title.is_empty() {
                slug.replace("-", " ")
                    .split_whitespace()
                    .map(|word| {
                        let mut chars = word.chars();
                        match chars.next() {
                            None => String::new(),
                            Some(first) => first.to_uppercase().chain(chars).collect(),
                        }
                    })
                    .collect::<Vec<String>>()
                    .join(" ")
            } else {
                title
            };
            
            if title.is_empty() {
                continue;
            }
            
            // Look for associated image and type in wider context
            let mut cover = None;
            let mut manga_type = String::new();
            
            // Try multiple strategies to find cover image
            let mut current = link.parent();
            for _level in 0..3 { // Go up to 3 levels up in DOM
                if let Some(element) = current {
                    if let Some(img) = element.select("img").and_then(|list| list.first()) {
                        let img_src = img.attr("src")
                            .or_else(|| img.attr("data-src"))
                            .or_else(|| img.attr("data-lazy-src"))
                            .unwrap_or_default();
                        
                        if !img_src.is_empty() {
                            cover = Some(helper::make_absolute_url("https://crunchyscan.fr", &img_src));
                            break;
                        }
                    }
                    
                    // Look for manga type in this level
                    if let Some(type_elem) = element.select("p").and_then(|list| list.first()) {
                        let type_text = type_elem.text().unwrap_or_default().trim().to_uppercase();
                        if type_text == "MANGA" || type_text == "MANHWA" || type_text == "MANHUA" {
                            manga_type = type_text;
                        }
                    }
                    
                    current = element.parent();
                } else {
                    break;
                }
            }
            
            mangas.push(Manga {
                key: slug.clone(),
                cover,
                title,
                authors: None,
                artists: None,
                description: None,
                tags: if !manga_type.is_empty() { Some(vec![manga_type]) } else { None },
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                url: Some(helper::build_manga_url(&slug)),
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            });
        }
    }
    
    // Remove duplicates based on key (simple approach for no_std)
    let mut unique_mangas = Vec::new();
    for manga in mangas {
        if !unique_mangas.iter().any(|m: &Manga| m.key == manga.key) {
            unique_mangas.push(manga);
        }
    }
    let mut mangas = unique_mangas;
    
    // Temporary fallback: if no mangas found, add a test entry to debug
    if mangas.is_empty() {
        mangas.push(Manga {
            key: "test-manga".to_string(),
            cover: None,
            title: "Test: Parsing found no manga links".to_string(),
            authors: None,
            artists: None,
            description: Some("This is a debug entry. If you see this, the HTML parsing failed to find manga links.".to_string()),
            tags: None,
            status: MangaStatus::Unknown,
            content_rating: ContentRating::Safe,
            viewer: Viewer::LeftToRight,
            chapters: None,
            url: Some("https://crunchyscan.fr".to_string()),
            next_update_time: None,
            update_strategy: UpdateStrategy::Always,
        });
    }
    
    let has_more = check_pagination(&html);

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: has_more,
    })
}

pub fn parse_manga_details(html: Document, manga_key: String) -> Result<Manga> {
    let title = if let Some(title_elem) = html.select("h1, .manga-title, .series-title").and_then(|list| list.first()) {
        helper::clean_title(&title_elem.text().unwrap_or_default())
    } else {
        String::new()
    };

    let description = if let Some(desc_elem) = html.select(".description, .synopsis, [class*='desc']").and_then(|list| list.first()) {
        Some(desc_elem.text().unwrap_or_default().trim().to_string())
    } else {
        None
    };

    let cover = if let Some(img) = html.select("img[class*='cover'], .manga-cover img, .series-cover img").and_then(|list| list.first()) {
        let img_src = img.attr("src")
            .or_else(|| img.attr("data-src"))
            .or_else(|| img.attr("data-lazy-src"))
            .unwrap_or_default();
        
        if !img_src.is_empty() {
            Some(helper::make_absolute_url("https://crunchyscan.fr", &img_src))
        } else {
            None
        }
    } else {
        None
    };

    let mut tags = Vec::new();
    if let Some(genre_elements) = html.select(".genre, .tag, [class*='genre'] a, [class*='tag'] a") {
        for genre in genre_elements {
            let tag_text_raw = genre.text().unwrap_or_default();
            let tag_text = tag_text_raw.trim();
            if !tag_text.is_empty() {
                tags.push(tag_text.to_string());
            }
        }
    }

    let status = if let Some(status_elem) = html.select(".status, [class*='status']").and_then(|list| list.first()) {
        let status_text = status_elem.text().unwrap_or_default().to_lowercase();
        if status_text.contains("terminé") || status_text.contains("complete") {
            MangaStatus::Completed
        } else if status_text.contains("en cours") || status_text.contains("ongoing") {
            MangaStatus::Ongoing
        } else if status_text.contains("abandonné") || status_text.contains("dropped") {
            MangaStatus::Cancelled
        } else {
            MangaStatus::Unknown
        }
    } else {
        MangaStatus::Unknown
    };

    Ok(Manga {
        key: manga_key,
        cover,
        title,
        authors: None,
        artists: None,
        description,
        tags: if tags.is_empty() { None } else { Some(tags) },
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::LeftToRight,
        chapters: None,
        url: None,
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    })
}

pub fn parse_chapter_list(html: Document, _manga_key: String) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();

    if let Some(chapter_elements) = html.select("a[href*='/read/'], .chapter-link, [class*='chapter'] a") {
        for chapter_elem in chapter_elements {
            let url = chapter_elem.attr("href").unwrap_or_default();
            if url.is_empty() || !url.contains("/read/") {
                continue;
            }

            let title = chapter_elem.text().unwrap_or_default().trim().to_string();
            if title.is_empty() {
                continue;
            }

            let chapter_number = helper::extract_chapter_number(&title);
            let volume = if title.to_lowercase().contains("volume") {
                Some(chapter_number)
            } else {
                None
            };

            let date_uploaded = if let Some(time_elem) = chapter_elem.parent()
                .and_then(|parent| parent.select(".time, .date, [class*='time'], [class*='date']").and_then(|list| list.first()))
            {
                let time_text = time_elem.text().unwrap_or_default();
                Some(helper::parse_relative_time(&time_text))
            } else {
                None
            };

            let chapter_key = if url.starts_with("http") {
                url.clone()
            } else {
                helper::make_absolute_url("https://crunchyscan.fr", &url)
            };

            chapters.push(Chapter {
                key: chapter_key.clone(),
                title: Some(title),
                chapter_number: Some(chapter_number),
                volume_number: volume,
                date_uploaded,
                scanlators: Some(vec!["CrunchyScan".to_string()]),
                language: Some("fr".to_string()),
                locked: false,
                thumbnail: None,
                url: Some(chapter_key),
            });
        }
    }

    chapters.sort_by(|a, b| {
        b.chapter_number.partial_cmp(&a.chapter_number).unwrap_or(Ordering::Equal)
    });

    Ok(chapters)
}

pub fn parse_page_list(html: Document) -> Result<Vec<Page>> {
    let mut pages = Vec::new();

    if let Some(img_elements) = html.select("img[class*='page'], .page img, #reader img") {
        for (_index, img) in img_elements.enumerate() {
            let img_src = img.attr("src")
                .or_else(|| img.attr("data-src"))
                .or_else(|| img.attr("data-lazy-src"))
                .unwrap_or_default();

            if !img_src.is_empty() {
                let absolute_url = helper::make_absolute_url("https://crunchyscan.fr", &img_src);
                pages.push(Page {
                    content: PageContent::Url(absolute_url, None),
                    thumbnail: None,
                    has_description: false,
                    description: None,
                });
            }
        }
    }

    if pages.is_empty() {
        if let Some(scripts) = html.select("script") {
            for script in scripts {
                if let Some(script_content) = script.text() {
                    if script_content.contains("pages") || script_content.contains("images") {
                        let lines: Vec<&str> = script_content.lines().collect();
                        for line in lines {
                            if (line.contains(".jpg") || line.contains(".png") || line.contains(".jpeg"))
                                && (line.contains("http") || line.contains("//"))
                            {
                                if let Some(start) = line.find("\"http") {
                                    if let Some(end) = line[start + 1..].find("\"") {
                                        let url = &line[start + 1..start + 1 + end];
                                        pages.push(Page {
                                            content: PageContent::Url(url.to_string(), None),
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

pub fn search_manga(search_json: &str) -> Result<MangaPageResult> {
    let mut mangas = Vec::new();
    
    // Simple JSON parsing without serde - look for title patterns
    let lines: Vec<&str> = search_json.lines().collect();
    for line in lines {
        if line.contains("\"title\"") && line.contains("\"slug\"") {
            // Extract title between quotes
            if let Some(title_start) = line.find("\"title\":\"") {
                let title_offset = title_start + 9;
                if let Some(title_end) = line[title_offset..].find("\"") {
                    let title = &line[title_offset..title_offset + title_end];
                    
                    // Extract slug
                    if let Some(slug_start) = line.find("\"slug\":\"") {
                        let slug_offset = slug_start + 8;
                        if let Some(slug_end) = line[slug_offset..].find("\"") {
                            let slug = &line[slug_offset..slug_offset + slug_end];
                            
                            mangas.push(Manga {
                                key: slug.to_string(),
                                cover: None,
                                title: title.to_string(),
                                authors: None,
                                artists: None,
                                description: None,
                                tags: None,
                                status: MangaStatus::Unknown,
                                content_rating: ContentRating::Safe,
                                viewer: Viewer::LeftToRight,
                                chapters: None,
                                url: Some(helper::build_manga_url(slug)),
                                next_update_time: None,
                                update_strategy: UpdateStrategy::Always,
                            });
                        }
                    }
                }
            }
        }
    }

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: false,
    })
}

fn check_pagination(html: &Document) -> bool {
    // First, check the specific CrunchyScan pagination structure
    if let Some(p_elements) = html.select("p") {
        for elem in p_elements {
            if let Some(text) = elem.text() {
                if text.contains("/") && text.chars().any(|c| c.is_ascii_digit()) {
                    if let Some(slash_pos) = text.find("/") {
                        let after_slash = &text[slash_pos + 1..].trim();
                        if let Ok(total_pages) = after_slash.parse::<i32>() {
                            // Also try to get current page from textbox
                            if let Some(textbox) = html.select("input[type='text']").and_then(|list| list.first()) {
                                if let Some(value) = textbox.attr("value") {
                                    if let Ok(current_page) = value.parse::<i32>() {
                                        return current_page < total_pages;
                                    }
                                }
                            }
                            // If we can't find current page, assume we're on page 1 and there are more if total > 1
                            return total_pages > 1;
                        }
                    }
                }
            }
        }
    }

    // Fallback to the original logic
    if let Some(page_elements) = html.select("input[type='text'], .paginate_button, [class*='page']") {
        for elem in page_elements {
            if let Some(text) = elem.text() {
                if text.contains("/") {
                    let parts: Vec<&str> = text.split("/").collect();
                    if parts.len() == 2 {
                        if let (Ok(current), Ok(total)) = (parts[0].trim().parse::<i32>(), parts[1].trim().parse::<i32>()) {
                            return current < total;
                        }
                    }
                }
            }

            if let Some(value) = elem.attr("value") {
                if let Ok(current_page) = value.parse::<i32>() {
                    if let Some(parent) = elem.parent() {
                        if let Some(total_text) = parent.text() {
                            if total_text.contains("/") {
                                let parts: Vec<&str> = total_text.split("/").collect();
                                if parts.len() == 2 {
                                    if let Ok(total) = parts[1].trim().parse::<i32>() {
                                        return current_page < total;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Check for next page buttons as final fallback
    if let Some(next_links) = html.select("a[href*='page='], .next, [class*='next']") {
        return !next_links.is_empty();
    }

    false
}