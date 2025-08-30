use aidoku::{
	Result, Manga, Page, PageContent, MangaPageResult, MangaStatus, Chapter,
	ContentRating, Viewer, UpdateStrategy,
	alloc::{String, Vec, vec, format},
	imports::html::Document,
};
use core::cmp::Ordering;

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

	// Check for pagination - improved detection
	let has_more = 'pagination_check: {
		// Method 1: Check for pagination text "Page X of Y"
		if let Some(pagination_elements) = html.select("div, span, p") {
			for elem in pagination_elements {
				if let Some(text) = elem.text() {
					// Look for "Page X of Y" pattern
					if text.contains("Page ") && text.contains(" of ") {
						if let Some(of_pos) = text.find(" of ") {
							let after_of = &text[of_pos + 4..].trim();
							if let Some(total_pages_str) = after_of.split_whitespace().next() {
								if let Ok(total_pages) = total_pages_str.parse::<i32>() {
									if let Some(page_start) = text.find("Page ") {
										let after_page = &text[page_start + 5..of_pos];
										if let Ok(current_page) = after_page.trim().parse::<i32>() {
											// Found "Page X of Y" - return exact result
											break 'pagination_check current_page < total_pages;
										}
									}
								}
							}
						}
					}
				}
			}
		}
		
		// Method 2: Check for pagination controls with ellipsis
		let pagination_selectors = [".pagination", ".page-numbers", ".pages"];
		for selector in pagination_selectors {
			if let Some(pagination) = html.select(selector) {
				if let Some(first_pagination) = pagination.first() {
					if let Some(pagination_text) = first_pagination.text() {
						// Check for "..." which indicates more pages
						if pagination_text.contains("…") || pagination_text.contains("...") {
							break 'pagination_check true;
						}
					}
					break;
				}
			}
		}
		
		// Method 3: Check for numbered pagination links
		let mut max_page = 0;
		for selector in pagination_selectors {
			if let Some(pagination) = html.select(selector) {
				if let Some(first_pagination) = pagination.first() {
					// Look for number links
					if let Some(links) = first_pagination.select("a") {
						for link in links {
							if let Some(link_text) = link.text() {
								if let Ok(page_num) = link_text.trim().parse::<i32>() {
									if page_num > max_page {
										max_page = page_num;
									}
								}
							}
						}
					}
					break;
				}
			}
		}
		
		// If we found numbered pages > 1, likely more pages exist
		if max_page > 1 {
			true
		} else {
			// Method 4: Fallback - if we have many mangas, assume more pages
			mangas.len() >= 15 // LelscanFR typically shows 20+ per page
		}
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
	
	// Extract tags/genres
	let mut tags: Vec<String> = Vec::new();
	let tag_selectors = [
		"a[href*=\"?genre=\"]",
		".genre a",
		".genres a",
		".tag a",
		".tags a"
	];
	
	for selector in tag_selectors {
		if let Some(tag_elements) = html.select(selector) {
			for tag_elem in tag_elements {
				if let Some(tag_text) = tag_elem.text() {
					let clean_tag = tag_text.trim();
					if !clean_tag.is_empty() && !tags.contains(&String::from(clean_tag)) {
						tags.push(String::from(clean_tag));
					}
				}
			}
			if !tags.is_empty() {
				break; // Found tags with this selector, stop trying others
			}
		}
	}
	
	if !tags.is_empty() {
		manga.tags = Some(tags);
	}
	
	// Extract manga status 
	let status_selectors = [
		"a[href*=\"?status=\"]",
		".status",
		".manga-status",
		"span:contains(Statut)+span",
		"span:contains(Status)+span"
	];
	
	for selector in status_selectors {
		if let Some(status_elements) = html.select(selector) {
			if let Some(status_elem) = status_elements.first() {
				if let Some(status_text) = status_elem.text() {
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
	}
	
	Ok(manga)
}

pub fn parse_chapter_list(manga_key: &str, all_html: Vec<Document>) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for html in all_html {
		// Get ALL links first to debug
		if let Some(all_links) = html.select("a") {
			for link in all_links {
				let href = link.attr("href").unwrap_or_default();
				let link_text = link.text().unwrap_or_default();
				
				// Very broad check - any link that looks like a chapter
				if href.contains(&format!("/manga/{}/", manga_key)) && 
				   (link_text.contains("Chapitre") || href.split('/').last().unwrap_or("").parse::<f32>().is_ok()) {
					
					// Extract chapter number from URL (most reliable)
					let chapter_number: f32 = if let Some(url_part) = href.split('/').last() {
						url_part.parse().unwrap_or_else(|_| {
							// Fallback to text extraction if URL parsing fails
							if let Some(num_start) = link_text.find("Chapitre ") {
								let after_chapitre = &link_text[num_start + 9..];
								let num_str: String = after_chapitre
									.chars()
									.take_while(|c| c.is_ascii_digit() || *c == '.')
									.collect();
								num_str.parse().unwrap_or(0.0)
							} else {
								0.0
							}
						})
					} else {
						0.0
					};
					
					if chapter_number > 0.0 {
						// Create clean chapter key (relative path)  
						let chapter_key = if href.starts_with("http") {
							href.replace("https://lelscanfr.com", "")
						} else {
							href
						};
						
						// Clean up chapter title - remove extra info like dates and view counts
						let chapter_title = if !link_text.is_empty() && link_text.contains("Chapitre") {
							// Extract clean title from messy text like "Ch.710 - Chapitre 710 il y a 1 an 3679"
							if let Some(chapitre_pos) = link_text.find("Chapitre ") {
								let from_chapitre = &link_text[chapitre_pos..];
								// Take first two words: "Chapitre" + number
								let words: Vec<&str> = from_chapitre.split_whitespace().collect();
								if words.len() >= 2 {
									format!("{} {}", words[0], words[1])
								} else {
									format!("Chapitre {}", chapter_number)
								}
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
							url: Some(super::helper::make_absolute_url("https://lelscanfr.com", &chapter_key)),
						});
					}
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

pub fn parse_page_list(html: &Document) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	// Use the exact selector from the old working implementation
	let image_selectors = [
		"#chapter-container .chapter-image",  // Original working selector
		"#chapter-container img",             // Alternative in same container
		".chapter-image",                     // Fallback without container
		"img.chapter-image",                  // Specific class selector
		"img.lazyload",                       // Lazyload images
		"img[data-src]",                      // Any image with data-src
		"img"                                 // Final fallback
	];

	for selector in image_selectors {
		if let Some(images) = html.select(selector) {
			for img in images {
				// Use the exact attribute extraction from old implementation
				let img_url = img.attr("data-src")
					.or_else(|| img.attr("src"))
					.unwrap_or_default();
				
				if !img_url.is_empty() && !img_url.starts_with("data:") {
					// Ensure URL is absolute
					let absolute_url = if img_url.starts_with("http") {
						img_url
					} else {
						super::helper::make_absolute_url("https://lelscanfr.com", &img_url)
					};
					
					pages.push(Page {
						content: PageContent::Url(absolute_url, None),
						thumbnail: None,
						has_description: false,
						description: None,
					});
				}
			}
			
			// If we found pages with this selector, stop trying others
			if !pages.is_empty() {
				break;
			}
		}
	}

	Ok(pages)
}

