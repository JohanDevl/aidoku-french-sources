use aidoku::{
	Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, ContentRating, Viewer, Chapter,
	UpdateStrategy,
	alloc::{String, Vec, vec, format},
	imports::html::Document,
};

extern crate alloc;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Simple debug check - try to select basic elements
	let debug_check = html.select("html");
	if debug_check.is_none() {
		// HTML structure issue
		mangas.push(Manga {
			key: String::from("debug-no-html"),
			cover: None,
			title: String::from("DEBUG: No HTML structure found"),
			authors: None,
			artists: None,
			description: None,
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::LeftToRight,
			chapters: None,
			url: Some(String::from("https://lelscanfr.com/manga/debug")),
			next_update_time: None,
			update_strategy: UpdateStrategy::Always,
		});
		return Ok(MangaPageResult {
			entries: mangas,
			has_next_page: false,
		});
	}

	// Primary selector based on actual site structure
	let selectors = [
		"a[href*=\"/manga/\"]"
	];
	
	let mut items = None;
	let mut selector_found = false;
	
	for selector in selectors {
		if let Some(found_items) = html.select(selector) {
			selector_found = true;
			if found_items.first().is_some() {
				items = html.select(selector);
				break;
			}
		}
	}
	
	// Debug: No selectors found - return empty list to identify issue
	if !selector_found {
		return Ok(MangaPageResult {
			entries: vec![],
			has_next_page: false,
		});
	}

	if let Some(items) = items {
		let item_count = items.count();
		
		// Debug: Return a test manga if no real items found but selector worked
		if item_count == 0 {
			mangas.push(Manga {
				key: String::from("test-debug-1"),
				cover: None,
				title: String::from("DEBUG: Selector found but no items"),
				authors: None,
				artists: None,
				description: None,
				tags: None,
				status: MangaStatus::Unknown,
				content_rating: ContentRating::Safe,
				viewer: Viewer::LeftToRight,
				chapters: None,
				url: Some(String::from("https://lelscanfr.com/manga/test-debug")),
				next_update_time: None,
				update_strategy: UpdateStrategy::Always,
			});
		}
		
		if let Some(manga_links) = html.select("a[href*=\"/manga/\"]") {
			for item in manga_links {
			
			// Extract title - since item is already an <a> tag, look for h2 inside
			let title = if let Some(h2_elements) = item.select("h2") {
				if let Some(h2) = h2_elements.first() {
					h2.text().unwrap_or_default()
				} else {
					continue;
				}
			} else {
				// Fallback to the whole link text if no h2 found
				let fallback_title = item.text().unwrap_or_default();
				if fallback_title.is_empty() {
					String::from("DEBUG: Empty title fallback")
				} else {
					fallback_title
				}
			};

			if title.is_empty() {
				continue;
			}
			
			// Extract URL - item is already an <a> tag
			let url = item.attr("href").unwrap_or_default();
			
			if url.is_empty() || !url.contains("/manga/") {
				//println!("LelscanFR: Skipping invalid URL: {}", url);
				continue;
			}

			let key = super::helper::extract_id_from_url(&url);
			if key.is_empty() {
				//println!("LelscanFR: Skipping item with empty key from URL: {}", url);
				continue;
			}
			//println!("LelscanFR: Extracted key: {}", key);
			
			// Extract cover - look for img tag within the link
			let cover = if let Some(img_elements) = item.select("img") {
				if let Some(img) = img_elements.first() {
					img.attr("src")
						.or_else(|| img.attr("data-src"))
						.or_else(|| img.attr("data-lazy-src"))
						.unwrap_or_default()
				} else {
					String::new()
				}
			} else {
				String::new()
			};

			let absolute_cover = if cover.is_empty() {
				None
			} else {
				Some(super::helper::make_absolute_url("https://lelscanfr.com", &cover))
			};

			mangas.push(Manga {
				key: key.clone(),
				cover: absolute_cover,
				title,
				authors: None,
				artists: None,
				description: None,
				tags: None,
				status: MangaStatus::Unknown,
				content_rating: ContentRating::Safe,
				viewer: Viewer::LeftToRight,
				chapters: None,
				url: Some(super::helper::make_absolute_url("https://lelscanfr.com", &url)),
				next_update_time: None,
				update_strategy: UpdateStrategy::Always,
			});
			//println!("LelscanFR: Successfully created manga: {}", title);
			}
		}
	} else {
		// Debug: No items found, return debug info
		mangas.push(Manga {
			key: String::from("debug-no-items"),
			cover: None,
			title: String::from("DEBUG: No items found with manga selector"),
			authors: None,
			artists: None,
			description: None,
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::LeftToRight,
			chapters: None,
			url: Some(String::from("https://lelscanfr.com/manga/debug")),
			next_update_time: None,
			update_strategy: UpdateStrategy::Always,
		});
	}

	// Check pagination - multiple approaches for robustness
	let has_more = if let Some(pagination) = html.select(".pagination") {
		let pagination_count = pagination.count();
		let has_pagination = pagination_count > 0 && mangas.len() >= 20;
		//println!("LelscanFR: Pagination elements found: {}, has_more: {}", pagination_count, has_pagination);
		has_pagination
	} else {
		let has_more = mangas.len() >= 20;
		//println!("LelscanFR: No pagination found, has_more based on count: {}", has_more);
		has_more
	};

	//println!("LelscanFR: Final result - {} mangas, has_next_page: {}", mangas.len(), has_more);

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_details(mut manga: Manga, html: &Document) -> Result<Manga> {
	// Extract cover with multiple selectors
	let cover_selectors = [
		"img[src*=\"storage/covers/\"]",
		"main img",
		".manga-cover img", 
		".cover img",
		"img"
	];
	
	for selector in cover_selectors {
		if let Some(img_elements) = html.select(selector) {
			if let Some(img) = img_elements.first() {
				if let Some(src) = img.attr("src") {
					if !src.is_empty() {
						manga.cover = Some(super::helper::make_absolute_url("https://lelscanfr.com", &src));
						break;
					}
				}
			}
		}
	}
	
	// Extract title with multiple approaches
	if let Some(h1_elements) = html.select("h1") {
		if let Some(h1) = h1_elements.first() {
			if let Some(title_text) = h1.text() {
				if !title_text.is_empty() {
					manga.title = title_text;
				}
			}
		}
	} else if let Some(title_elements) = html.select(".manga-title") {
		if let Some(title_elem) = title_elements.first() {
			if let Some(title_text) = title_elem.text() {
				if !title_text.is_empty() {
					manga.title = title_text;
				}
			}
		}
	}
	
	// Extract author and artist
	let author_selectors = [
		"span:contains(Auteur)+span",
		"span:contains(Author)+span",
		".author-info",
		".manga-author"
	];
	
	for selector in author_selectors {
		if let Some(author_elements) = html.select(selector) {
			if let Some(author_elem) = author_elements.first() {
				if let Some(author_text) = author_elem.text() {
					if !author_text.is_empty() {
						manga.authors = Some(vec![author_text]);
						break;
					}
				}
			}
		}
	}
	
	let artist_selectors = [
		"span:contains(Artiste)+span", 
		"span:contains(Artist)+span",
		".artist-info",
		".manga-artist"
	];
	
	for selector in artist_selectors {
		if let Some(artist_elements) = html.select(selector) {
			if let Some(artist_elem) = artist_elements.first() {
				if let Some(artist_text) = artist_elem.text() {
					if !artist_text.is_empty() {
						manga.artists = Some(vec![artist_text]);
						break;
					}
				}
			}
		}
	}
	
	// Extract description
	let description_selectors = [
		".manga-synopsis",
		"#description+p",
		"main .card p",
		".description", 
		".summary"
	];
	
	for selector in description_selectors {
		if let Some(desc_elements) = html.select(selector) {
			if let Some(desc_elem) = desc_elements.first() {
				if let Some(desc_text) = desc_elem.text() {
					if !desc_text.is_empty() {
						manga.description = Some(desc_text);
						break;
					}
				}
			}
		}
	}
	
	Ok(manga)
}

pub fn parse_chapter_list(manga_key: &str, all_html: Vec<Document>) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for html in all_html {
		// Multiple selectors for chapter lists
		let chapter_selectors = [
			".chapter-list a",
			"#chapters-list a",
			"a[href*=\"/manga/\"]",
			".chapter-item a",
			".chapters a"
		];
		
		let mut chapter_links = None;
		for selector in chapter_selectors {
			if let Some(found_links) = html.select(selector) {
				if found_links.first().is_some() {
					chapter_links = html.select(selector);
					break;
				}
			}
		}

		if let Some(links) = chapter_links {
			for link in links {
				let href = link.attr("href").unwrap_or_default();
				if href.is_empty() || !href.contains(manga_key) {
					continue;
				}
				
				// Extract chapter key (relative path)
				let chapter_key = if href.starts_with("http") {
					href.replace("https://lelscanfr.com", "")
				} else {
					href.clone()
				};
				
				// Extract chapter number from URL
				let parts: Vec<&str> = href.split('/').collect();
				let chapter_number: f32 = if parts.len() >= 6 {
					parts[5].parse().unwrap_or(0.0)
				} else {
					0.0
				};
				
				// Extract chapter title
				let title = link.text().unwrap_or_else(|| format!("Chapitre {}", chapter_number));
				let clean_title = if title.contains("Chapitre") || title.contains("Chapter") {
					title
				} else {
					format!("Chapitre {}", chapter_number)
				};
		
				chapters.push(Chapter {
					key: chapter_key.clone(),
					title: Some(clean_title),
					chapter_number: Some(chapter_number),
					volume_number: None,
					date_uploaded: None,
					scanlators: None,
					language: Some(String::from("fr")),
					locked: false,
					thumbnail: None,
					url: Some(super::helper::make_absolute_url("https://lelscanfr.com", &chapter_key)),
				});
			}
		}
	}
	
	Ok(chapters)
}

pub fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	// Multiple selectors for different image containers
	let image_selectors = [
		"img[src*=\"storage/chapters/\"]",
		"#chapter-container .chapter-image",
		".chapter-container img",
		".page-image",
		".manga-page img",
		"img[data-src]",
		"img[src]"
	];
	
	let mut images = None;
	for selector in image_selectors {
		if let Some(found_images) = html.select(selector) {
			if found_images.first().is_some() {
				images = html.select(selector);
				break;
			}
		}
	}

	if let Some(images) = images {
		for img in images {
			// Try multiple attributes for image URL
			let url = img.attr("data-src")
				.or_else(|| img.attr("data-lazy-src"))
				.or_else(|| img.attr("src"))
				.unwrap_or_default();
			
			if !url.is_empty() {
				let absolute_url = super::helper::make_absolute_url("https://lelscanfr.com", &url);
				pages.push(Page {
					content: PageContent::url(absolute_url),
					thumbnail: None,
					has_description: false,
					description: None,
				});
			}
		}
	}

	Ok(pages)
}
