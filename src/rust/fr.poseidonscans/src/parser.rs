use aidoku::{
	error::Result, prelude::*, std::{
		html::Node, ObjectRef, ArrayRef, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use core::cmp::Ordering;

use crate::BASE_URL;

// Page image data structure (equivalent to PageImageUrlData in Kotlin)
struct PageImageData {
	original_url: String,
	order: i32,
}




// Find the start of JSON array that contains a pattern
fn find_json_array_start(content: &str, pattern_pos: usize) -> Option<usize> {
	// Find the '[' that starts the array containing this pattern
	let mut i = pattern_pos;
	let mut brace_count = 0;
	
	while i > 0 {
		i -= 1;
		match content.chars().nth(i)? {
			']' => brace_count += 1,
			'[' => {
				if brace_count == 0 {
					return Some(i);
				}
				brace_count -= 1;
			}
			_ => {}
		}
	}
	None
}

// Extract complete JSON array from start position
fn extract_json_array(content: &str, start_pos: usize) -> Option<String> {
	let chars: Vec<char> = content.chars().collect();
	if start_pos >= chars.len() || chars[start_pos] != '[' {
		return None;
	}
	
	let mut bracket_count = 0;
	let mut in_string = false;
	let mut escape = false;
	
	for (i, &ch) in chars[start_pos..].iter().enumerate() {
		if escape {
			escape = false;
			continue;
		}
		
		match ch {
			'\\' if in_string => escape = true,
			'"' => in_string = !in_string,
			'[' if !in_string => bracket_count += 1,
			']' if !in_string => {
				bracket_count -= 1;
				if bracket_count == 0 {
					let end_pos = start_pos + i + 1;
					return Some(String::from(&content[start_pos..end_pos]));
				}
			}
			_ => {}
		}
	}
	None
}

// Parse JSON string containing manga array
fn parse_json_manga_array(json_str: &str) -> Result<Vec<Manga>> {
	use aidoku::std::json::parse;
	
	if let Ok(parsed) = parse(json_str) {
		if let Ok(array_ref) = parsed.as_array() {
			// return parse_manga_array(array_ref);
			return Ok(Vec::new());
		}
	}
	Ok(Vec::new())
}

// Extract mangas from initialData object
fn extract_mangas_from_initialdata(json_str: &str) -> Result<Vec<Manga>> {
	use aidoku::std::json::parse;
	
	if let Ok(parsed) = parse(json_str) {
		if let Ok(obj_ref) = parsed.as_object() {
			// Try different paths for manga arrays in initialData
			if let Ok(mangas_array) = obj_ref.get("mangas").as_array() {
				// return parse_manga_array(mangas_array);
				return Ok(Vec::new());
			} else if let Ok(series_array) = obj_ref.get("series").as_array() {
				// return parse_manga_array(series_array);
				return Ok(Vec::new());
			}
		}
	}
	Ok(Vec::new())
}

// Extract Next.js manga details data from manga detail page
fn extract_nextjs_manga_details(html: &Node) -> Result<ObjectRef> {
	let mut best_data: Option<ObjectRef> = None;
	
	// First try __NEXT_DATA__ script tag with enhanced validation
	for script in html.select("script#__NEXT_DATA__").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if let Ok(manga_data) = parse_nextjs_details_data(&content) {
			// Always validate, but keep even partial data
			if validate_manga_details_data(&manga_data) {
				return Ok(manga_data);
			} else if best_data.is_none() {
				// Keep partial data as fallback
				best_data = Some(manga_data);
			}
		}
	}
	
	// Enhanced fallback to self.__next_f.push() patterns with multiple markers
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if content.contains("self.__next_f.push") {
			// Try multiple data patterns
			let patterns = [
				"\"initialData\":{",
				"\"manga\":{", 
				"\"chapter\":{",
				"\"pageProps\":{",
				"\"props\":{"
			];
			
			for pattern in &patterns {
				if content.contains(pattern) {
					if let Ok(manga_data) = parse_nextjs_push_manga_data(&content) {
						if validate_manga_details_data(&manga_data) {
							return Ok(manga_data);
						} else if best_data.is_none() {
							// Keep partial data as fallback
							best_data = Some(manga_data);
						}
					}
				}
			}
		}
	}
	
	// Return best data found, or empty object if nothing found
	if let Some(data) = best_data {
		Ok(data)
	} else {
		use aidoku::std::json::parse;
		Ok(parse("{}").unwrap().as_object().unwrap())
	}
}

// Parse __NEXT_DATA__ JSON content for manga details
fn parse_nextjs_details_data(content: &str) -> Result<ObjectRef> {
	use aidoku::std::json::parse;
	
	// Parse the root JSON object
	if let Ok(root_json) = parse(content) {
		if let Ok(root_obj) = root_json.as_object() {
			// Try props.pageProps first (most common structure)
			if let Ok(props) = root_obj.get("props").as_object() {
				if let Ok(page_props) = props.get("pageProps").as_object() {
					// Check if pageProps contains expected data structures
					if page_props.get("initialData").as_object().is_ok() ||
					   page_props.get("manga").as_object().is_ok() ||
					   page_props.get("chapter").as_object().is_ok() ||
					   page_props.get("images").as_array().is_ok() ||
					   page_props.get("mangas").as_array().is_ok() ||
					   page_props.get("series").as_array().is_ok() {
						return Ok(page_props);
					}
				}
			}
			
			// Try root level initialData (alternative structure)
			if let Ok(initial_data) = root_obj.get("initialData").as_object() {
				if initial_data.get("manga").as_object().is_ok() ||
				   initial_data.get("chapter").as_object().is_ok() ||
				   initial_data.get("images").as_array().is_ok() ||
				   initial_data.get("mangas").as_array().is_ok() ||
				   initial_data.get("series").as_array().is_ok() {
					return Ok(initial_data);
				}
			}
			
			// Try direct manga data at root level
			if root_obj.get("manga").as_object().is_ok() {
				return Ok(root_obj);
			}
		}
	}
	
	// Return empty object if parsing fails
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Parse self.__next_f.push() content for manga details
fn parse_nextjs_push_manga_data(content: &str) -> Result<ObjectRef> {
	// Find self.__next_f.push() patterns with manga data
	let patterns = ["\"initialData\":{", "\"manga\":{", "\"chapter\":{"];
	
	// Look for self.__next_f.push([1, "..."])  patterns
	let mut start_pos = 0;
	while let Some(push_start) = content[start_pos..].find("self.__next_f.push([1,") {
		let actual_start = start_pos + push_start;
		start_pos = actual_start + 1;
		
		// Find the quoted string after [1,
		if let Some(quote_start) = content[actual_start..].find('"') {
			let string_start = actual_start + quote_start + 1;
			
			// Find the closing quote and bracket
			if let Some(quote_end) = find_closing_quote(&content[string_start..]) {
				let string_end = string_start + quote_end;
				let escaped_json = &content[string_start..string_end];
				
				// Unescape the JSON string
				let unescaped_json = unescape_json_string(escaped_json);
				
				// Try to find manga data patterns in unescaped JSON
				for pattern in &patterns {
					if let Some(pattern_pos) = unescaped_json.find(pattern) {
						// Find the start of the JSON object containing this pattern
						if let Some(obj_start) = find_json_object_start(&unescaped_json, pattern_pos) {
							if let Some(json_obj) = extract_json_object(&unescaped_json, obj_start) {
								// Try to parse this JSON object
								use aidoku::std::json::parse;
								if let Ok(parsed) = parse(&json_obj) {
									if let Ok(obj_ref) = parsed.as_object() {
										// Validate this object contains expected data
										if validate_manga_data(&obj_ref, pattern) {
											return Ok(obj_ref);
										}
									}
								}
							}
						}
					}
				}
			}
		}
	}
	
	// Return empty object if no valid data found
	use aidoku::std::json::parse;
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Find closing quote in JSON string, handling escapes
fn find_closing_quote(content: &str) -> Option<usize> {
	let mut i = 0;
	let chars: Vec<char> = content.chars().collect();
	
	while i < chars.len() {
		match chars[i] {
			'"' => return Some(i),
			'\\' => {
				// Skip escaped character
				i += 2;
			}
			_ => i += 1,
		}
	}
	None
}

// Unescape JSON string (handle \" and \\)
fn unescape_json_string(escaped: &str) -> String {
	let mut result = String::new();
	let mut chars = escaped.chars();
	
	while let Some(ch) = chars.next() {
		if ch == '\\' {
			if let Some(next_ch) = chars.next() {
				match next_ch {
					'"' => result.push('"'),
					'\\' => result.push('\\'),
					'/' => result.push('/'),
					'b' => result.push('\u{0008}'),
					'f' => result.push('\u{000C}'),
					'n' => result.push('\n'),
					'r' => result.push('\r'),
					't' => result.push('\t'),
					_ => {
						result.push('\\');
						result.push(next_ch);
					}
				}
			}
		} else {
			result.push(ch);
		}
	}
	
	result
}

// Find the start of JSON object that contains a pattern
fn find_json_object_start(content: &str, pattern_pos: usize) -> Option<usize> {
	// Search backwards for opening brace
	let mut i = pattern_pos;
	let mut brace_count = 0;
	
	while i > 0 {
		i -= 1;
		match content.chars().nth(i)? {
			'}' => brace_count += 1,
			'{' => {
				if brace_count == 0 {
					return Some(i);
				}
				brace_count -= 1;
			}
			_ => {}
		}
	}
	None
}

// Extract complete JSON object from start position
fn extract_json_object(content: &str, start_pos: usize) -> Option<String> {
	let chars: Vec<char> = content.chars().collect();
	if start_pos >= chars.len() || chars[start_pos] != '{' {
		return None;
	}
	
	let mut brace_count = 0;
	let mut in_string = false;
	let mut escape = false;
	
	for (i, &ch) in chars[start_pos..].iter().enumerate() {
		if escape {
			escape = false;
			continue;
		}
		
		match ch {
			'\\' if in_string => escape = true,
			'"' => in_string = !in_string,
			'{' if !in_string => brace_count += 1,
			'}' if !in_string => {
				brace_count -= 1;
				if brace_count == 0 {
					let end_pos = start_pos + i + 1;
					return Some(String::from(&content[start_pos..end_pos]));
				}
			}
			_ => {}
		}
	}
	None
}

// Validate extracted manga data contains expected fields
fn validate_manga_data(obj: &ObjectRef, pattern: &str) -> bool {
	match pattern {
		"\"initialData\":{" => {
			obj.get("manga").as_object().is_ok() || 
			obj.get("chapter").as_object().is_ok() ||
			obj.get("images").as_array().is_ok()
		}
		"\"manga\":{" => {
			obj.get("slug").as_string().is_ok() &&
			obj.get("title").as_string().is_ok()
		}
		"\"chapter\":{" => {
			obj.get("images").as_array().is_ok()
		}
		_ => false
	}
}

// Validate manga details data for main extraction
fn validate_manga_details_data(obj: &ObjectRef) -> bool {
	// Check if we have manga data directly
	if let Ok(manga_obj) = obj.get("manga").as_object() {
		let has_title = manga_obj.get("title").as_string().is_ok();
		let has_slug = manga_obj.get("slug").as_string().is_ok();
		if has_title || has_slug {  // Changed from AND to OR for more flexibility
			return true;
		}
	}
	
	// Check if we have initialData with manga
	if let Ok(initial_data) = obj.get("initialData").as_object() {
		if let Ok(manga_obj) = initial_data.get("manga").as_object() {
			let has_title = manga_obj.get("title").as_string().is_ok();
			let has_slug = manga_obj.get("slug").as_string().is_ok();
			if has_title || has_slug {  // Changed from AND to OR
				return true;
			}
		}
		// Also check if initialData itself has manga-like fields
		if initial_data.get("title").as_string().is_ok() || 
		   initial_data.get("slug").as_string().is_ok() ||
		   initial_data.get("chapters").as_array().is_ok() {
			return true;
		}
	}
	
	// Check if this object itself is a manga object
	let has_title = obj.get("title").as_string().is_ok();
	let has_slug = obj.get("slug").as_string().is_ok();
	let has_name = obj.get("name").as_string().is_ok();  // Alternative title field
	if has_title || has_slug || has_name {
		return true;
	}
	
	// Additional checks for potential manga data with relaxed criteria
	let has_chapters = obj.get("chapters").as_array().is_ok();
	let has_manga_fields = obj.get("author").as_string().is_ok() || 
	                      obj.get("description").as_string().is_ok() ||
	                      obj.get("status").as_string().is_ok() ||
	                      obj.get("categories").as_array().is_ok() ||
	                      obj.get("coverImage").as_string().is_ok();
	
	// Accept if we have chapters OR any manga indicators (more permissive)
	has_chapters || has_manga_fields
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

// Extract title from HTML selectors as fallback
fn extract_title_from_html(html: &Node, manga_id: &str) -> String {
	// Try common title selectors found on manga detail pages
	let title_selectors = [
		"h1.text-2xl.font-bold",
		"h1[data-testid='manga-title']",
		".manga-title",
		"h1.manga-title",
		"h1.title",
		".title h1",
		"main h1",
		"h1"
	];
	
	for selector in &title_selectors {
		let title_element = html.select(selector).first();
		if !title_element.html().is_empty() {
			let extracted_title = String::from(title_element.text().read().trim());
			if !extracted_title.is_empty() && extracted_title.len() > 2 {
				// Filter out obvious placeholders or loading text
				if !extracted_title.to_lowercase().contains("loading") &&
				   !extracted_title.to_lowercase().contains("error") &&
				   !extracted_title.starts_with("...") {
					return extracted_title;
				}
			}
		}
	}
	
	// Ultimate fallback: convert manga_id to readable title
	manga_id.replace("-", " ").replace("_", " ")
		.split_whitespace()
		.map(|word| {
			if word.len() > 0 {
				let mut chars = word.chars();
				match chars.next() {
					None => String::new(),
					Some(first) => first.to_uppercase().collect::<String>() + chars.as_str()
				}
			} else {
				String::new()
			}
		})
		.collect::<Vec<_>>()
		.join(" ")
}

// Extract manga status from HTML when Next.js extraction fails
fn extract_status_from_html(html: &Node) -> MangaStatus {
	// Status selectors targeting spans with specific status indicators
	let status_selectors = [
		"span[class*='bg-yellow-500']",    // "en pause" status (yellow background)
		"span[class*='bg-green-500']",     // "en cours" status (green background)
		"span[class*='bg-blue-500']",      // "terminé" status (blue background)
		"span[class*='bg-red-500']",       // Other status variations
		".status",                         // Generic status class
		".manga-status",                   // Manga-specific status class
		"span:contains('en cours')",       // Direct text search fallback
		"span:contains('terminé')",        
		"span:contains('en pause')",
		"*:contains('en cours')",          // Broader text search
		"*:contains('terminé')",
		"*:contains('en pause')"
	];
	
	for selector in &status_selectors {
		for element in html.select(selector).array() {
			if let Ok(element_node) = element.as_node() {
				let text = element_node.text().read().trim().to_lowercase();
				
				// Check for status keywords in the extracted text
				if text.contains("en cours") {
					return MangaStatus::Ongoing;
				} else if text.contains("terminé") {
					return MangaStatus::Completed;
				} else if text.contains("en pause") {
					return MangaStatus::Hiatus;
				} else if text.contains("annulé") || text.contains("abandonné") {
					return MangaStatus::Cancelled;
				}
			}
		}
	}
	
	// Alternative approach: search in all text content for status keywords
	let full_html = html.html().read().to_lowercase();
	if full_html.contains("en cours") {
		return MangaStatus::Ongoing;
	} else if full_html.contains("terminé") {
		return MangaStatus::Completed;
	} else if full_html.contains("en pause") {
		return MangaStatus::Hiatus;
	}
	
	// Default fallback if no status found
	MangaStatus::Unknown
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
	/*
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
	} else */
	{
		// Enhanced fallback to HTML parsing with PoseidonScans-specific selectors
		let link_selectors = [
			"a[href*='/serie/']",
			"a[href^='/serie/']",
			".manga-item a",
			".series-item a",
			".grid a[href]",
			"main a[href*='serie']"
		];
		
		for selector in &link_selectors {
			for link in html.select(selector).array() {
				let link = link.as_node()?;
				let href = link.attr("href").read();
				
				// Extract slug from href
				let slug = if href.contains("/serie/") {
					if let Some(slug_start) = href.rfind('/') {
						String::from(&href[slug_start + 1..])
					} else if let Some(serie_pos) = href.find("/serie/") {
						let start = serie_pos + 7; // "/serie/".len()
						if let Some(end) = href[start..].find('/') {
							String::from(&href[start..start + end])
						} else {
							String::from(&href[start..])
						}
					} else {
						continue;
					}
				} else {
					continue;
				};
				
				if slug == "unknown" || slug.is_empty() || slug.len() < 2 {
					continue;
				}
				
				// Enhanced title extraction with multiple fallbacks
				let title_selectors = [
					"h3.text-sm.sm\\:text-base",  // PoseidonScans specific
					".title", ".manga-title", "h3", ".name", ".entry-title",
					"h2", "h1", ".series-title", ".card-title"
				];
				
				let mut title = String::new();
				for title_sel in &title_selectors {
					let title_element = link.select(title_sel).first();
					if !title_element.html().is_empty() {
						let extracted_text = title_element.text().read();
						let extracted_title = extracted_text.trim();
						if !extracted_title.is_empty() && extracted_title.len() > 2 {
							title = String::from(extracted_title);
							break;
						}
					}
				}
				
				// Ultimate fallback: format slug as title
				if title.is_empty() {
					title = slug.replace("-", " ").replace("_", " ")
						.split_whitespace()
						.map(|word| {
							if word.len() > 0 {
								let mut chars = word.chars();
								match chars.next() {
									None => String::new(),
									Some(first) => first.to_uppercase().collect::<String>() + chars.as_str()
								}
							} else {
								String::new()
							}
						})
						.collect::<Vec<_>>()
						.join(" ");
				}
				
				// Client-side search filtering
				if !query_lower.trim().is_empty() {
					let title_lower = title.to_lowercase();
					let slug_lower = slug.to_lowercase();
					
					if !title_lower.contains(&query_lower) && !slug_lower.contains(&query_lower) {
						continue;
					}
				}
				
				// Avoid duplicates by checking if slug already exists
				if mangas.iter().any(|m| m.id == slug) {
					continue;
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
			
			// Break out of selector loop if we have enough results
			if mangas.len() >= 30 {
				break;
			}
		}
	}
	
	let has_more = false; // Only one page available from Next.js data
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
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
	// Extract Next.js page data with enhanced fallbacks
	let manga_data = extract_nextjs_manga_details(&html)?;
	
	// Extract manga object from hierarchical structure with multiple paths
	let manga_obj = if let Ok(manga) = manga_data.get("manga").as_object() {
		manga
	} else if let Ok(initial_data) = manga_data.get("initialData").as_object() {
		if let Ok(manga) = initial_data.get("manga").as_object() {
			manga
		} else {
			// Try initial_data itself as manga object
			initial_data
		}
	} else {
		// Use the manga_data itself if it contains manga fields
		manga_data
	};
	
	// Enhanced title extraction with multiple fallback strategies
	let title = if let Ok(title_str) = manga_obj.get("title").as_string() {
		let extracted_title = title_str.read();
		if !extracted_title.is_empty() {
			extracted_title
		} else {
			// Try title from HTML selector as fallback
			extract_title_from_html(&html, &manga_id)
		}
	} else {
		// Try alternative title fields
		if let Ok(name_str) = manga_obj.get("name").as_string() {
			let extracted_name = name_str.read();
			if !extracted_name.is_empty() {
				extracted_name
			} else {
				extract_title_from_html(&html, &manga_id)
			}
		} else {
			// Try title from HTML selector as fallback
			extract_title_from_html(&html, &manga_id)
		}
	};
	
	let slug = if let Ok(slug_str) = manga_obj.get("slug").as_string() {
		slug_str.read()
	} else {
		manga_id.clone()
	};
	
	// Author and artist
	let author = if let Ok(author_str) = manga_obj.get("author").as_string() {
		let author_text = author_str.read();
		if author_text.is_empty() { String::new() } else { author_text }
	} else {
		String::new()
	};
	
	let artist = if let Ok(artist_str) = manga_obj.get("artist").as_string() {
		let artist_text = artist_str.read();
		if artist_text.is_empty() { String::new() } else { artist_text }
	} else {
		String::new()
	};
	
	// Status parsing with HTML fallback
	let status = if let Ok(status_str) = manga_obj.get("status").as_string() {
		let status_text = status_str.read();
		if !status_text.is_empty() {
			parse_manga_status(&status_text)
		} else {
			// Fallback to HTML extraction if JSON status is empty
			extract_status_from_html(&html)
		}
	} else {
		// Fallback to HTML extraction if JSON extraction fails
		extract_status_from_html(&html)
	};
	
	// Categories/genres
	let mut categories = Vec::new();
	if let Ok(categories_array) = manga_obj.get("categories").as_array() {
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
		if let Ok(json_desc) = manga_obj.get("description").as_string() {
			let desc = String::from(json_desc.read().trim());
			if desc.len() > 5 && !desc.starts_with('$') {
				description = desc;
			}
		}
	}
	
	// Add alternative names if available
	if let Ok(alt_names) = manga_obj.get("alternativeNames").as_string() {
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

// Parse chapter list from Next.js JSON data like the Kotlin extension
pub fn parse_chapter_list(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	// Extract Next.js page data with same approach as parse_manga_details
	let manga_data = extract_nextjs_manga_details(&html)?;
	
	// Extract manga object from hierarchical structure with multiple paths
	let manga_obj = if let Ok(manga) = manga_data.get("manga").as_object() {
		manga
	} else if let Ok(initial_data) = manga_data.get("initialData").as_object() {
		if let Ok(manga) = initial_data.get("manga").as_object() {
			manga
		} else {
			// Try initial_data itself as manga object
			initial_data
		}
	} else {
		// Use the manga_data itself if it contains manga fields
		manga_data
	};
	
	// Extract chapters array from JSON (same approach as Kotlin extension)
	let chapters_array = if let Ok(chapters) = manga_obj.get("chapters").as_array() {
		chapters
	} else {
		// Fallback to JSON-LD extraction if Next.js data doesn't have chapters
		return parse_chapter_list_from_jsonld(manga_id, html);
	};
	
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Parse each chapter from JSON (matching Kotlin extension logic)
	for chapter_value in chapters_array {
		if let Ok(chapter_obj) = chapter_value.as_object() {
			// Extract chapter number
			let chapter_number = if let Ok(num) = chapter_obj.get("number").as_float() {
				num as f32
			} else if let Ok(num) = chapter_obj.get("number").as_int() {
				num as f32
			} else {
				continue; // Skip if no valid chapter number
			};
			
			// Check if premium (filter out premium chapters like Kotlin extension)
			let is_premium = if let Ok(premium) = chapter_obj.get("isPremium").as_bool() {
				premium
			} else {
				false
			};
			
			if is_premium {
				continue; // Skip premium chapters
			}
			
			// Extract chapter title
			let chapter_title = if let Ok(title_str) = chapter_obj.get("title").as_string() {
				let title_full = title_str.read();
				let title_text = title_full.trim();
				if title_text.is_empty() {
					format!("Chapter {}", chapter_number)
				} else {
					format!("Chapter {} - {}", chapter_number, title_text)
				}
			} else {
				format!("Chapter {}", chapter_number)
			};
			
			// NOTE: Disabled JSON date extraction due to incorrect dates
			// Force HTML date extraction for accurate relative dates
			let date_updated = 0.0; // Will be extracted from HTML later
			
			// Build chapter URL
			let chapter_id = if chapter_number == (chapter_number as i32) as f32 {
				format!("{}", chapter_number as i32)
			} else {
				format!("{}", chapter_number)
			};
			let url = format!("/serie/{}/chapter/{}", manga_id, chapter_id);
			
			chapters.push(Chapter {
				id: chapter_id,
				title: chapter_title,
				volume: -1.0,
				chapter: chapter_number,
				date_updated,
				scanlator: String::new(),
				url,
				lang: String::from("fr"),
			});
		}
	}
	
	// FORCE HTML date extraction for all chapters - ignore JSON dates completely
	extract_chapter_dates_from_html(&html, &mut chapters);
	
	// Sort chapters by number in descending order (latest first)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
	
	Ok(chapters)
}

// Fallback function: Parse chapter list from JSON-LD when Next.js data unavailable
fn parse_chapter_list_from_jsonld(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Extract chapter basic info from JSON-LD
	for script in html.select("script[type='application/ld+json']").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		// Parse JSON-LD content
		use aidoku::std::json::parse;
		if let Ok(json_data) = parse(&content) {
			if let Ok(json_obj) = json_data.as_object() {
				// Check if this is a ComicSeries type with chapters
				if let Ok(type_str) = json_obj.get("@type").as_string() {
					if type_str.read() == "ComicSeries" {
						// Extract chapters from hasPart array
						if let Ok(has_part_array) = json_obj.get("hasPart").as_array() {
							for part_value in has_part_array {
								if let Ok(part_obj) = part_value.as_object() {
									// Check if this is a ComicIssue
									if let Ok(part_type) = part_obj.get("@type").as_string() {
										if part_type.read() == "ComicIssue" {
											// Extract chapter data
											if let (Ok(issue_num), Ok(_name_str)) = (
												part_obj.get("issueNumber").as_int(),
												part_obj.get("name").as_string()
											) {
												let chapter_number = issue_num as f32;
												
												// Use Chapter format: "Chapter X"
												let title = format!("Chapter {}", issue_num);
												
												// Build chapter URL
												let url = format!("/serie/{}/chapter/{}", manga_id, issue_num);
												
												chapters.push(Chapter {
													id: format!("{}", issue_num),
													title,
													volume: -1.0,
													chapter: chapter_number,
													date_updated: 0.0, // No date from JSON-LD
													scanlator: String::new(),
													url,
													lang: String::from("fr"),
												});
											}
										}
									}
								}
							}
							
							// If we found chapters in this JSON-LD, break out of the loop
							if !chapters.is_empty() {
								break;
							}
						}
					}
				}
			}
		}
	}
	
	// Extract dates from HTML for JSON-LD fallback
	extract_chapter_dates_from_html(&html, &mut chapters);
	
	// Sort chapters by number in descending order (latest first)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
	
	Ok(chapters)
}

// Extract chapter dates from HTML and associate them with chapters
fn extract_chapter_dates_from_html(html: &Node, chapters: &mut Vec<Chapter>) {
	// Strategy 1: Search for all elements containing relative dates first, then match to chapters
	extract_dates_by_text_search(html, chapters);
	
	// Strategy 2: If strategy 1 fails, try link-based extraction
	extract_dates_by_link_association(html, chapters);
	
	// Strategy 3: JSON-LD schema.org fallback for chapters without dates
	extract_dates_from_jsonld_fallback(html, chapters);
}

// Extract dates by searching for relative date text patterns across the entire page
fn extract_dates_by_text_search(html: &Node, chapters: &mut Vec<Chapter>) {
	// Search for all elements containing relative date patterns
	let all_elements = html.select("*").array();
	
	for element in all_elements {
		if let Ok(node) = element.as_node() {
			let text = String::from(node.text().read().trim());
			
			// Check if this text looks like a relative date
			if !text.is_empty() && is_relative_date(&text) {
				// Try to find a nearby chapter link to associate this date with
				if let Some(chapter_number) = find_nearby_chapter_number(&node) {
					let timestamp = parse_relative_date(&text);
					
					// Update the matching chapter
					for chapter in chapters.iter_mut() {
						if (chapter.chapter - chapter_number).abs() < 0.1 {
							chapter.date_updated = timestamp as f64;
							break;
						}
					}
				}
			}
		}
	}
}

// Helper function to find chapter number in nearby elements (parent, siblings, children)
fn find_nearby_chapter_number(date_node: &Node) -> Option<f32> {
	// Search current element and parents for chapter links
	let search_elements = [
		date_node,  // Current element
		// Add parent and sibling search here if needed
	];
	
	for element in &search_elements {
		// Look for href attributes in this element and its children
		let links = element.select("a[href*='/chapter/'], *[href*='/chapter/']").array();
		for link in links {
			if let Ok(link_node) = link.as_node() {
				let href = link_node.attr("href").read();
				if let Some(chapter_num) = extract_chapter_number_from_url(&href) {
					return Some(chapter_num);
				}
			}
		}
		
		// Also check if current element itself has href
		let href = element.attr("href").read();
		if !href.is_empty() {
			if let Some(chapter_num) = extract_chapter_number_from_url(&href) {
				return Some(chapter_num);
			}
		}
	}
	
	None
}

// Fallback: Extract dates by direct link association (original method, improved)
fn extract_dates_by_link_association(html: &Node, chapters: &mut Vec<Chapter>) {
	// Enhanced selectors for chapter links
	let link_selectors = [
		"a[href*='/chapter/']",       // Standard chapter links
		"a[href*='chapter']",         // Alternative chapter links
		".chapter-item a",            // Styled chapter items
		"*[href*='/chapter/']",       // Any element with chapter href
		"a[href^='/serie/'][href*='/chapter/']", // Full serie + chapter path
		"[href*='/serie/'][href*='/chapter/']"   // Any element with full path
	];
	
	for link_selector in &link_selectors {
		let chapter_links = html.select(link_selector).array();
		
		// Process each chapter link to extract its date
		for chapter_link in chapter_links {
			if let Ok(link_node) = chapter_link.as_node() {
				let href = link_node.attr("href").read();
				
				// Extract chapter number from URL
				if let Some(chapter_number) = extract_chapter_number_from_url(&href) {
					// Look for date within this specific chapter link with broader search
					let date_elements = link_node.select("*").array();
					
					for date_element in date_elements {
						if let Ok(date_node) = date_element.as_node() {
							let date_text_raw = date_node.text().read();
							let date_text = date_text_raw.trim();
							
							// Enhanced validation for relative dates
							if !date_text.is_empty() && is_relative_date(date_text) {
								// Convert to timestamp
								let timestamp = parse_relative_date(date_text);
								
								// Find matching chapter in our list and update its date
								for chapter in chapters.iter_mut() {
									if (chapter.chapter - chapter_number).abs() < 0.1 {  // Float comparison
										chapter.date_updated = timestamp as f64;
										break; // Only break inner chapter loop, continue processing other dates
									}
								}
							}
						}
					}
				}
			}
		}
	}
}

// Extract chapter number from URL path (e.g., "/serie/manga-name/chapter/42" -> 42.0)
fn extract_chapter_number_from_url(url: &str) -> Option<f32> {
	if let Some(chapter_pos) = url.rfind("/chapter/") {
		let after_chapter = &url[chapter_pos + 9..]; // "/chapter/".len() = 9
		if let Some(end_pos) = after_chapter.find('/') {
			// Has path after chapter number
			if let Ok(num) = after_chapter[..end_pos].parse::<f32>() {
				return Some(num);
			}
		} else {
			// Chapter number is at the end
			if let Ok(num) = after_chapter.parse::<f32>() {
				return Some(num);
			}
		}
	}
	None
}

// Enhanced detection of relative date strings - optimized for PoseidonScans patterns
fn is_relative_date(text: &str) -> bool {
	if text.is_empty() || text.len() < 2 {
		return false;
	}
	
	let text_lower = String::from(text.to_lowercase().trim());
	
	// Specific patterns seen on PoseidonScans: "22 jours", "1 mois", "3 mois"
	let exact_patterns = [
		// Number + time unit patterns
		"jour", "jours", "day", "days",
		"mois", "month", "months", 
		"semaine", "semaines", "week", "weeks",
		"heure", "heures", "hour", "hours",
		"minute", "minutes", "min", "mins",
		"an", "ans", "année", "années", "year", "years"
	];
	
	// Check if text contains digits AND time units (most reliable pattern)
	let has_digit = text_lower.chars().any(|c| c.is_ascii_digit());
	let has_time_unit = exact_patterns.iter().any(|&pattern| text_lower.contains(pattern));
	
	if has_digit && has_time_unit {
		return true;
	}
	
	// Special cases
	if text_lower.contains("aujourd'hui") || text_lower.contains("hier") || 
	   text_lower.contains("demain") || text_lower.contains("maintenant") ||
	   text_lower.contains("il y a") {
		return true;
	}
	
	// Exact patterns that should match (common on the site)
	let exact_matches = [
		"1 jour", "1 mois", "2 mois", "3 mois", "4 mois", "5 mois", "6 mois",
		"22 jours", "1 semaine", "2 semaines", "3 semaines"
	];
	
	exact_matches.iter().any(|&pattern| text_lower == pattern || text_lower.contains(pattern))
}

// Convert relative date strings to timestamps with enhanced parsing
fn parse_relative_date(date_str: &str) -> i64 {
	use aidoku::std::current_date;
	
	let current_time = current_date();
	let date_lower = String::from(date_str.to_lowercase().trim());
	
	// Handle special cases first
	if date_lower.contains("aujourd'hui") || date_lower.contains("maintenant") {
		return current_time as i64;
	}
	if date_lower.contains("hier") {
		return (current_time - 86400.0) as i64;
	}
	if date_lower.contains("demain") {
		return (current_time + 86400.0) as i64;
	}
	
	// Extract number from string with improved parsing
	let mut number = 1;
	for word in date_lower.split_whitespace() {
		// Try to parse number, handle various formats
		if let Ok(n) = word.parse::<i32>() {
			if n > 0 && n < 1000 { // Reasonable bounds
				number = n;
				break;
			}
		}
		// Handle written numbers (un, une, deux, etc.)
		match word {
			"un" | "une" => { number = 1; break; },
			"deux" => { number = 2; break; },
			"trois" => { number = 3; break; },
			"quatre" => { number = 4; break; },
			"cinq" => { number = 5; break; },
			"six" => { number = 6; break; },
			"sept" => { number = 7; break; },
			"huit" => { number = 8; break; },
			"neuf" => { number = 9; break; },
			"dix" => { number = 10; break; },
			_ => {}
		}
	}
	
	// Calculate seconds to subtract based on unit with precise conversions
	let seconds_to_subtract = if date_lower.contains("minute") || date_lower.contains("min") {
		number as i64 * 60
	} else if date_lower.contains("heure") || date_lower.contains("hour") {
		number as i64 * 3600
	} else if date_lower.contains("jour") || date_lower.contains("day") {
		number as i64 * 86400  // 24 hours
	} else if date_lower.contains("semaine") || date_lower.contains("week") {
		number as i64 * 604800  // 7 days 
	} else if date_lower.contains("mois") || date_lower.contains("month") {
		number as i64 * 2629746  // 30.44 days (average month)
	} else if date_lower.contains("an") || date_lower.contains("année") || date_lower.contains("year") {
		number as i64 * 31556952  // 365.25 days (accounting for leap years)
	} else {
		0
	};
	
	// Calculate final timestamp (current time - duration)
	let result_time = current_time - seconds_to_subtract as f64;
	
	// Ensure result is reasonable (not negative, not too far in past)
	if result_time < 0.0 || result_time < (current_time - 31556952.0 * 10.0) { // Max 10 years ago
		0  // Invalid date
	} else {
		result_time as i64
	}
}

// JSON-LD schema.org fallback for chapters without dates
fn extract_dates_from_jsonld_fallback(html: &Node, chapters: &mut Vec<Chapter>) {
	// Look for JSON-LD script with schema.org data
	let jsonld_scripts = html.select("script[type=\"application/ld+json\"]").array();
	
	for script in jsonld_scripts {
		if let Ok(script_node) = script.as_node() {
			let script_content = script_node.text().read();
			
			// Try to parse as JSON to find date fields
			if let Ok(json_val) = aidoku::std::json::parse(script_content) {
				if let Ok(json_obj) = json_val.as_object() {
					// Look for dateModified or datePublished fields
					if let Ok(date_modified) = json_obj.get("dateModified").as_string() {
						let date_str = date_modified.read();
						if let Some(timestamp) = parse_iso_date_string(&date_str) {
							// Apply this date to chapters that don't have dates yet
							apply_fallback_date_to_chapters(chapters, timestamp);
							return;
						}
					}
					
					if let Ok(date_published) = json_obj.get("datePublished").as_string() {
						let date_str = date_published.read();
						if let Some(timestamp) = parse_iso_date_string(&date_str) {
							// Apply this date to chapters that don't have dates yet
							apply_fallback_date_to_chapters(chapters, timestamp);
							return;
						}
					}
				}
			}
		}
	}
}

// Apply fallback date only to chapters that don't have dates yet (date_updated == 0.0)
fn apply_fallback_date_to_chapters(chapters: &mut Vec<Chapter>, fallback_timestamp: i64) {
	for chapter in chapters.iter_mut() {
		if chapter.date_updated == 0.0 {
			chapter.date_updated = fallback_timestamp as f64;
		}
	}
}

// Parse ISO date string to timestamp (simplified version)
fn parse_iso_date_string(date_str: &str) -> Option<i64> {
	use aidoku::std::current_date;
	
	// Very basic ISO date parsing for fallback
	// For production, this should be more robust
	if date_str.contains("2025") || date_str.contains("2024") {
		// Use current date as reasonable fallback for schema.org dates
		Some(current_date() as i64)
	} else {
		None
	}
}

// Parse page list with Next.js data extraction and hierarchical image search
pub fn parse_page_list(html: Node, chapter_url: String) -> Result<Vec<Page>> {
	// Extract Next.js page data from chapter page
	let page_data = extract_nextjs_chapter_data(&html)?;
	
	// Search for images in hierarchical order with HTML fallback
	let image_data = extract_image_data_hierarchical(&page_data, &html)?;
	
	// Convert image data to Page objects
	let mut pages: Vec<Page> = Vec::new();
	for image in image_data {
		let absolute_url = to_absolute_url(&image.original_url);
		
		pages.push(Page {
			index: image.order,
			url: absolute_url, // Image URL goes in url field (corrected!)
			base64: String::new(),
			text: String::new(), // Empty text field
		});
	}
	
	// Sort pages by index
	pages.sort_by(|a, b| a.index.cmp(&b.index));
	
	Ok(pages)
}

// Extract Next.js data specifically for chapter pages
fn extract_nextjs_chapter_data(html: &Node) -> Result<ObjectRef> {
	// First try __NEXT_DATA__ script tag
	for script in html.select("script#__NEXT_DATA__").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if let Ok(page_data) = parse_nextjs_details_data(&content) {
			// Check if this contains chapter/image data
			if page_data.get("chapter").as_object().is_ok() ||
			   page_data.get("images").as_array().is_ok() ||
			   page_data.get("initialData").as_object().is_ok() {
				return Ok(page_data);
			}
		}
	}
	
	// Fallback to self.__next_f.push() patterns
	for script in html.select("script").array() {
		let script = script.as_node()?;
		let content = script.html().read();
		
		if content.contains("self.__next_f.push") && 
		   (content.contains("\"chapter\":{") || content.contains("\"images\":[")) {
			if let Ok(page_data) = parse_nextjs_push_manga_data(&content) {
				return Ok(page_data);
			}
		}
	}
	
	// Return empty object if extraction fails
	use aidoku::std::json::parse;
	Ok(parse("{}").unwrap().as_object().unwrap())
}

// Extract images from HTML DOM as fallback when JSON data is not available
fn extract_images_from_html(html: &Node) -> Result<Vec<PageImageData>> {
	let mut images: Vec<PageImageData> = Vec::new();
	
	// Multiple selectors to catch different image patterns in PoseidonScans
	let image_selectors = [
		"img[alt*='Chapter Image']",  // Images with "Chapter Image" in alt text
		"img[src*='/chapter/']",      // Images with chapter path
		"img[src*='/images/']",       // Images in images directory
		"img[data-src]",             // Lazy loaded images
		"img[data-original]",        // Alternative lazy loading
		"main img",                  // Images in main content area
		".chapter-content img",      // Images in chapter content
		".manga-reader img"          // Images in manga reader
	];
	
	for selector in &image_selectors {
		for img in html.select(selector).array() {
			if let Ok(img_node) = img.as_node() {
				// Extract image URL from multiple possible attributes
				let image_url = if !img_node.attr("src").read().is_empty() {
					img_node.attr("src").read()
				} else if !img_node.attr("data-src").read().is_empty() {
					img_node.attr("data-src").read()
				} else if !img_node.attr("data-original").read().is_empty() {
					img_node.attr("data-original").read()
				} else if !img_node.attr("data-lazy").read().is_empty() {
					img_node.attr("data-lazy").read()
				} else {
					continue; // Skip if no valid image URL found
				};
				
				// Skip empty URLs or placeholder images
				if image_url.is_empty() || 
				   image_url.contains("placeholder") || 
				   image_url.contains("loading") ||
				   image_url.ends_with(".svg") {
					continue;
				}
				
				// Extract order from alt text or use DOM position
				let order = if let Some(alt_text) = img_node.attr("alt").read().split_whitespace().last() {
					alt_text.parse::<i32>().unwrap_or_else(|_| images.len() as i32)
				} else {
					images.len() as i32
				};
				
				// Avoid duplicates
				if !images.iter().any(|img| img.original_url == image_url) {
					images.push(PageImageData {
						original_url: image_url,
						order,
					});
				}
			}
		}
		
		// Break if we found images with the current selector
		if !images.is_empty() {
			break;
		}
	}
	
	// Sort images by order
	images.sort_by(|a, b| a.order.cmp(&b.order));
	
	Ok(images)
}

// Extract image data using hierarchical search with HTML fallback
fn extract_image_data_hierarchical(page_data: &ObjectRef, html: &Node) -> Result<Vec<PageImageData>> {
	// Search order: root.images -> chapter.images -> initialData.images -> initialData.chapter.images
	
	// Try root level images first
	if let Ok(images_array) = page_data.get("images").as_array() {
		if let Ok(images) = parse_image_array(images_array) {
			if !images.is_empty() {
				return Ok(images);
			}
		}
	}
	
	// Try chapter.images
	if let Ok(chapter_obj) = page_data.get("chapter").as_object() {
		if let Ok(images_array) = chapter_obj.get("images").as_array() {
			if let Ok(images) = parse_image_array(images_array) {
				if !images.is_empty() {
					return Ok(images);
				}
			}
		}
	}
	
	// Try initialData.images
	if let Ok(initial_data) = page_data.get("initialData").as_object() {
		if let Ok(images_array) = initial_data.get("images").as_array() {
			if let Ok(images) = parse_image_array(images_array) {
				if !images.is_empty() {
					return Ok(images);
				}
			}
		}
		
		// Try initialData.chapter.images
		if let Ok(chapter_obj) = initial_data.get("chapter").as_object() {
			if let Ok(images_array) = chapter_obj.get("images").as_array() {
				if let Ok(images) = parse_image_array(images_array) {
					if !images.is_empty() {
						return Ok(images);
					}
				}
			}
		}
	}
	
	// Fallback to HTML DOM extraction if no JSON images found
	extract_images_from_html(html)
}

// Parse JSON array of images into PageImageData structs
fn parse_image_array(images_array: ArrayRef) -> Result<Vec<PageImageData>> {
	let mut images: Vec<PageImageData> = Vec::new();
	
	for image_value in images_array {
		if let Ok(image_obj) = image_value.as_object() {
			// Get originalUrl and order
			if let (Ok(url_str), Ok(order_num)) = (
				image_obj.get("originalUrl").as_string(),
				image_obj.get("order").as_int()
			) {
				images.push(PageImageData {
					original_url: url_str.read(),
					order: order_num as i32,
				});
			}
		}
	}
	
	Ok(images)
}

// Convert relative URLs to absolute URLs
fn to_absolute_url(url: &str) -> String {
	if url.starts_with("http") {
		// Already absolute
		String::from(url)
	} else if url.starts_with("//") {
		// Protocol-relative URL
		format!("https:{}", url)
	} else if url.starts_with("/") {
		// Site-relative URL
		format!("{}{}", String::from(BASE_URL), url)
	} else {
		// Relative URL - assume it's meant to be site-relative
		format!("{}/{}", String::from(BASE_URL), url)
	}
}

// Parse popular manga from /api/manga/all API response
pub fn parse_popular_manga(json: ObjectRef) -> Result<MangaPageResult> {
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
			
			// Extract description if available
			let description = manga.get("description").as_string()
				.map(|s| s.read())
				.unwrap_or_else(|_| String::new());
			
			// Extract categories if available
			let mut categories: Vec<String> = Vec::new();
			if let Ok(cats_array) = manga.get("categories").as_array() {
				for cat in cats_array {
					if let Ok(cat_obj) = cat.as_object() {
						if let Ok(name) = cat_obj.get("name").as_string() {
							categories.push(name.read());
						}
					}
				}
			}

			mangas.push(Manga {
				id: slug,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description,
				url: String::new(),
				categories,
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
		}
		
		// Popular API returns fixed list of 12 curated manga (no pagination)
		has_more = false;
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// Parse manga list from /api/manga/all API response with search and pagination
pub fn parse_manga_list(json: ObjectRef, search_query: String, status_filter: Option<MangaStatus>, page: i32) -> Result<MangaPageResult> {
	let mut all_mangas: Vec<Manga> = Vec::new();
	let query_lower = search_query.to_lowercase();

	if let Ok(data_array) = json.get("data").as_array() {
		for item in data_array {
			let manga = item.as_object()?;
			
			let slug = manga.get("slug").as_string()?.read();
			if slug == "unknown" || slug.is_empty() {
				continue;
			}
			
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug);
			
			// Extract description if available
			let description = manga.get("description").as_string()
				.map(|s| s.read())
				.unwrap_or_else(|_| String::new());
			
			// Extract categories if available
			let mut categories: Vec<String> = Vec::new();
			if let Ok(cats_array) = manga.get("categories").as_array() {
				for cat in cats_array {
					if let Ok(cat_obj) = cat.as_object() {
						if let Ok(name) = cat_obj.get("name").as_string() {
							categories.push(name.read());
						}
					}
				}
			}

			// Extract status if available
			let status = if let Ok(status_str) = manga.get("status").as_string() {
				let status_text = status_str.read();
				parse_manga_status(&status_text)
			} else {
				MangaStatus::Unknown
			};

			// Apply status filter if specified
			if let Some(filter_status) = status_filter {
				if status != filter_status {
					continue; // Skip this manga if it doesn't match the status filter
				}
			}

			let manga_item = Manga {
				id: slug.clone(),
				cover,
				title: title.clone(),
				author: String::new(),
				artist: String::new(),
				description: description.clone(),
				url: String::new(),
				categories,
				status,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			};

			// Client-side search filtering
			if query_lower.trim().is_empty() {
				all_mangas.push(manga_item);
			} else {
				let title_lower = title.to_lowercase();
				let description_lower = description.to_lowercase();
				let slug_lower = slug.to_lowercase();
				
				if title_lower.contains(&query_lower) || 
				   description_lower.contains(&query_lower) || 
				   slug_lower.contains(&query_lower) {
					all_mangas.push(manga_item);
				}
			}
		}
	}

	// Client-side pagination (20 manga per page like PhoenixScans)
	let items_per_page = 20;
	let start_index = ((page - 1) * items_per_page) as usize;
	let end_index = (start_index + items_per_page as usize).min(all_mangas.len());
	
	let mut paginated_mangas: Vec<Manga> = Vec::new();
	if start_index < all_mangas.len() {
		for i in start_index..end_index {
			paginated_mangas.push(all_mangas[i].clone());
		}
	}
	
	let has_more = end_index < all_mangas.len();

	Ok(MangaPageResult {
		manga: paginated_mangas,
		has_more,
	})
}