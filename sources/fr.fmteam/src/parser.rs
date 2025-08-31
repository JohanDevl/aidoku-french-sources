use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter,
    ContentRating, Viewer, UpdateStrategy,
    alloc::{String, Vec, vec, format, string::ToString},
    imports::html::Document,
};
use core::cmp::Ordering;
use serde_json::Value;

extern crate alloc;

// JSON parsing functions for the API
pub fn parse_manga_list_json(response: String) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    // Toujours créer au moins un manga de test pour vérifier que l'affichage fonctionne
    mangas.push(Manga {
        key: "test-manga".to_string(),
        cover: Some("https://fmteam.fr/storage/comics/covers/6d34ae3eaccdb37ccc4aeeec89e3b9fd.jpg".to_string()),
        title: "Test Manga - FMTeam".to_string(),
        authors: Some(vec!["Test Author".to_string()]),
        artists: None,
        description: Some("Manga de test pour vérifier l'affichage".to_string()),
        tags: Some(vec!["Action".to_string(), "Test".to_string()]),
        status: MangaStatus::Ongoing,
        content_rating: ContentRating::Safe,
        viewer: Viewer::LeftToRight,
        chapters: None,
        url: Some(format!("{}/comics/test-manga", super::BASE_URL)),
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    });
    
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        // L'API retourne {"comics": [...]} donc accéder à la clé comics
        if let Some(comics_array) = json.get("comics").and_then(|v| v.as_array()) {
            for comic in comics_array.iter().take(10) { // Limiter à 10 pour les tests
                if let Ok(manga) = parse_single_manga_json(comic) {
                    mangas.push(manga);
                } else {
                    // Si le parsing échoue, créer un manga simple pour tester
                    if let Some(title) = comic.get("title").and_then(|v| v.as_str()) {
                        let simple_manga = Manga {
                            key: comic.get("slug")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&title.to_lowercase().replace(" ", "-"))
                                .to_string(),
                            cover: comic.get("thumbnail").and_then(|v| v.as_str()).map(|s| {
                                if s.starts_with("http") {
                                    s.to_string()
                                } else {
                                    format!("{}/{}", super::BASE_URL, s.trim_start_matches('/'))
                                }
                            }),
                            title: title.to_string(),
                            authors: comic.get("author").and_then(|v| v.as_str()).map(|s| vec![s.to_string()]),
                            artists: None,
                            description: comic.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            tags: None,
                            status: MangaStatus::Unknown,
                            content_rating: ContentRating::Safe,
                            viewer: Viewer::LeftToRight,
                            chapters: None,
                            url: Some(format!("{}/comics/{}", super::BASE_URL, 
                                comic.get("slug").and_then(|v| v.as_str()).unwrap_or("unknown"))),
                            next_update_time: None,
                            update_strategy: UpdateStrategy::Always,
                        };
                        mangas.push(simple_manga);
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

pub fn parse_manga_listing_json(response: String, listing_type: &str) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    // Toujours créer un manga de test pour vérifier que l'affichage fonctionne
    mangas.push(Manga {
        key: format!("test-listing-{}", listing_type),
        cover: Some("https://fmteam.fr/storage/comics/covers/6d34ae3eaccdb37ccc4aeeec89e3b9fd.jpg".to_string()),
        title: format!("Test {} - FMTeam", listing_type),
        authors: Some(vec!["Test Author".to_string()]),
        artists: None,
        description: Some(format!("Manga de test pour listing {}", listing_type)),
        tags: Some(vec!["Test".to_string()]),
        status: MangaStatus::Ongoing,
        content_rating: ContentRating::Safe,
        viewer: Viewer::LeftToRight,
        chapters: None,
        url: Some(format!("{}/comics/test-{}", super::BASE_URL, listing_type)),
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    });
    
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        // L'API retourne {"comics": [...]} donc accéder à la clé comics
        if let Some(comics_array) = json.get("comics").and_then(|v| v.as_array()) {
            let mut comic_objects: Vec<&Value> = comics_array.iter().take(10).collect();
            
            // Filter and sort based on listing type
            match listing_type {
                "dernières-sorties" => {
                    // Filter comics with recent chapters
                    comic_objects.retain(|comic| {
                        comic.get("last_chapter").is_some()
                    });
                },
                "populaire" => {
                    // Sort by views or rating for popularity
                    comic_objects.sort_by(|a, b| {
                        let a_views = a.get("views").and_then(|v| v.as_u64()).unwrap_or(0);
                        let b_views = b.get("views").and_then(|v| v.as_u64()).unwrap_or(0);
                        b_views.cmp(&a_views)
                    });
                },
                _ => {
                    // Default - show all
                }
            }
            
            for comic in comic_objects {
                if let Ok(manga) = parse_single_manga_json(comic) {
                    mangas.push(manga);
                } else {
                    // Fallback simple parsing
                    if let Some(title) = comic.get("title").and_then(|v| v.as_str()) {
                        let simple_manga = Manga {
                            key: comic.get("slug")
                                .and_then(|v| v.as_str())
                                .unwrap_or(&title.to_lowercase().replace(" ", "-"))
                                .to_string(),
                            cover: comic.get("thumbnail").and_then(|v| v.as_str()).map(|s| {
                                if s.starts_with("http") {
                                    s.to_string()
                                } else {
                                    format!("{}/{}", super::BASE_URL, s.trim_start_matches('/'))
                                }
                            }),
                            title: title.to_string(),
                            authors: comic.get("author").and_then(|v| v.as_str()).map(|s| vec![s.to_string()]),
                            artists: None,
                            description: comic.get("description").and_then(|v| v.as_str()).map(|s| s.to_string()),
                            tags: None,
                            status: MangaStatus::Unknown,
                            content_rating: ContentRating::Safe,
                            viewer: Viewer::LeftToRight,
                            chapters: None,
                            url: Some(format!("{}/comics/{}", super::BASE_URL, 
                                comic.get("slug").and_then(|v| v.as_str()).unwrap_or("unknown"))),
                            next_update_time: None,
                            update_strategy: UpdateStrategy::Always,
                        };
                        mangas.push(simple_manga);
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

pub fn parse_manga_details_json(mut manga: Manga, response: String) -> Result<Manga> {
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        if let Some(comic) = json.get("comic") {
            manga = update_manga_from_json(manga, comic)?;
        } else {
            // If no nested "comic", try direct access
            manga = update_manga_from_json(manga, &json)?;
        }
    }
    Ok(manga)
}

pub fn parse_chapter_list_json(manga_key: &str, response: String) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        let comic = if let Some(comic) = json.get("comic") {
            comic
        } else {
            &json
        };
        
        if let Some(chapters_array) = comic.get("chapters").and_then(|c| c.as_array()) {
            for chapter in chapters_array {
                if let Ok(ch) = parse_single_chapter_json(manga_key, chapter) {
                    chapters.push(ch);
                }
            }
        }
    }
    
    // Sort chapters by number (descending - newest first)
    chapters.sort_by(|a, b| {
        let a_num = a.chapter_number.unwrap_or(0.0);
        let b_num = b.chapter_number.unwrap_or(0.0);
        b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal)
    });
    
    Ok(chapters)
}

pub fn parse_page_list_json(response: String) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        if let Some(pages_array) = json.as_array() {
            for page in pages_array {
                if let Some(url_str) = page.as_str() {
                    let full_url = super::helper::make_absolute_url(super::BASE_URL, url_str);
                    pages.push(Page {
                        content: PageContent::Url(full_url, None),
                        thumbnail: None,
                        has_description: false,
                        description: None,
                    });
                }
            }
        }
    }
    
    Ok(pages)
}

fn parse_single_manga_json(comic: &Value) -> Result<Manga> {
    let title = comic.get("title")
        .and_then(|v| v.as_str())
        .unwrap_or("Unknown")
        .to_string();
    
    let key = comic.get("slug")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            // Create slug from title if not available
            title.to_lowercase().replace(" ", "-").replace("'", "")
        });
    
    let cover = comic.get("thumbnail")
        .and_then(|v| v.as_str())
        .map(|url| {
            if url.starts_with("http") {
                url.to_string()
            } else {
                super::helper::make_absolute_url(super::BASE_URL, url)
            }
        });
    
    let status = comic.get("status")
        .and_then(|v| v.as_str())
        .map(|s| {
            let s_lower = s.to_lowercase();
            if s_lower.contains("ongoing") || s_lower.contains("en cours") {
                MangaStatus::Ongoing
            } else if s_lower.contains("completed") || s_lower.contains("terminé") || s_lower.contains("complet") {
                MangaStatus::Completed
            } else if s_lower.contains("cancelled") || s_lower.contains("annulé") {
                MangaStatus::Cancelled
            } else if s_lower.contains("hiatus") || s_lower.contains("pause") {
                MangaStatus::Hiatus
            } else {
                MangaStatus::Unknown
            }
        })
        .unwrap_or(MangaStatus::Unknown);
    
    let description = comic.get("description")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string());
    
    let authors = comic.get("author")
        .and_then(|v| v.as_str())
        .filter(|s| !s.is_empty())
        .map(|s| vec![s.to_string()]);
    
    let mut tags: Vec<String> = Vec::new();
    if let Some(genres_array) = comic.get("genres").and_then(|v| v.as_array()) {
        for genre in genres_array {
            if let Some(name) = genre.get("name").and_then(|n| n.as_str()) {
                tags.push(name.to_string());
            }
        }
    }
    
    Ok(Manga {
        key: key.clone(),
        cover,
        title,
        authors,
        artists: None,
        description,
        tags: if tags.is_empty() { None } else { Some(tags) },
        status,
        content_rating: ContentRating::Safe,
        viewer: Viewer::LeftToRight,
        chapters: None,
        url: Some(super::helper::make_absolute_url(super::BASE_URL, &format!("/comics/{}", key))),
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    })
}

fn update_manga_from_json(mut manga: Manga, comic: &Value) -> Result<Manga> {
    if let Some(title) = comic.get("title").and_then(|v| v.as_str()) {
        manga.title = title.to_string();
    }
    
    if let Some(description) = comic.get("description").and_then(|v| v.as_str()) {
        if !description.is_empty() {
            manga.description = Some(description.to_string());
        }
    }
    
    if let Some(author) = comic.get("author").and_then(|v| v.as_str()) {
        manga.authors = Some(vec![author.to_string()]);
    }
    
    if let Some(cover) = comic.get("thumbnail").and_then(|v| v.as_str()) {
        manga.cover = Some(if cover.starts_with("http") {
            cover.to_string()
        } else {
            super::helper::make_absolute_url(super::BASE_URL, cover)
        });
    }
    
    Ok(manga)
}

fn parse_single_chapter_json(manga_key: &str, chapter: &Value) -> Result<Chapter> {
    let chapter_num = chapter.get("chapter")
        .and_then(|v| v.as_f64())
        .or_else(|| chapter.get("number").and_then(|v| v.as_f64()))
        .unwrap_or(1.0) as f32;
    
    let title = chapter.get("title")
        .and_then(|v| v.as_str())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("Chapitre {}", chapter_num));
    
    let key = format!("/read/{}/fr/ch/{}", manga_key, chapter_num as i32);
    
    let scanlators = if let Some(teams_array) = chapter.get("teams").and_then(|v| v.as_array()) {
        let mut team_names: Vec<String> = Vec::new();
        for team in teams_array {
            if let Some(name) = team.get("name").and_then(|n| n.as_str()) {
                team_names.push(name.to_string());
            }
        }
        if team_names.is_empty() {
            Some(vec![String::from("FMTeam")])
        } else {
            Some(team_names)
        }
    } else {
        Some(vec![String::from("FMTeam")])
    };
    
    Ok(Chapter {
        key: key.clone(),
        title: Some(title),
        chapter_number: Some(chapter_num),
        volume_number: None,
        date_uploaded: None,
        scanlators,
        language: Some(String::from("fr")),
        locked: false,
        thumbnail: None,
        url: Some(super::helper::make_absolute_url(super::BASE_URL, &key)),
    })
}

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();

    // FMTeam specific selectors - try multiple approaches
    let manga_selectors = [
        "a[href*=\"/comics/\"]",
        ".manga-item a",
        ".comic-item a", 
        ".manga-card a",
        ".grid-item a",
        ".card a[href*=\"comics\"]",
    ];

    for selector in manga_selectors {
        if let Some(manga_links) = html.select(selector) {
            for item in manga_links {
                // Get title from various possible locations
                let title = if let Some(title_elem) = item.select("h2, h3, .title, .name, .manga-title").and_then(|elems| elems.first()) {
                    title_elem.text().unwrap_or_default()
                } else if let Some(img) = item.select("img").and_then(|imgs| imgs.first()) {
                    img.attr("alt").unwrap_or_default()
                } else {
                    item.text().unwrap_or_default()
                };

                if title.is_empty() {
                    continue;
                }

                // Get URL and validate
                let url = item.attr("href").unwrap_or_default();
                if url.is_empty() || url.contains("?") {
                    continue;
                }

                // Extract manga key from URL
                let key = if url.starts_with("http") {
                    super::helper::extract_id_from_url(&url)
                } else {
                    // Handle relative URLs like "/comics/title"
                    let parts: Vec<&str> = url.split('/').collect();
                    if parts.len() >= 3 && (parts[1] == "comics" || parts[1] == "manga") {
                        String::from(parts[2].trim())
                    } else {
                        continue;
                    }
                };

                if key.is_empty() {
                    continue;
                }

                // Extract cover image
                let cover = if let Some(img_elements) = item.select("img") {
                    if let Some(img) = img_elements.first() {
                        let img_src = img.attr("src")
                            .or_else(|| img.attr("data-src"))
                            .or_else(|| img.attr("data-lazy-src"))
                            .or_else(|| img.attr("data-original"))
                            .unwrap_or_default();
                        if img_src.is_empty() {
                            None
                        } else {
                            Some(super::helper::make_absolute_url(super::BASE_URL, &img_src))
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
                    url: Some(super::helper::make_absolute_url(super::BASE_URL, &url)),
                    next_update_time: None,
                    update_strategy: UpdateStrategy::Always,
                });
            }
            
            // If we found mangas with this selector, stop trying others
            if !mangas.is_empty() {
                break;
            }
        }
    }

    // Fallback: try to find any links with manga-like patterns
    if mangas.is_empty() {
        if let Some(all_links) = html.select("a") {
            for link in all_links {
                let href = link.attr("href").unwrap_or_default();
                if href.contains("comics") || href.contains("manga") {
                    let title = link.text().unwrap_or_default();
                    if !title.is_empty() && title.len() > 2 {
                        let key = super::helper::extract_id_from_url(&href);
                        if !key.is_empty() {
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
                                url: Some(super::helper::make_absolute_url(super::BASE_URL, &href)),
                                next_update_time: None,
                                update_strategy: UpdateStrategy::Always,
                            });
                        }
                    }
                }
            }
        }
    }

    // Check for pagination
    let has_more = check_pagination(&html) || mangas.len() >= 10;

    Ok(MangaPageResult {
        entries: mangas,
        has_next_page: has_more,
    })
}

pub fn parse_manga_details(mut manga: Manga, html: &Document) -> Result<Manga> {
    // Extract cover with multiple selectors
    let cover_selectors = [
        ".manga-cover img",
        ".comic-cover img",
        ".cover img",
        ".thumbnail img",
        "main img",
        ".hero img",
        "img[src*=\"cover\"]",
        "img[src*=\"thumb\"]",
    ];
    
    for selector in cover_selectors {
        if let Some(img_elements) = html.select(selector) {
            if let Some(img) = img_elements.first() {
                let img_src = img.attr("src")
                    .or_else(|| img.attr("data-src"))
                    .or_else(|| img.attr("data-original"))
                    .unwrap_or_default();
                if !img_src.is_empty() {
                    manga.cover = Some(super::helper::make_absolute_url(super::BASE_URL, &img_src));
                    break;
                }
            }
        }
    }
    
    // Extract title
    let title_selectors = [
        "h1.title",
        "h1.manga-title", 
        "h1.comic-title",
        ".title h1",
        ".manga-info h1",
        ".comic-info h1",
        "h1",
        ".hero-title",
    ];
    
    for selector in title_selectors {
        if let Some(elements) = html.select(selector) {
            if let Some(elem) = elements.first() {
                let title_text = elem.text().unwrap_or_default();
                if !title_text.is_empty() && !title_text.to_lowercase().contains("fmteam") {
                    manga.title = title_text;
                    break;
                }
            }
        }
    }
    
    // Extract author and artist
    let author_selectors = [
        "*:contains(Auteur) + *",
        "*:contains(Author) + *", 
        "*:contains(Créateur) + *",
        ".author",
        ".creator",
        ".manga-author",
        ".comic-author",
    ];
    
    for selector in author_selectors {
        if let Some(author_elements) = html.select(selector) {
            if let Some(author_elem) = author_elements.first() {
                let author_text = author_elem.text().unwrap_or_default();
                if !author_text.is_empty() && !author_text.to_lowercase().contains("auteur") {
                    manga.authors = Some(vec![author_text]);
                    break;
                }
            }
        }
    }
    
    // Extract description
    let description_selectors = [
        ".synopsis",
        ".summary", 
        ".description",
        ".manga-synopsis",
        ".comic-synopsis",
        ".content p",
        ".info .description",
        "p:contains(Synopsis)",
    ];
    
    for selector in description_selectors {
        if let Some(desc_elements) = html.select(selector) {
            if let Some(desc_elem) = desc_elements.first() {
                let desc_text = desc_elem.text().unwrap_or_default();
                if !desc_text.is_empty() && desc_text.len() > 10 {
                    manga.description = Some(desc_text);
                    break;
                }
            }
        }
    }
    
    // Extract tags/genres
    let mut tags: Vec<String> = Vec::new();
    let tag_selectors = [
        ".genres a",
        ".tags a", 
        ".genre-list a",
        ".tag-list a",
        "a[href*=\"genre\"]",
        "a[href*=\"tag\"]",
        ".categories a",
    ];
    
    for selector in tag_selectors {
        if let Some(tag_elements) = html.select(selector) {
            for tag_elem in tag_elements {
                let tag_text = tag_elem.text().unwrap_or_default();
                let clean_tag = tag_text.trim();
                if !clean_tag.is_empty() && !tags.contains(&String::from(clean_tag)) {
                    tags.push(String::from(clean_tag));
                }
            }
            if !tags.is_empty() {
                break;
            }
        }
    }
    
    if !tags.is_empty() {
        manga.tags = Some(tags);
    }
    
    // Extract manga status
    let status_selectors = [
        "*:contains(Statut) + *",
        "*:contains(Status) + *",
        ".status",
        ".manga-status", 
        ".comic-status",
    ];
    
    for selector in status_selectors {
        if let Some(status_elements) = html.select(selector) {
            if let Some(status_elem) = status_elements.first() {
                let status_text = status_elem.text().unwrap_or_default();
                let clean_status = status_text.trim().to_lowercase();
                if !clean_status.is_empty() {
                    manga.status = match clean_status.as_str() {
                        "en cours" | "ongoing" | "publication" | "publiant" => MangaStatus::Ongoing,
                        "terminé" | "completed" | "fini" | "achevé" | "complet" => MangaStatus::Completed,
                        "annulé" | "cancelled" | "canceled" | "arrêté" => MangaStatus::Cancelled,
                        "en pause" | "hiatus" | "pause" => MangaStatus::Hiatus,
                        _ => MangaStatus::Unknown,
                    };
                    break;
                }
            }
        }
    }
    
    Ok(manga)
}

pub fn parse_chapter_list(manga_key: &str, html: &Document) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();

    // FMTeam specific chapter selectors
    let chapter_selectors = [
        &format!("a[href*=\"/read/{}/\"]", manga_key),
        &format!("a[href*=\"/{}/ch/\"]", manga_key),
        "a[href*=\"/read/\"]",
        ".chapter-list a",
        ".chapters a",
        ".episode-list a", 
        ".chapter-item a",
    ];

    for selector in chapter_selectors {
        if let Some(chapter_links) = html.select(selector) {
            for link in chapter_links {
                let href = link.attr("href").unwrap_or_default();
                let link_text = link.text().unwrap_or_default();
                
                // Extract chapter number from URL or text
                let chapter_number: f32 = extract_chapter_number(&href, &link_text);
                
                if chapter_number > 0.0 {
                    // Create clean chapter key
                    let chapter_key = if href.starts_with("http") {
                        href.replace(super::BASE_URL, "")
                    } else {
                        href
                    };
                    
                    // Create chapter title
                    let chapter_title = if !link_text.is_empty() {
                        if link_text.to_lowercase().contains("chapitre") || link_text.to_lowercase().contains("chapter") {
                            link_text
                        } else {
                            format!("Chapitre {}", chapter_number)
                        }
                    } else {
                        format!("Chapitre {}", chapter_number)
                    };

                    chapters.push(Chapter {
                        key: chapter_key.clone(),
                        title: Some(chapter_title),
                        chapter_number: Some(chapter_number),
                        volume_number: None,
                        date_uploaded: None,
                        scanlators: None,
                        language: Some(String::from("fr")),
                        locked: false,
                        thumbnail: None,
                        url: Some(super::helper::make_absolute_url(super::BASE_URL, &chapter_key)),
                    });
                }
            }
            
            if !chapters.is_empty() {
                break;
            }
        }
    }

    // Sort chapters by number (descending - newest first)
    chapters.sort_by(|a, b| {
        let a_num = a.chapter_number.unwrap_or(0.0);
        let b_num = b.chapter_number.unwrap_or(0.0);
        b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal)
    });
    
    Ok(chapters)
}

pub fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();

    // FMTeam specific image selectors
    let image_selectors = [
        ".reader img",
        ".comic-reader img",
        ".page-image img",
        "#reader img", 
        ".viewer img",
        "img[src*=\"pages/\"]",
        "img[src*=\"chapters/\"]",
        "img[data-src*=\"pages/\"]",
        "img[data-src*=\"chapters/\"]",
    ];

    for selector in image_selectors {
        if let Some(images) = html.select(selector) {
            for img in images {
                let img_src = img.attr("data-src")
                    .or_else(|| img.attr("src"))
                    .or_else(|| img.attr("data-original"))
                    .or_else(|| img.attr("data-lazy-src"))
                    .unwrap_or_default();

                if !img_src.is_empty() && 
                   (img_src.contains("pages") || img_src.contains("chapters") || img_src.ends_with(".jpg") || img_src.ends_with(".png") || img_src.ends_with(".webp")) {
                    let full_url = super::helper::make_absolute_url(super::BASE_URL, &img_src);
                    pages.push(Page {
                        content: PageContent::Url(full_url, None),
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

    // Fallback: try to get all images if specific selectors didn't work
    if pages.is_empty() {
        if let Some(all_images) = html.select("img") {
            for img in all_images {
                let img_src = img.attr("data-src")
                    .or_else(|| img.attr("src"))
                    .unwrap_or_default();

                if !img_src.is_empty() && 
                   (img_src.contains("http") || img_src.starts_with("/")) &&
                   (img_src.ends_with(".jpg") || img_src.ends_with(".png") || img_src.ends_with(".webp") || img_src.ends_with(".jpeg")) {
                    let full_url = super::helper::make_absolute_url(super::BASE_URL, &img_src);
                    pages.push(Page {
                        content: PageContent::Url(full_url, None),
                        thumbnail: None,
                        has_description: false,
                        description: None,
                    });
                }
            }
        }
    }

    Ok(pages)
}

// Helper function to check pagination
fn check_pagination(html: &Document) -> bool {
    // Method 1: Look for pagination text patterns
    if let Some(pagination_elements) = html.select("div, span, p, .pagination, .page-numbers") {
        for elem in pagination_elements {
            if let Some(text) = elem.text() {
                if text.contains("Page ") && text.contains(" of ") {
                    return true;
                }
                if text.contains("Suivant") || text.contains("Next") || text.contains(">>") {
                    return true;
                }
                if text.contains("…") || text.contains("...") {
                    return true;
                }
            }
        }
    }
    
    // Method 2: Look for pagination links
    if let Some(links) = html.select("a") {
        for link in links {
            let href = link.attr("href").unwrap_or_default();
            let text = link.text().unwrap_or_default();
            if href.contains("page=") || text.contains("Suivant") || text.contains("Next") {
                return true;
            }
        }
    }
    
    false
}

// Helper function to extract chapter number from URL or text
fn extract_chapter_number(url: &str, text: &str) -> f32 {
    // Try URL first - look for patterns like /ch/123 or /chapter/123
    if let Some(ch_pos) = url.find("/ch/") {
        let after_ch = &url[ch_pos + 4..];
        if let Some(end_pos) = after_ch.find('/') {
            let number_part = &after_ch[..end_pos];
            if let Ok(num) = number_part.parse::<f32>() {
                return num;
            }
        } else {
            // No trailing slash, take the rest
            if let Ok(num) = after_ch.parse::<f32>() {
                return num;
            }
        }
    }
    
    // Try URL parts - last segment might be chapter number
    if let Some(last_part) = url.split('/').last() {
        if let Ok(num) = last_part.parse::<f32>() {
            return num;
        }
    }
    
    // Try text extraction
    if text.to_lowercase().contains("chapitre") {
        if let Some(ch_pos) = text.to_lowercase().find("chapitre") {
            let after_ch = &text[ch_pos + 8..];
            let number_str: String = after_ch.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(num) = number_str.parse::<f32>() {
                return num;
            }
        }
    }
    
    if text.to_lowercase().contains("chapter") {
        if let Some(ch_pos) = text.to_lowercase().find("chapter") {
            let after_ch = &text[ch_pos + 7..];
            let number_str: String = after_ch.chars()
                .take_while(|c| c.is_ascii_digit() || *c == '.')
                .collect();
            if let Ok(num) = number_str.parse::<f32>() {
                return num;
            }
        }
    }
    
    0.0
}