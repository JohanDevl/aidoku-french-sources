use aidoku::{
	error::Result, prelude::*, std::{
		html::Node, ObjectRef, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use core::cmp::Ordering;

use crate::BASE_URL;

// Extract Next.js series data from /series page
fn extract_nextjs_series_data(html: &Node) -> Result<Vec<Manga>> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// First try __NEXT_DATA__ script tag
	for script in html.select("script#__NEXT_DATA__").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if let Ok(nextjs_mangas) = parse_nextjs_series_data(&content) {
			mangas.extend(nextjs_mangas);
			if !mangas.is_empty() {
				return Ok(mangas);
			}
		}
	}
	
	// Fallback to self.__next_f.push() patterns
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if content.contains("self.__next_f.push") && 
		   (content.contains("\"mangas\":[") || content.contains("\"series\":[")) {
			if let Ok(nextjs_mangas) = parse_nextjs_push_data(&content) {
				mangas.extend(nextjs_mangas);
				if mangas.len() >= 20 {
					break;
				}
			}
		}
	}
	
	Ok(mangas)
}

// Parse __NEXT_DATA__ JSON content for series data
fn parse_nextjs_series_data(_content: &str) -> Result<Vec<Manga>> {
	// TODO: Implement __NEXT_DATA__ JSON parsing
	// This would parse JSON structure like:
	// { "props": { "pageProps": { "mangas": [...] } } }
	Ok(Vec::new())
}

// Parse self.__next_f.push() content for manga data  
fn parse_nextjs_push_data(_content: &str) -> Result<Vec<Manga>> {
	// TODO: Implement self.__next_f.push() pattern parsing
	// This would extract JSON from patterns like:
	// self.__next_f.push([1, "...escaped JSON..."])
	Ok(Vec::new())
}

// Extract Next.js manga details data from manga detail page
fn extract_nextjs_manga_details(html: &Node) -> Result<ObjectRef> {
	// First try __NEXT_DATA__ script tag
	for script in html.select("script#__NEXT_DATA__").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if let Ok(manga_data) = parse_nextjs_details_data(&content) {
			return Ok(manga_data);
		}
	}
	
	// Fallback to self.__next_f.push() patterns
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if content.contains("self.__next_f.push") && 
		   (content.contains("\"manga\":{") || content.contains("\"initialData\":{")) {
			if let Ok(manga_data) = parse_nextjs_push_manga_data(&content) {
				return Ok(manga_data);
			}
		}
	}
	
	// Return empty object if extraction fails
	use aidoku::std::json::parse;
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Parse __NEXT_DATA__ JSON content for manga details
fn parse_nextjs_details_data(_content: &str) -> Result<ObjectRef> {
	// TODO: Implement __NEXT_DATA__ JSON parsing for manga details
	// This would parse JSON structure like:
	// { "props": { "pageProps": { "manga": {...} } } }
	use aidoku::std::json::parse;
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Parse self.__next_f.push() content for manga details
fn parse_nextjs_push_manga_data(_content: &str) -> Result<ObjectRef> {
	// TODO: Implement self.__next_f.push() pattern parsing for manga details
	// This would extract JSON from patterns like:
	// self.__next_f.push([1, "...escaped JSON containing manga object..."])
	use aidoku::std::json::parse;
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Parse manga status from French status string
fn parse_manga_status(status: &str) -> MangaStatus {
	match status.trim().to_lowercase().as_str() {
		"en cours" => MangaStatus::Ongoing,
		"terminé" => MangaStatus::Completed,
		"en pause" | "hiatus" => MangaStatus::Hiatus,
		"annulé" | "abandonné" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	}
}

// Parse ISO date string with special prefix handling
fn parse_iso_date(date_string: &str) -> i64 {
	if date_string.is_empty() {
		return 0;
	}
	
	// Clean up date string - remove special prefixes used by PoseidonScans
	let cleaned_date = if date_string.starts_with("\"$D") {
		date_string.strip_prefix("\"$D").unwrap_or(date_string).strip_suffix("\"").unwrap_or(date_string)
	} else if date_string.starts_with("$D") {
		date_string.strip_prefix("$D").unwrap_or(date_string)
	} else if date_string.starts_with('"') && date_string.ends_with('"') && date_string.len() > 2 {
		&date_string[1..date_string.len()-1]
	} else {
		date_string
	};
	
	// Try to parse ISO 8601 format: 2024-01-15T10:30:00.000Z
	if let Ok(timestamp) = parse_date_iso(cleaned_date) {
		timestamp
	} else {
		0
	}
}

// Simple ISO 8601 date parser (basic implementation)
fn parse_date_iso(_date_str: &str) -> Result<i64> {
	// For now, return 0 to avoid compilation errors
	// TODO: Implement proper ISO 8601 parsing
	// This would parse dates like "2024-01-15T10:30:00.000Z"
	Ok(0)
}

// Extract a title from a description (first sentence or meaningful part)
fn extract_title_from_description(description: &str) -> String {
	// Try to find a title-like pattern (first sentence ending with .)
	if let Some(first_sentence_end) = description.find('.') {
		let first_sentence = description[..first_sentence_end].trim();
		if first_sentence.len() > 10 && first_sentence.len() < 80 {
			return String::from(first_sentence);
		}
	}
	
	// Fallback: take first meaningful chunk (up to first major punctuation)
	let chunk_end = description.find(['.', '!', '?', ','])
		.unwrap_or_else(|| description.len().min(60));
	
	let title_chunk = description[..chunk_end].trim();
	if title_chunk.len() > 5 {
		String::from(title_chunk)
	} else {
		// Ultimate fallback: generic title with ID indication
		String::from("Manga série disponible")
	}
}

// Parse search results with client-side filtering on /series page
pub fn parse_search_manga(search_query: String, html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let query_lower = search_query.to_lowercase();
	
	// Parse all manga from series page - try Next.js data first, fallback to HTML
	if let Ok(nextjs_mangas) = extract_nextjs_series_data(&html) {
		// Use Next.js extracted data for better accuracy
		for manga in nextjs_mangas {
			// Client-side search filtering
			if query_lower.trim().is_empty() {
				mangas.push(manga);
			} else {
				let title_lower = manga.title.to_lowercase();
				let description_lower = manga.description.to_lowercase();
				let id_lower = manga.id.to_lowercase();
				
				if title_lower.contains(&query_lower) || 
				   description_lower.contains(&query_lower) || 
				   id_lower.contains(&query_lower) {
					mangas.push(manga);
				}
			}
			
			// Limit results
			if mangas.len() >= 30 {
				break;
			}
		}
	} else {
		// Fallback to HTML parsing if Next.js extraction fails
		for link in html.select("a[href*='/serie/']").array() {
			let link = link.as_node()?;
			let href = link.attr("href").read();
			
			let slug = if let Some(slug_start) = href.rfind('/') {
				String::from(&href[slug_start + 1..])
			} else {
				continue;
			};
			
			if slug == "unknown" || slug.is_empty() {
				continue;
			}
			
			let title_element = link.select(".title, .manga-title, h3, .name, .entry-title").first();
			let title = if !title_element.html().is_empty() {
				title_element.text().read()
			} else {
				String::from(&slug.replace("-", " "))
			};
			
			// Client-side search filtering
			if !query_lower.trim().is_empty() {
				let title_lower = title.to_lowercase();
				let slug_lower = slug.to_lowercase();
				
				if !title_lower.contains(&query_lower) && !slug_lower.contains(&query_lower) {
					continue;
				}
			}
			
			let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug);
			let url = format!("{}/serie/{}", String::from(BASE_URL), slug);
			
			mangas.push(Manga {
				id: slug,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url,
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
			
			if mangas.len() >= 30 {
				break;
			}
		}
	}
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
	})
}

// Parse latest manga from API response  
pub fn parse_latest_manga(json: ObjectRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	if let Ok(data_array) = json.get("data").as_array() {
		for item in data_array {
			let manga = item.as_object()?;
			
			let slug = manga.get("slug").as_string()?.read();
			if slug == "unknown" || slug.is_empty() {
				continue;
			}
			
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug);

			mangas.push(Manga {
				id: slug,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
		}
		
		has_more = mangas.len() == 20;
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// Parse manga details with Next.js data extraction
pub fn parse_manga_details(manga_id: String, html: Node) -> Result<Manga> {
	// Extract Next.js page data
	let manga_data = extract_nextjs_manga_details(&html)?;
	
	// Get basic info with fallbacks
	let title = if let Ok(title_str) = manga_data.get("title").as_string() {
		title_str.read()
	} else {
		format!("Manga {}", manga_id)
	};
	
	let slug = if let Ok(slug_str) = manga_data.get("slug").as_string() {
		slug_str.read()
	} else {
		manga_id.clone()
	};
	
	// Author and artist
	let author = if let Ok(author_str) = manga_data.get("author").as_string() {
		let author_text = author_str.read();
		if author_text.is_empty() { String::new() } else { author_text }
	} else {
		String::new()
	};
	
	let artist = if let Ok(artist_str) = manga_data.get("artist").as_string() {
		let artist_text = artist_str.read();
		if artist_text.is_empty() { String::new() } else { artist_text }
	} else {
		String::new()
	};
	
	// Status parsing
	let status = if let Ok(status_str) = manga_data.get("status").as_string() {
		parse_manga_status(&status_str.read())
	} else {
		MangaStatus::Unknown
	};
	
	// Categories/genres
	let mut categories = Vec::new();
	if let Ok(categories_array) = manga_data.get("categories").as_array() {
		for category in categories_array {
			if let Ok(cat_obj) = category.as_object() {
				if let Ok(name) = cat_obj.get("name").as_string() {
					let genre = String::from(name.read().trim());
					if !genre.is_empty() {
						categories.push(genre);
					}
				}
			}
		}
	}
	
	// Description - try HTML selector first, then JSON fallback
	let mut description = String::new();
	
	// Try to get description from HTML using CSS selector
	let desc_element = html.select("p.text-gray-300.leading-relaxed.whitespace-pre-line").first();
	if !desc_element.html().is_empty() {
		let html_desc = String::from(desc_element.text().read().trim());
		if !html_desc.is_empty() {
			// Remove redundant title prefix if present
			description = String::from(html_desc.replace(&format!("Dans : {}", title), "").trim());
		}
	}
	
	// Fallback to JSON description if HTML extraction failed
	if description.is_empty() {
		if let Ok(json_desc) = manga_data.get("description").as_string() {
			let desc = String::from(json_desc.read().trim());
			if desc.len() > 5 && !desc.starts_with('$') {
				description = desc;
			}
		}
	}
	
	// Add alternative names if available
	if let Ok(alt_names) = manga_data.get("alternativeNames").as_string() {
		let alt_names = String::from(alt_names.read().trim());
		if !alt_names.is_empty() {
			if description == "Aucune description." || description.is_empty() {
				description = format!("Noms alternatifs: {}", alt_names);
			} else {
				description = format!("{}\n\nNoms alternatifs: {}", description, alt_names);
			}
		}
	}
	
	// Set default description if still empty
	if description.is_empty() {
		description = String::from("Aucune description disponible.");
	}
	
	// Construct URLs
	let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug);
	let url = format!("{}/serie/{}", String::from(BASE_URL), slug);
	
	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

// Parse chapter list with Next.js data extraction and premium filtering
pub fn parse_chapter_list(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	// Extract Next.js page data
	let manga_data = extract_nextjs_manga_details(&html)?;
	
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Get chapters array from manga data
	if let Ok(chapters_array) = manga_data.get("chapters").as_array() {
		for chapter_value in chapters_array {
			if let Ok(chapter_obj) = chapter_value.as_object() {
				// Skip premium chapters
				if let Ok(is_premium) = chapter_obj.get("isPremium").as_bool() {
					if is_premium {
						continue;
					}
				}
				
				// Get chapter number
				let chapter_number = if let Ok(num) = chapter_obj.get("number").as_float() {
					num
				} else {
					0.0
				};
				
				if chapter_number <= 0.0 {
					continue;
				}
				
				// Format chapter number (remove .0 suffix if present)
				let chapter_number_string = if chapter_number % 1.0 == 0.0 {
					format!("{}", chapter_number as i32)
				} else {
					format!("{}", chapter_number)
				};
				
				// Get chapter title
				let chapter_title = if let Ok(title_str) = chapter_obj.get("title").as_string() {
					let title_text = title_str.read();
					if title_text.trim().is_empty() { None } else { Some(title_text) }
				} else {
					None
				};
				
				// Build chapter name
				let base_name = format!("Chapitre {}", chapter_number_string);
				let name = if let Some(title) = chapter_title {
					format!("{} - {}", base_name, title.trim())
				} else {
					base_name
				};
				
				// Parse date
				let date_upload = if let Ok(date_str) = chapter_obj.get("createdAt").as_string() {
					parse_iso_date(&date_str.read())
				} else {
					0
				};
				
				// Build chapter URL
				let url = format!("/serie/{}/chapter/{}", manga_id, chapter_number_string);
				
				chapters.push(Chapter {
					id: chapter_number_string,
					title: name,
					volume: -1.0,
					chapter: chapter_number as f32,
					date_updated: date_upload as f64,
					scanlator: String::new(),
					url,
					lang: String::from("fr"),
				});
			}
		}
	}
	
	// Sort chapters by number in descending order (latest first)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
	
	Ok(chapters)
}

// Parse page list (simplified)
pub fn parse_page_list(_html: Node) -> Result<Vec<Page>> {
	let pages: Vec<Page> = Vec::new();
	
	// This would need real Next.js data extraction for image URLs
	// For now, return empty list
	
	Ok(pages)
}