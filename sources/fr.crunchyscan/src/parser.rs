use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter,
    ContentRating, Viewer, UpdateStrategy,
    alloc::{String, Vec, vec, string::ToString, format},
    imports::html::Document,
};
use core::cmp::Ordering;
extern crate alloc;
use crate::helper;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    // Target the specific manga container structure (with typo "containter")
    if let Some(manga_container) = html.select("#advanced_manga_containter") {
        if let Some(manga_items) = manga_container.first() {
            // Check if container only contains loading spinner
            if let Some(loading_content) = manga_items.select("p.loadingContent") {
                if !loading_content.is_empty() {
                    // Container has loading content, this means dynamic loading is still happening
                    // Return empty result to trigger API fallback
                    return Ok(MangaPageResult {
                        entries: Vec::new(),
                        has_next_page: false,
                    });
                }
            }
            
            if let Some(manga_divs) = manga_items.select("div.flex.flex-col.w-full.gap-3") {
                for manga_div in manga_divs {
                    // Find the main manga link (not chapter links)
                    if let Some(manga_links) = manga_div.select("a[href*='/lecture-en-ligne/']") {
                        let mut main_link = None;
                        for link in manga_links {
                            let href = link.attr("href").unwrap_or_default();
                            if !href.contains("/read/") && !href.contains("?") && !href.contains("#") {
                                main_link = Some(link);
                                break;
                            }
                        }
                        
                        if let Some(main_link) = main_link {
                            let url = main_link.attr("href").unwrap_or_default();
                            let key = helper::extract_slug_from_url(&url);
                            
                            if key.is_empty() {
                                continue;
                            }
                            
                            // Extract title from the title link (not the image link)
                            let title = if let Some(title_links) = manga_div.select("a.font-bold.text-lg.truncate") {
                                if let Some(title_link) = title_links.first() {
                                    let raw_title = title_link.text().unwrap_or_default();
                                    if raw_title.starts_with("Lire le manga ") {
                                        raw_title.replace("Lire le manga ", "").trim().to_string()
                                    } else {
                                        raw_title.trim().to_string()
                                    }
                                } else {
                                    // Fallback: generate from slug
                                    key.replace("-", " ")
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
                                }
                            } else {
                                // Try alternative title selection
                                if let Some(title_links) = manga_div.select("a[title*='Lire le manga']") {
                                    if let Some(title_link) = title_links.first() {
                                        let title_attr = title_link.attr("title").unwrap_or_default();
                                        if title_attr.starts_with("Lire le manga ") {
                                            title_attr.replace("Lire le manga ", "").trim().to_string()
                                        } else {
                                            title_attr.trim().to_string()
                                        }
                                    } else {
                                        key.replace("-", " ")
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
                                    }
                                } else {
                                    key.replace("-", " ")
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
                                }
                            };
                            
                            // Extract cover image
                            let cover = if let Some(img) = manga_div.select("img").and_then(|imgs| imgs.first()) {
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
                            
                            // Extract manga type from tag (MANGA/MANHWA/MANHUA)
                            let manga_type = if let Some(tag_elem) = manga_div.select("p.tag").and_then(|tags| tags.first()) {
                                tag_elem.text().unwrap_or_default().trim().to_string()
                            } else {
                                String::new()
                            };
                            
                            // Create tags from manga type if available
                            let tags = if !manga_type.is_empty() {
                                Some(vec![manga_type])
                            } else {
                                None
                            };
                            
                            // Create manga entry
                            mangas.push(Manga {
                                key: key.clone(),
                                cover,
                                title,
                                authors: None,
                                artists: None,
                                description: None,
                                tags,
                                status: MangaStatus::Unknown,
                                content_rating: ContentRating::Safe,
                                viewer: Viewer::LeftToRight,
                                chapters: None,
                                url: Some(helper::make_absolute_url("https://crunchyscan.fr", &url)),
                                next_update_time: None,
                                update_strategy: UpdateStrategy::Always,
                            });
                        }
                    }
                }
            }
        }
    }

    // If no mangas found, try alternative selectors as fallback
    if mangas.is_empty() {
        // Try alternative container selectors in case the structure changed
        let alternative_selectors = [
            "#advanced_manga_container", // Fix potential typo
            ".grid.grid-cols-2", // Direct grid selector
            "[class*='manga_container']", // Any class containing manga_container
            ".manga-list", // Generic manga list class
        ];
        
        for selector in &alternative_selectors {
            if let Some(container) = html.select(selector) {
                if let Some(container_elem) = container.first() {
                    if let Some(manga_divs) = container_elem.select("div.flex.flex-col.w-full.gap-3") {
                        for manga_div in manga_divs {
                            if let Some(manga_links) = manga_div.select("a[href*='/lecture-en-ligne/']") {
                                let mut main_link = None;
                                for link in manga_links {
                                    let href = link.attr("href").unwrap_or_default();
                                    if !href.contains("/read/") && !href.contains("?") && !href.contains("#") {
                                        main_link = Some(link);
                                        break;
                                    }
                                }
                                
                                if let Some(main_link) = main_link {
                                    let url = main_link.attr("href").unwrap_or_default();
                                    let key = helper::extract_slug_from_url(&url);
                                    
                                    if !key.is_empty() {
                                        // Use simplified parsing for fallback
                                        let title = main_link.attr("title")
                                            .unwrap_or_default()
                                            .replace("Lire le manga ", "")
                                            .trim()
                                            .to_string();
                                        
                                        if !title.is_empty() {
                                            mangas.push(Manga {
                                                key: key.clone(),
                                                cover: None,
                                                title,
                                                authors: None,
                                                artists: None,
                                                description: None,
                                                tags: None,
                                                status: MangaStatus::Unknown,
                                                content_rating: ContentRating::Safe,
                                                viewer: Viewer::LeftToRight,
                                                chapters: None,
                                                url: Some(helper::make_absolute_url("https://crunchyscan.fr", &url)),
                                                next_update_time: None,
                                                update_strategy: UpdateStrategy::Always,
                                            });
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
                // If we found any mangas with this selector, break and use them
                if !mangas.is_empty() {
                    break;
                }
            }
        }
        
        // If still no mangas found, return empty to trigger API fallback
        if mangas.is_empty() {
            return Ok(MangaPageResult {
                entries: mangas,
                has_next_page: false,
            });
        }
    }

    // Check for pagination - following LelscanFR pattern
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

pub fn parse_api_response(api_response: &str) -> Result<MangaPageResult> {
    let mut mangas = Vec::new();
    
    // Simple JSON parsing without serde for the API response
    // Look for "data" array containing manga objects
    if let Some(data_start) = api_response.find("\"data\":[") {
        let data_content = &api_response[data_start + 8..];
        
        // Find individual manga objects within the data array
        let mut current_pos = 0;
        while let Some(obj_start) = data_content[current_pos..].find("{\"id\":") {
            let absolute_start = current_pos + obj_start;
            
            // Find the end of this object by counting braces
            let mut brace_count = 0;
            let mut obj_end = absolute_start;
            let chars: Vec<char> = data_content.chars().collect();
            
            for (i, &ch) in chars[absolute_start..].iter().enumerate() {
                match ch {
                    '{' => brace_count += 1,
                    '}' => {
                        brace_count -= 1;
                        if brace_count == 0 {
                            obj_end = absolute_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            
            if obj_end > absolute_start {
                let obj_str = &data_content[absolute_start..=obj_end];
                
                // Extract manga data from this object
                if let (Some(name), Some(slug)) = (extract_json_string(obj_str, "name"), extract_json_string(obj_str, "slug")) {
                    let cover_url = extract_json_string(obj_str, "cover_url");
                    let description = extract_json_string(obj_str, "synopsis");
                    let manga_type = extract_json_string(obj_str, "type").unwrap_or_default();
                    
                    let cover = if let Some(cover) = cover_url {
                        if cover.starts_with("http") {
                            Some(cover)
                        } else {
                            Some(helper::make_absolute_url("https://crunchyscan.fr", &cover))
                        }
                    } else {
                        None
                    };
                    
                    // Determine status from type or other fields
                    let status = if manga_type.to_lowercase().contains("ongoing") {
                        MangaStatus::Ongoing
                    } else if manga_type.to_lowercase().contains("completed") {
                        MangaStatus::Completed
                    } else {
                        MangaStatus::Unknown
                    };
                    
                    mangas.push(Manga {
                        key: slug.clone(),
                        cover,
                        title: name,
                        authors: None,
                        artists: None,
                        description,
                        tags: None,
                        status,
                        content_rating: ContentRating::Safe,
                        viewer: Viewer::LeftToRight,
                        chapters: None,
                        url: Some(helper::build_manga_url(&slug)),
                        next_update_time: None,
                        update_strategy: UpdateStrategy::Always,
                    });
                }
            }
            
            current_pos = obj_end + 1;
        }
    }
    
    // Check for pagination info
    let has_more = check_api_pagination(api_response);
    
    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: has_more,
    })
}

// Helper function to extract string values from JSON
fn extract_json_string(json: &str, key: &str) -> Option<String> {
    let pattern = &format!("\"{}\":\"", key);
    if let Some(start) = json.find(pattern) {
        let value_start = start + pattern.len();
        if let Some(end) = json[value_start..].find("\"") {
            let value = &json[value_start..value_start + end];
            // Unescape basic JSON escape sequences
            let unescaped = value.replace("\\\"", "\"").replace("\\\\", "\\").replace("\\/", "/");
            return Some(unescaped);
        }
    }
    None
}

// Helper function to check pagination in API response
fn check_api_pagination(json: &str) -> bool {
    // Look for pagination info like "current_page", "last_page", etc.
    if let (Some(current), Some(last)) = (extract_json_number(json, "current_page"), extract_json_number(json, "last_page")) {
        return current < last;
    }
    
    // Fallback: check if there are exactly 24 mangas (typical page size)
    if json.matches("\"id\":").count() >= 24 {
        return true;
    }
    
    false
}

// Helper function to extract number values from JSON
fn extract_json_number(json: &str, key: &str) -> Option<i32> {
    let pattern = &format!("\"{}\":", key);
    if let Some(start) = json.find(pattern) {
        let value_start = start + pattern.len();
        let remaining = &json[value_start..];
        
        // Skip whitespace
        let trimmed = remaining.trim_start();
        
        // Find where the number ends
        let mut end = 0;
        for (i, ch) in trimmed.chars().enumerate() {
            if ch.is_ascii_digit() || ch == '-' {
                end = i + 1;
            } else {
                break;
            }
        }
        
        if end > 0 {
            if let Ok(num) = trimmed[..end].parse::<i32>() {
                return Some(num);
            }
        }
    }
    None
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