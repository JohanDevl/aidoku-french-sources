use aidoku::{
	error::Result, prelude::*, std::{
		html::Node, ObjectRef, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;

// Extract manga data from Next.js push scripts with improved pattern detection
fn extract_nextjs_manga_data(script_content: &str) -> Result<Vec<Manga>> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Parse Next.js push data for manga descriptions with T-prefixed IDs
	// Try multiple patterns: :T and ,T
	let patterns = [":T", ",T"];
	
	for pattern in patterns {
		let mut current_pos = 0;
		while let Some(start_pos) = script_content[current_pos..].find(pattern) {
			let actual_start = current_pos + start_pos + pattern.len(); // Skip the pattern
			current_pos = actual_start + 1;
			
			// Extract the T-prefixed ID (until comma or other delimiter)
			let id_end = script_content[actual_start..].find([',', ' ', '\n', '"', '\\'])
				.map(|pos| actual_start + pos)
				.unwrap_or_else(|| (actual_start + 10).min(script_content.len()));
			
			if id_end > actual_start {
				let manga_id = String::from(&script_content[actual_start..id_end]);
				
				// Skip if not a valid T-ID format
				if !manga_id.starts_with('T') || manga_id.len() < 3 {
					continue;
				}
				
				// Find the start of description (after the comma)
				if let Some(comma_pos) = script_content[id_end..].find(',') {
					let desc_start = id_end + comma_pos + 1;
					
					// Extract description (until next manga ID pattern or end marker)
					let mut desc_end = desc_start + 1000; // Default length
					
					// Look for next manga pattern or other end markers
					for next_pattern in &patterns {
						if let Some(next_pos) = script_content[desc_start..].find(next_pattern) {
							desc_end = desc_end.min(desc_start + next_pos);
						}
					}
					
					// Also check for common end markers
					if let Some(end_pos) = script_content[desc_start..].find(['"', '\\', '\n']) {
						desc_end = desc_end.min(desc_start + end_pos);
					}
					
					desc_end = desc_end.min(script_content.len());
					
					if desc_end > desc_start {
						let description = String::from(script_content[desc_start..desc_end].trim());
						
						// Only process if description is substantial (not a fragment) - relaxed criteria
						if description.len() > 20 && description.chars().filter(|c| c.is_alphabetic()).count() > 15 {
							// Skip duplicates
							if mangas.iter().any(|m| m.id == manga_id.to_lowercase()) {
								continue;
							}
							
							// Create manga title from description (first sentence or truncated)
							let title = extract_title_from_description(&description);
							let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), manga_id.to_lowercase());
							let url = format!("{}/serie/{}", String::from(BASE_URL), manga_id.to_lowercase());
							
							// Truncate description for display
							let display_description = if description.len() > 200 {
								format!("{}...", &description[..200])
							} else {
								description
							};
							
							mangas.push(Manga {
								id: manga_id.to_lowercase(),
								cover,
								title,
								author: String::new(),
								artist: String::new(),
								description: display_description,
								url,
								categories: Vec::new(),
								status: MangaStatus::Unknown,
								nsfw: MangaContentRating::Safe,
								viewer: MangaViewer::Scroll
							});
							
							// Limit results to avoid too many
							if mangas.len() >= 25 {
								break;
							}
						}
					}
				}
			}
		}
		
		// If we found enough manga with the first pattern, stop
		if mangas.len() >= 15 {
			break;
		}
	}
	
	Ok(mangas)
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

// Parse search results with client-side filtering using Next.js data extraction
pub fn parse_search_manga(search_query: String, html: Node) -> Result<MangaPageResult> {
	let query = search_query.to_lowercase();
	
	// If search query is empty, return popular manga
	if query.trim().is_empty() {
		return parse_popular_manga(html);
	}

	// Extract all manga from Next.js scripts first
	let mut all_mangas: Vec<Manga> = Vec::new();
	
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		// Look for self.__next_f.push() patterns with manga data
		if content.contains("self.__next_f.push") && (content.contains(":T") || content.contains(",T")) {
			// Extract manga data from this script
			if let Ok(script_mangas) = extract_nextjs_manga_data(&content) {
				all_mangas.extend(script_mangas);
				
				// Continue collecting until we have enough for filtering
				if all_mangas.len() >= 100 {
					break;
				}
			}
		}
	}
	
	// Client-side filtering on extracted manga data
	let mut filtered_mangas: Vec<Manga> = Vec::new();
	
	for manga in all_mangas {
		// Check if query matches title, ID, or description
		let title_lower = manga.title.to_lowercase();
		let id_lower = manga.id.to_lowercase();
		let desc_lower = manga.description.to_lowercase();
		
		let matches = title_lower.contains(&query) ||
					  id_lower.contains(&query) ||
					  desc_lower.contains(&query);
		
		if matches {
			filtered_mangas.push(manga);
			
			// Limit search results
			if filtered_mangas.len() >= 30 {
				break;
			}
		}
	}
	
	Ok(MangaPageResult {
		manga: filtered_mangas,
		has_more: false, // No pagination for search
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

// Parse popular manga from /series HTML page using Next.js data extraction
pub fn parse_popular_manga(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Extract data from Next.js scripts
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		// Look for self.__next_f.push() patterns with manga data
		if content.contains("self.__next_f.push") && (content.contains(":T") || content.contains(",T")) {
			// Extract manga data from this script
			if let Ok(script_mangas) = extract_nextjs_manga_data(&content) {
				mangas.extend(script_mangas);
				
				// Stop after finding enough manga
				if mangas.len() >= 20 {
					break;
				}
			}
		}
	}
	
	// Limit to 20 results for popular manga listing
	mangas.truncate(20);
	
	let has_more = mangas.len() >= 20;
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// Parse manga details (simplified)  
pub fn parse_manga_details(manga_id: String, _html: Node) -> Result<Manga> {
	// Simplified implementation - would need real Next.js data extraction
	let title = format!("Manga {}", manga_id);
	let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), manga_id);
	let url = format!("{}/serie/{}", String::from(BASE_URL), manga_id);
	
	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description: String::from("Aucune description disponible."),
		url,
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

// Parse chapter list (simplified)
pub fn parse_chapter_list(_manga_id: String, _html: Node) -> Result<Vec<Chapter>> {
	let chapters: Vec<Chapter> = Vec::new();
	
	// This would need real Next.js data extraction  
	// For now, return empty list to avoid compilation errors
	
	Ok(chapters)
}

// Parse page list (simplified)
pub fn parse_page_list(_html: Node) -> Result<Vec<Page>> {
	let pages: Vec<Page> = Vec::new();
	
	// This would need real Next.js data extraction for image URLs
	// For now, return empty list
	
	Ok(pages)
}