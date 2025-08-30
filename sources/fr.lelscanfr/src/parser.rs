use aidoku::{
	Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, ContentRating, Viewer, Chapter,
	UpdateStrategy,
	alloc::{String, Vec, vec, format},
	imports::html::Document,
};

extern crate alloc;

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	// Select manga links that have h2 titles (actual manga, not genre links)
	if let Some(manga_links) = html.select("a[href*=\"/manga/\"]") {
		for item in manga_links {
			// Only process links that have h2 elements (actual manga entries)
			if let Some(h2_elements) = item.select("h2") {
				if let Some(h2) = h2_elements.first() {
					let title = h2.text().unwrap_or_default();
					if title.is_empty() {
						continue;
					}
					
					// Get URL and validate
					let url = item.attr("href").unwrap_or_default();
					if url.is_empty() || url.contains("?genre=") || url.contains("?tag=") {
						continue; // Skip genre links
					}
					
					// Extract manga key/slug from URL
					let key = if url.starts_with("http") {
						super::helper::extract_id_from_url(&url)
					} else {
						// Handle relative URLs like "/manga/title"
						let parts: Vec<&str> = url.split('/').collect();
						if parts.len() >= 3 && parts[1] == "manga" {
							String::from(parts[2].trim())
						} else {
							continue;
						}
					};
					
					if key.is_empty() {
						continue;
					}
					
					// Extract cover image if available
					let cover = if let Some(img_elements) = item.select("img") {
						if let Some(img) = img_elements.first() {
							let img_src = img.attr("src")
								.or_else(|| img.attr("data-src"))
								.or_else(|| img.attr("data-lazy-src"))
								.unwrap_or_default();
							if img_src.is_empty() {
								None
							} else {
								Some(super::helper::make_absolute_url("https://lelscanfr.com", &img_src))
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
						url: Some(super::helper::make_absolute_url("https://lelscanfr.com", &url)),
						next_update_time: None,
						update_strategy: UpdateStrategy::Always,
					});
				}
			}
		}
	}

	// Check for pagination
	let has_more = if let Some(pagination) = html.select(".pagination") {
		pagination.count() > 0 && mangas.len() >= 10
	} else {
		mangas.len() >= 10
	};

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
		// Select chapter links based on actual site structure  
		// Format: a[href*="/manga/{manga_key}/"] containing "Chapitre"
		let chapter_selector = format!("a[href*=\"/manga/{}/\"]", manga_key);
		
		if let Some(chapter_links) = html.select(&chapter_selector) {
			for link in chapter_links {
				let href = link.attr("href").unwrap_or_default();
				let link_text = link.text().unwrap_or_default();
				
				// Skip if empty or doesn't look like a chapter link
				if href.is_empty() || !link_text.contains("Chapitre") {
					continue;
				}
				
				// Extract chapter number from link text (format: "Chapitre 1158")
				let chapter_number: f32 = if let Some(num_start) = link_text.find("Chapitre ") {
					let after_chapitre = &link_text[num_start + 9..]; // Skip "Chapitre "
					let num_str: String = after_chapitre
						.chars()
						.take_while(|c| c.is_ascii_digit() || *c == '.')
						.collect();
					num_str.parse().unwrap_or(0.0)
				} else {
					0.0
				};
				
				if chapter_number == 0.0 {
					continue; // Skip invalid chapters
				}
				
				// Create clean chapter key (relative path)
				let chapter_key = if href.starts_with("http") {
					href.replace("https://lelscanfr.com", "")
				} else {
					href
				};
				
				let chapter_title = format!("Chapitre {}", chapter_number);
		
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
