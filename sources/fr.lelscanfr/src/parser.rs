use aidoku::{
	Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, ContentRating, Viewer, Chapter,
	UpdateStrategy,
	alloc::{String, Vec, vec, format},
	imports::html::Document,
};

extern crate alloc;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
	// Create default result that we'll always return successfully
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Add a debug entry to show we're processing
	mangas.push(Manga {
		key: String::from("debug-processing"),
		cover: None,
		title: String::from("DEBUG: Processing manga list..."),
		authors: None,
		artists: None,
		description: None,
		tags: None,
		status: MangaStatus::Unknown,
		content_rating: ContentRating::Safe,
		viewer: Viewer::LeftToRight,
		chapters: None,
		url: Some(String::from("https://lelscanfr.com/manga/debug-processing")),
		next_update_time: None,
		update_strategy: UpdateStrategy::Always,
	});

	// Try to select manga links - if this fails, we still have the debug entry
	if let Some(manga_links) = html.select("a[href*=\"/manga/\"]") {
		let total_links = manga_links.count();
		
		// Add debug info about total links found
		mangas.push(Manga {
			key: String::from("debug-links-found"),
			cover: None,
			title: format!("DEBUG: Found {} links with '/manga/' in href", total_links),
			authors: None,
			artists: None,
			description: None,
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::LeftToRight,
			chapters: None,
			url: Some(String::from("https://lelscanfr.com/manga/debug-links")),
			next_update_time: None,
			update_strategy: UpdateStrategy::Always,
		});
		
		// Try to re-select and process (count() consumes the iterator)
		if let Some(manga_links_again) = html.select("a[href*=\"/manga/\"]") {
			let mut processed_count = 0;
			let mut manga_count = 0;
			
			for item in manga_links_again {
				processed_count += 1;
				
				// Only process first few to avoid overwhelming debug output
				if processed_count > 5 {
					break;
				}
				
				// Try to get title safely
				let title = if let Some(h2_elements) = item.select("h2") {
					if let Some(h2) = h2_elements.first() {
						let title_text = h2.text().unwrap_or(String::from("NO_TEXT"));
						if !title_text.is_empty() && title_text != "NO_TEXT" {
							title_text
						} else {
							format!("DEBUG: Empty h2 text #{}", processed_count)
						}
					} else {
						format!("DEBUG: h2 found but no first() #{}", processed_count)
					}
				} else {
					format!("DEBUG: No h2 in link #{}", processed_count)
				};

				// Add debug entry for each link processed
				let url = item.attr("href").unwrap_or(String::from("NO_HREF"));
				
				mangas.push(Manga {
					key: format!("debug-link-{}", processed_count),
					cover: None,
					title: format!("DEBUG Link #{}: {} | URL: {}", processed_count, title, url),
					authors: None,
					artists: None,
					description: None,
					tags: None,
					status: MangaStatus::Unknown,
					content_rating: ContentRating::Safe,
					viewer: Viewer::LeftToRight,
					chapters: None,
					url: Some(format!("https://lelscanfr.com/manga/debug-{}", processed_count)),
					next_update_time: None,
					update_strategy: UpdateStrategy::Always,
				});
				
				// Count actual manga entries (those with h2 and proper URLs)
				if title.starts_with("DEBUG: Empty") || title.starts_with("DEBUG: No h2") || url.contains("?genre=") {
					// Skip debug entries or genre links
				} else {
					manga_count += 1;
				}
			}
			
			// Add summary debug entry
			mangas.push(Manga {
				key: String::from("debug-summary"),
				cover: None,
				title: format!("DEBUG: Processed {} links, found {} manga entries", processed_count, manga_count),
				authors: None,
				artists: None,
				description: None,
				tags: None,
				status: MangaStatus::Unknown,
				content_rating: ContentRating::Safe,
				viewer: Viewer::LeftToRight,
				chapters: None,
				url: Some(String::from("https://lelscanfr.com/manga/debug-summary")),
				next_update_time: None,
				update_strategy: UpdateStrategy::Always,
			});
		}
	} else {
		// No manga links found at all
		mangas.push(Manga {
			key: String::from("debug-no-links"),
			cover: None,
			title: String::from("DEBUG: No links found with '/manga/' in href"),
			authors: None,
			artists: None,
			description: None,
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::LeftToRight,
			chapters: None,
			url: Some(String::from("https://lelscanfr.com/manga/debug-no-links")),
			next_update_time: None,
			update_strategy: UpdateStrategy::Always,
		});
	}

	// Always return success - never fail
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: false, // Keep it simple for debugging
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
