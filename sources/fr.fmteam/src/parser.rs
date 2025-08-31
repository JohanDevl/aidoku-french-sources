use aidoku::{
    Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter,
    ContentRating, Viewer, UpdateStrategy,
    alloc::{String, Vec, vec, format, string::ToString},
};
use core::cmp::Ordering;
use serde_json::Value;

extern crate alloc;

// JSON parsing functions for the API
pub fn parse_manga_list_json(response: String, search_query: Option<String>) -> Result<MangaPageResult> {
    let mut mangas: Vec<Manga> = Vec::new();
    
    if let Ok(json) = serde_json::from_str::<Value>(&response) {
        // L'API retourne {"comics": [...]} donc accéder à la clé comics
        if let Some(comics_array) = json.get("comics").and_then(|v| v.as_array()) {
            for comic in comics_array {
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
    
    // Filter results client-side if search query provided
    if let Some(query) = search_query {
        let query_lower = query.to_lowercase();
        mangas.retain(|manga| {
            // Search in title
            if manga.title.to_lowercase().contains(&query_lower) {
                return true;
            }
            
            // Search in description  
            if let Some(ref description) = manga.description {
                if description.to_lowercase().contains(&query_lower) {
                    return true;
                }
            }
            
            // Search in authors
            if let Some(ref authors) = manga.authors {
                for author in authors {
                    if author.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }
            }
            
            // Search in tags
            if let Some(ref tags) = manga.tags {
                for tag in tags {
                    if tag.to_lowercase().contains(&query_lower) {
                        return true;
                    }
                }
            }
            
            false
        });
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
        // Follow PizzaReader structure: result.chapter.pages
        if let Some(chapter) = json.get("chapter") {
            if let Some(pages_array) = chapter.get("pages").and_then(|p| p.as_array()) {
                for (_i, page) in pages_array.iter().enumerate() {
                    if let Some(url_str) = page.as_str() {
                        // URLs should be complete from API like PizzaReader
                        let full_url = if url_str.starts_with("http") {
                            url_str.to_string()
                        } else {
                            format!("{}{}", super::BASE_URL, url_str)
                        };
                        
                        pages.push(Page {
                            content: PageContent::url(full_url),
                            thumbnail: None,
                            has_description: false,
                            description: None,
                        });
                    }
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
    
    // Try to get the actual URL from API data first, fallback to constructed URL
    let key = if let Some(chapter_url) = chapter.get("url").and_then(|v| v.as_str()) {
        chapter_url.to_string()
    } else {
        format!("/read/{}/fr/ch/{}", manga_key, chapter_num as i32)
    };
    
    
    Ok(Chapter {
        key: key.clone(),
        title: Some(title),
        chapter_number: Some(chapter_num),
        volume_number: None,
        date_uploaded: None,
        scanlators: None,
        language: None,
        locked: false,
        thumbnail: None,
        url: Some(if key.starts_with("http") { key } else { format!("{}{}", super::BASE_URL, key) }),
    })
}



