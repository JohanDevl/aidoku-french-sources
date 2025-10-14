#![no_std]

use aidoku::{
    Chapter, ContentRating, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, MangaStatus,
    Page, PageContent, PageContext, Result, Source, UpdateStrategy, Viewer,
    alloc::{String, Vec},
    imports::{net::Request, html::Document, std::send_partial_result},
    prelude::*,
};

extern crate alloc;
use alloc::{string::ToString, vec};

pub static BASE_URL: &str = "https://www.lelmanga.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.2 Mobile/15E148 Safari/604.1";

// Calculate viewer type based on tags (Manhwa/Webtoon vs Manga)
fn calculate_viewer(tags: &[String]) -> Viewer {
	if tags.iter().any(|tag| {
		let lower = tag.to_lowercase();
		matches!(lower.as_str(), "manhwa" | "manhua" | "webtoon")
	}) {
		Viewer::Vertical
	} else {
		Viewer::RightToLeft
	}
}

pub struct LelManga;

impl Source for LelManga {
    fn new() -> Self {
        Self
    }

    fn get_search_manga_list(
        &self,
        query: Option<String>,
        page: i32,
        filters: Vec<FilterValue>,
    ) -> Result<MangaPageResult> {

        let mut selected_genres: Vec<String> = Vec::new();
        let mut selected_status = String::new();
        let mut selected_type = String::new();
        let mut selected_order = String::new();
        
        // Process filters
        for filter in &filters {
            match filter {
                FilterValue::Select { id, value } => {
                    if id == "genre" && !value.is_empty() && value != "Tout" {
                        selected_genres.push(value.clone());
                    } else if id == "status" && !value.is_empty() && value != "Tout" {
                        // Map French status values to English for server
                        selected_status = match value.as_str() {
                            "En cours" => "ongoing".to_string(),
                            "Terminé" => "completed".to_string(),
                            _ => value.clone(),
                        };
                    } else if id == "type" && !value.is_empty() && value != "Tout" {
                        // Map type filter values to server values
                        selected_type = value.to_lowercase();
                    } else if id == "order" && !value.is_empty() && value != "Default" {
                        // Map order filter values to server values
                        selected_order = value.clone();
                    }
                }
                FilterValue::Text { id, value } => {
                    if id == "genre" && !value.is_empty() && value != "Tout" {
                        selected_genres.push(value.clone());
                    } else if id == "status" && !value.is_empty() && value != "Tout" {
                        selected_status = value.clone();
                    } else if id == "type" && !value.is_empty() && value != "Tout" {
                        selected_type = value.to_lowercase();
                    } else if id == "order" && !value.is_empty() && value != "Default" {
                        selected_order = value.clone();
                    }
                }
                FilterValue::MultiSelect { id, included, excluded } => {
                    if id == "genre" {
                        // Add included genres (positive IDs)
                        for value in included {
                            if !value.is_empty() && value != "Tout" {
                                selected_genres.push(value.clone());
                            }
                        }
                        // Add excluded genres with "-" prefix (negative IDs)
                        for value in excluded {
                            if !value.is_empty() && value != "Tout" {
                                selected_genres.push(format!("-{}", value));
                            }
                        }
                    }
                }
                _ => {}
            }
        }

        // Build URL parameters
        let mut url_params = Vec::new();
        
        // Add page parameter first if not first page
        if page > 1 {
            url_params.push(format!("page={}", page));
        }
        
        // Add genre parameters - new format: genre[]=ID (URL encoded)
        for genre_id in &selected_genres {
            if !genre_id.is_empty() && genre_id != "Tout" {
                url_params.push(format!("genre%5B%5D={}", genre_id));
            }
        }
        
        // Always add status parameter (empty if not set)
        let status_value = if selected_status.is_empty() { "" } else { &selected_status };
        url_params.push(format!("status={}", Self::urlencode(status_value)));
        
        // Always add type parameter (empty if not set)
        let type_value = if selected_type.is_empty() { "" } else { &selected_type };
        url_params.push(format!("type={}", Self::urlencode(type_value)));
        
        // Always add order parameter (use selected value or empty)
        let order_value = if selected_order.is_empty() { "" } else { &selected_order };
        url_params.push(format!("order={}", Self::urlencode(order_value)));
        
        let url = if let Some(ref search_query) = query {
            // Search mode
            if search_query.is_empty() {
                format!("{}/manga/?{}", BASE_URL, url_params.join("&"))
            } else {
                let mut search_params = vec![format!("s={}", Self::urlencode(&search_query))];
                search_params.extend(url_params);
                format!("{}/?{}", BASE_URL, search_params.join("&"))
            }
        } else {
            // Browse/filter mode using /manga endpoint
            format!("{}/manga/?{}", BASE_URL, url_params.join("&"))
        };
        
        self.get_manga_from_page(&url)
    }

    fn get_manga_update(&self, manga: Manga, _needs_details: bool, needs_chapters: bool) -> Result<Manga> {
        println!("[lelmanga] get_manga_update START - manga_id: {}, needs_details: {}, needs_chapters: {}",
            manga.key, _needs_details, needs_chapters);

        let url = format!("{}/manga/{}/", BASE_URL, manga.key);

        let html = Request::get(&url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        // Parse manga details
        let mut updated_manga = self.parse_manga_details(manga.key.clone(), &html)?;

        if _needs_details {
            println!("[lelmanga] Metadata fetched successfully - title: {}", updated_manga.title);
            send_partial_result(&updated_manga);
        }

        // Parse chapters if needed
        if needs_chapters {
            let chapters = self.parse_chapter_list(manga.key, &html)?;
            let chapter_count = chapters.len();
            updated_manga.chapters = Some(chapters);
            println!("[lelmanga] Chapters fetched successfully - count: {}", chapter_count);
        }

        println!("[lelmanga] get_manga_update COMPLETE");
        Ok(updated_manga)
    }

    fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {

        let url = format!("{}/{}/", BASE_URL, chapter.key);

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

        let mut url = format!("{}/manga/", BASE_URL);

        // Add sorting parameter based on listing type
        match listing.name.as_str() {
            "Populaire" => url.push_str("?order=popular"),
            "Tendance" => url.push_str("?order=update"),
            _ => url.push_str("?order=latest"),
        }

        // Add page parameter
        if page > 1 {
            url.push_str(&format!("&page={}", page));
        }

        self.get_manga_from_page(&url)
    }
}

impl ImageRequestProvider for LelManga {
    fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {

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
    fn urlencode(s: &str) -> String {
        let mut result = String::new();
        for c in s.chars() {
            match c {
                'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                    result.push(c);
                }
                ' ' => result.push('+'),
                _ => {
                    for byte in c.to_string().bytes() {
                        result.push_str(&format!("%{:02X}", byte));
                    }
                }
            }
        }
        result
    }

    fn is_valid_url(url: &str) -> bool {
        url.starts_with("http://") || url.starts_with("https://")
    }

    fn get_manga_from_page(&self, url: &str) -> Result<MangaPageResult> {
        let html = Request::get(url)?
            .header("User-Agent", USER_AGENT)
            .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
            .header("Referer", BASE_URL)
            .html()?;

        let mut entries: Vec<Manga> = Vec::new();

        // Use MangaThemesia selectors with debugging
        let selectors = [
            ".utao .uta .imgu", 
            ".listupd .bs .bsx", 
            ".page-listing-item",
            ".manga-item",
            ".manga-list .manga-item",
            ".post"
        ];
        
        let mut found_items = false;
        let mut items_vec = Vec::new();
        
        for selector in &selectors {
            if let Some(items) = html.select(selector) {
                items_vec = items.collect();
                if !items_vec.is_empty() {
                    found_items = true;
                    break;
                }
            }
        }
        
        if !found_items {
            // No items found - page might be empty or have different structure
        }

        if found_items {
            for item in items_vec.iter() {
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
                if href.is_empty() || !Self::is_valid_url(&href) {
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

                // NO genres in listing HTML (confirmed by logs) - only available on individual manga pages
                // Listing items only contain: "Title Chapitre X Rating" 
                let tags: Vec<String> = Vec::new();

                entries.push(Manga {
                    key,
                    title,
                    cover: if cover.is_empty() { None } else { Some(cover) },
                    authors: None,
                    artists: None,
                    description: None,
                    url: Some(href),
                    tags: if tags.is_empty() { None } else { Some(tags) },
                    status: MangaStatus::Unknown,
                    content_rating: ContentRating::Safe,
                    viewer: Viewer::RightToLeft,
                    chapters: None,
                    next_update_time: None,
                    update_strategy: UpdateStrategy::Never,
                });
            }
        }

        // Check for pagination - WebFetch confirmed "Suivante" link exists
        let has_next_page = html.select("a:contains(Suivante), a:contains(Next), .pagination .next, .hpage .r, .wp-pagenavi .next, .nav-links .next").is_some() 
            || entries.len() >= 20; // Fallback: if we have 20+ entries, assume there might be more pages

        Ok(MangaPageResult {
            entries,
            has_next_page,
        })
    }

    fn parse_manga_details(&self, key: String, html: &Document) -> Result<Manga> {

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


        if title.is_empty() {
            return Err(aidoku::AidokuError::message("Failed to parse manga title"));
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
            String::new()
        };


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
                desc_text
            } else {
                String::new()
            }
        } else {
            String::new()
        };


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


        // Extract status with multiple selectors
        let status = if let Some(status_elem) = html.select("div.post-content_item:contains(Statut) div.summary-content, .imptdt:contains(Statut) i, .status, .manga-status, .post-status, .series-status, .tsinfo .imptdt:contains(Status) i, .fmed b:contains(Status) + span, .spe span:contains(Status) + span") {
            if let Some(first_status) = status_elem.first() {
                let status_str = first_status.text().unwrap_or_default().trim().to_lowercase();
                
                let parsed_status = match status_str.as_str() {
                    "en cours" | "ongoing" | "en_cours" | "en-cours" | "publikasi" => MangaStatus::Ongoing,
                    "terminé" | "completed" | "termine" | "fini" | "achevé" => MangaStatus::Completed,
                    "annulé" | "cancelled" | "annule" | "canceled" => MangaStatus::Cancelled,
                    "en pause" | "hiatus" | "pause" | "en_pause" | "en-pause" => MangaStatus::Hiatus,
                    _ => {
                        MangaStatus::Unknown
                    }
                };
                
                parsed_status
            } else {
                MangaStatus::Unknown
            }
        } else {
            
            // Try broader selectors
            if let Some(info_elem) = html.select(".tsinfo, .infomanga, .manga-info, .post-content") {
                
                // Look for text containing status keywords
                for elem in info_elem {
                    let text = elem.text().unwrap_or_default().to_lowercase();
                    if text.contains("statut") || text.contains("status") {
                        
                        // Extract status from the text and break early
                        if text.contains("en cours") || text.contains("ongoing") {
                            return Ok(self.create_manga_result(key, title, cover, authors, artists, description, tags, MangaStatus::Ongoing));
                        } else if text.contains("terminé") || text.contains("completed") || text.contains("fini") {
                            return Ok(self.create_manga_result(key, title, cover, authors, artists, description, tags, MangaStatus::Completed));
                        }
                    }
                }
            }
            
            MangaStatus::Unknown
        };


        // Content rating will be calculated in create_manga_result

        Ok(self.create_manga_result(key, title, cover, authors, artists, description, tags, status))
    }

    fn parse_chapter_list(&self, _manga_key: String, html: &Document) -> Result<Vec<Chapter>> {

        let mut chapters: Vec<Chapter> = Vec::new();

        // Use MangaThemesia selectors for chapters with more alternatives
        let selector = "div.bxcl li, div.cl li, #chapterlist li, ul li:has(div.chbox):has(div.eph-num), .chapter-list li, .wp-manga-chapter, .manga-chapters li, li.wp-manga-chapter";

        if let Some(items) = html.select(selector) {
            let items_vec: Vec<_> = items.collect();

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
                if href.is_empty() || !Self::is_valid_url(&href) {
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

                // Extract chapter number from URL or title
                let chapter_number = self.extract_chapter_number(&chapter_key, &clean_title);

                // Parse chapter date with multiple selectors, prioritizing extracted date from title
                let date_uploaded = if let Some(extracted) = extracted_date {
                    Some(extracted)
                } else {
                    if let Some(date_elem) = item.select(".chapterdate, .dt, .chapter-date, .date, span.dt, .chapter-release-date") {
                        if let Some(first_date) = date_elem.first() {
                            let date_str = first_date.text().unwrap_or_default();
                            self.parse_chapter_date(&date_str)
                        } else {
                            None
                        }
                    } else {
                        None
                    }
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

        Ok(chapters)
    }

    fn extract_chapter_number(&self, chapter_id: &str, title: &str) -> f32 {
        let chapter_id_lower = chapter_id.to_lowercase();

        // Look for "chapitre-" or "chapter-" pattern
        if let Some(chapitre_pos) = chapter_id_lower.find("chapitre-") {
            let after_chapitre = &chapter_id[chapitre_pos + 9..];
            if let Some(num_str) = after_chapitre.split('-').next() {
                if let Ok(num) = num_str.parse::<f32>() {
                    return num;
                }
            }
        }

        if let Some(chapter_pos) = chapter_id_lower.find("chapter-") {
            let after_chapter = &chapter_id[chapter_pos + 8..];
            if let Some(num_str) = after_chapter.split('-').next() {
                if let Ok(num) = num_str.parse::<f32>() {
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
                    return second_last_num;
                }
            }
        }

        // Fallback: last segment
        if let Some(num_str) = chapter_id.split('-').last() {
            if let Ok(num) = num_str.parse::<f32>() {
                return num;
            }
        }

        // Extract from title
        let words: Vec<&str> = title.split_whitespace().collect();
        for (i, word) in words.iter().enumerate() {
            if word.to_lowercase().contains("chapitre") || word.to_lowercase().contains("chapter") {
                if i + 1 < words.len() {
                    if let Ok(num) = words[i + 1].parse::<f32>() {
                        return num;
                    }
                }
            }
        }

        // Last resort: any number in title
        for word in words {
            if let Ok(num) = word.parse::<f32>() {
                return num;
            }
        }

        -1.0
    }

    fn parse_chapter_date(&self, date_str: &str) -> Option<i64> {
        if date_str.is_empty() {
            return None;
        }


        // English months with their numbers
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
                    if day >= 1 && day <= 31 && year >= 1970 && year <= 2100 {
                        // More accurate timestamp calculation
                        // Days since epoch (January 1, 1970)
                        let mut days = 0i64;
                        
                        // Add days for complete years
                        for y in 1970..year {
                            if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
                                days += 366; // leap year
                            } else {
                                days += 365;
                            }
                        }
                        
                        // Add days for complete months in the current year
                        let days_in_month = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
                        for m in 1..*month {
                            if m == 2 && ((year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)) {
                                days += 29; // February in leap year
                            } else {
                                days += days_in_month[m as usize - 1] as i64;
                            }
                        }
                        
                        // Add the days in the current month
                        days += (day - 1) as i64;
                        
                        let timestamp = days * 86400; // Convert to seconds
                        return Some(timestamp);
                    }
                }
            }
        }

        None
    }

    fn parse_page_list(&self, html: &Document) -> Result<Vec<Page>> {

        let mut pages: Vec<Page> = Vec::new();

        // First try: HTML images
        if let Some(img_elements) = html.select("div#readerarea img") {
            for img_element in img_elements {
                let img_url = if let Some(lazy_src) = img_element.attr("data-lazy-src") {
                    if !lazy_src.is_empty() {
                        lazy_src.to_string()
                    } else if let Some(data_src) = img_element.attr("data-src") {
                        if !data_src.is_empty() {
                            data_src.to_string()
                        } else {
                            let src = img_element.attr("src").unwrap_or_default();
                            // Avoid SVG placeholders
                            if src.contains("/readerarea.svg") || src.ends_with(".svg") {
                                String::new()
                            } else {
                                src
                            }
                        }
                    } else {
                        let src = img_element.attr("src").unwrap_or_default();
                        // Avoid SVG placeholders
                        if src.contains("/readerarea.svg") || src.ends_with(".svg") {
                            String::new()
                        } else {
                            src
                        }
                    }
                } else if let Some(data_src) = img_element.attr("data-src") {
                    if !data_src.is_empty() {
                        data_src.to_string()
                    } else {
                        let src = img_element.attr("src").unwrap_or_default();
                        // Avoid SVG placeholders
                        if src.contains("/readerarea.svg") || src.ends_with(".svg") {
                            String::new()
                        } else {
                            src
                        }
                    }
                } else {
                    let src = img_element.attr("src").unwrap_or_default();
                    // Avoid SVG placeholders
                    if src.contains("/readerarea.svg") || src.ends_with(".svg") {
                        String::new()
                    } else {
                        src
                    }
                };

                // Additional validation: ensure URL points to a valid image
                if !img_url.is_empty() &&
                   (img_url.ends_with(".jpg") || img_url.ends_with(".jpeg") ||
                    img_url.ends_with(".png") || img_url.ends_with(".webp")) &&
                   !img_url.contains("/readerarea.svg") {
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
            return Ok(pages);
        }

        // Second try: JavaScript parsing
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
            if let Some(images_pattern_start) = html_content.find("\"images\":[") {
                let images_section = &html_content[images_pattern_start + 10..];
                if let Some(images_end) = images_section.find("]") {
                    let images_content = &images_section[..images_end];

                    let parts: Vec<&str> = images_content.split('\"').collect();

                    for part in parts {
                        if part.starts_with("https://") && part.contains("lelmanga.com") && 
                           (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp")) {
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

        if pages.is_empty() {
            return Err(aidoku::AidokuError::message("No pages found in chapter"));
        }

        Ok(pages)
    }

    fn clean_chapter_title_and_extract_date(&self, raw_title: &str) -> (String, Option<i64>) {
        
        let mut clean_title = raw_title.to_string();
        let mut extracted_date = None;
        
        // Look for date patterns like "August 29, 2025", "July 3, 2025", etc.
        let english_months = [
            "January", "February", "March", "April", "May", "June",
            "July", "August", "September", "October", "November", "December"
        ];
        
        // Check for dates anywhere in the title
        let words: Vec<&str> = raw_title.split_whitespace().collect();
        
        for i in 0..words.len().saturating_sub(2) {
            let potential_month = words[i];
            let potential_day = words.get(i + 1);
            let potential_year = words.get(i + 2);
            
            if let (Some(day_str), Some(year_str)) = (potential_day, potential_year) {
                // Check if this looks like a date (Month Day, Year or Month Day Year)
                if english_months.iter().any(|&month| month.eq_ignore_ascii_case(potential_month)) {
                    let day_clean = day_str.trim_end_matches(',');
                    let date_candidate = format!("{} {} {}", potential_month, day_clean, year_str);
                    
                    if let Some(parsed_date) = self.parse_chapter_date(&date_candidate) {
                        extracted_date = Some(parsed_date);
                        
                        // Remove the date part from title - find the original date text and remove it
                        let original_date_text = if day_str.ends_with(',') {
                            format!("{} {} {}", potential_month, day_str, year_str)
                        } else {
                            format!("{} {} {}", potential_month, day_str, year_str)
                        };
                        
                        if let Some(date_pos) = raw_title.find(&original_date_text) {
                            clean_title = format!("{}{}", 
                                &raw_title[..date_pos].trim(),
                                &raw_title[date_pos + original_date_text.len()..].trim()
                            ).trim().to_string();
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
        
        (clean_title, extracted_date)
    }

    fn create_manga_result(
        &self,
        key: String,
        title: String,
        cover: String,
        authors: Option<Vec<String>>,
        artists: Option<Vec<String>>,
        description: String,
        tags: Vec<String>,
        status: MangaStatus,
    ) -> Manga {
        let content_rating = if tags.iter().any(|v| ["ecchi", "mature", "adult"].iter().any(|tag| v.to_lowercase() == *tag)) {
            ContentRating::Suggestive
        } else {
            ContentRating::Safe
        };

        // Calculate viewer based on tags
        let viewer = calculate_viewer(&tags);

        Manga {
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
            viewer,
            chapters: None,
            next_update_time: None,
            update_strategy: UpdateStrategy::Never,
        }
    }
}

register_source!(LelManga, ListingProvider, ImageRequestProvider);