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

// Page data root structure (equivalent to PageDataRoot in Kotlin)  
struct PageDataRoot {
	images: Option<Vec<PageImageData>>,
	chapter: Option<PageDataChapter>,
	initial_data: Option<PageDataInitialData>,
}

// Page data chapter structure
struct PageDataChapter {
	images: Option<Vec<PageImageData>>,
}

// Page data initial data structure
struct PageDataInitialData {
	images: Option<Vec<PageImageData>>,
	chapter: Option<PageDataChapter>,
}

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
	
	// Only return Ok if we found some manga, otherwise return Err to trigger HTML fallback
	if mangas.is_empty() {
		use aidoku::std::String;
		Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented
		})
	} else {
		Ok(mangas)
	}
}

// Parse __NEXT_DATA__ JSON content for series data
fn parse_nextjs_series_data(content: &str) -> Result<Vec<Manga>> {
	use aidoku::std::json::parse;
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Parse the root JSON object
	if let Ok(root_json) = parse(content) {
		if let Ok(root_obj) = root_json.as_object() {
			// Try props.pageProps first (most common structure)
			if let Ok(props) = root_obj.get("props").as_object() {
				if let Ok(page_props) = props.get("pageProps").as_object() {
					// Look for manga arrays
					if let Ok(mangas_array) = page_props.get("mangas").as_array() {
						mangas.extend(parse_manga_array(mangas_array)?);
					} else if let Ok(series_array) = page_props.get("series").as_array() {
						mangas.extend(parse_manga_array(series_array)?);
					} else if let Ok(initial_data) = page_props.get("initialData").as_object() {
						if let Ok(mangas_array) = initial_data.get("mangas").as_array() {
							mangas.extend(parse_manga_array(mangas_array)?);
						} else if let Ok(series_array) = initial_data.get("series").as_array() {
							mangas.extend(parse_manga_array(series_array)?);
						}
					}
				}
			}
			
			// Try root level arrays (alternative structure)
			if mangas.is_empty() {
				if let Ok(mangas_array) = root_obj.get("mangas").as_array() {
					mangas.extend(parse_manga_array(mangas_array)?);
				} else if let Ok(series_array) = root_obj.get("series").as_array() {
					mangas.extend(parse_manga_array(series_array)?);
				} else if let Ok(initial_data) = root_obj.get("initialData").as_object() {
					if let Ok(mangas_array) = initial_data.get("mangas").as_array() {
						mangas.extend(parse_manga_array(mangas_array)?);
					} else if let Ok(series_array) = initial_data.get("series").as_array() {
						mangas.extend(parse_manga_array(series_array)?);
					}
				}
			}
		}
	}
	
	Ok(mangas)
}

// Parse JSON array of manga objects into Vec<Manga>
fn parse_manga_array(mangas_array: ArrayRef) -> Result<Vec<Manga>> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	for manga_value in mangas_array {
		if let Ok(manga_obj) = manga_value.as_object() {
			// Extract required fields: slug and title
			if let (Ok(slug_str), Ok(title_str)) = (
				manga_obj.get("slug").as_string(),
				manga_obj.get("title").as_string()
			) {
				let slug = slug_str.read();
				let title = title_str.read();
				
				// Skip invalid entries
				if slug.is_empty() || slug == "unknown" || title.is_empty() {
					continue;
				}
				
				// Build cover image URL
				let cover = if let Ok(cover_str) = manga_obj.get("coverImage").as_string() {
					let cover_path = cover_str.read();
					if cover_path.starts_with("http") {
						cover_path
					} else {
						format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
					}
				} else {
					format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
				};
				
				// Extract optional fields
				let author = if let Ok(author_str) = manga_obj.get("author").as_string() {
					author_str.read()
				} else {
					String::new()
				};
				
				let description = if let Ok(desc_str) = manga_obj.get("description").as_string() {
					let desc = desc_str.read();
					if desc.len() > 10 && !desc.starts_with('$') {
						desc
					} else {
						String::new()
					}
				} else {
					String::new()
				};
				
				// Build manga URL
				let url = format!("{}/serie/{}", String::from(BASE_URL), slug);
				
				// Create manga object
				mangas.push(Manga {
					id: slug,
					cover,
					title,
					author,
					artist: String::new(),
					description,
					url,
					categories: Vec::new(),
					status: MangaStatus::Unknown,
					nsfw: MangaContentRating::Safe,
					viewer: MangaViewer::Scroll
				});
			}
		}
	}
	
	Ok(mangas)
}

// Parse self.__next_f.push() content for manga data  
fn parse_nextjs_push_data(content: &str) -> Result<Vec<Manga>> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Find self.__next_f.push() patterns with manga array data
	let patterns = ["\"mangas\":[", "\"series\":[", "\"initialData\":{"];
	
	// Look for self.__next_f.push([1, "..."]) patterns
	let mut start_pos = 0;
	while let Some(push_start) = content[start_pos..].find("self.__next_f.push([1,") {
		let actual_start = start_pos + push_start;
		start_pos = actual_start + 1;
		
		// Find the quoted string after [1,
		if let Some(quote_start) = content[actual_start..].find('\"') {
			let string_start = actual_start + quote_start + 1;
			
			// Find the closing quote and bracket
			if let Some(quote_end) = find_closing_quote(&content[string_start..]) {
				let string_end = string_start + quote_end;
				let escaped_json = &content[string_start..string_end];
				
				// Unescape the JSON string
				let unescaped_json = unescape_json_string(escaped_json);
				
				// Try to find manga array patterns in unescaped JSON
				for pattern in &patterns {
					if let Some(pattern_pos) = unescaped_json.find(pattern) {
						if *pattern == "\"initialData\":{" {
							// Handle initialData object that might contain arrays
							if let Some(obj_start) = find_json_object_start(&unescaped_json, pattern_pos) {
								if let Some(json_obj) = extract_json_object(&unescaped_json, obj_start) {
									if let Ok(parsed_data) = extract_mangas_from_initialdata(&json_obj) {
										mangas.extend(parsed_data);
										if mangas.len() >= 50 {
											return Ok(mangas);
										}
									}
								}
							}
						} else {
							// Handle direct manga/series arrays
							if let Some(array_start) = find_json_array_start(&unescaped_json, pattern_pos) {
								if let Some(json_array) = extract_json_array(&unescaped_json, array_start) {
									if let Ok(parsed_mangas) = parse_json_manga_array(&json_array) {
										mangas.extend(parsed_mangas);
										if mangas.len() >= 50 {
											return Ok(mangas);
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
	
	Ok(mangas)
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
			return parse_manga_array(array_ref);
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
				return parse_manga_array(mangas_array);
			} else if let Ok(series_array) = obj_ref.get("series").as_array() {
				return parse_manga_array(series_array);
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
	
	// Status parsing
	let status = if let Ok(status_str) = manga_obj.get("status").as_string() {
		parse_manga_status(&status_str.read())
	} else {
		MangaStatus::Unknown
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

// Parse chapter list from JSON-LD structured data and HTML for dates
pub fn parse_chapter_list(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// First extract chapter basic info from JSON-LD
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
												
												// Use PhenixScans format: "Chapter X"
												let title = format!("Chapter {}", issue_num);
												
												// Build chapter URL
												let url = format!("/serie/{}/chapter/{}", manga_id, issue_num);
												
												chapters.push(Chapter {
													id: format!("{}", issue_num),
													title,
													volume: -1.0,
													chapter: chapter_number,
													date_updated: 0.0, // Will be updated from HTML
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
	
	// Extract dates from HTML chapter links with improved selectors
	for link in html.select("a[href*='/chapter/']").array() {
		let link = link.as_node()?;
		let href = link.attr("href").read();
		
		// Extract chapter number from URL with enhanced parsing
		if let Some(chapter_pos) = href.rfind("/chapter/") {
			let chapter_str = &href[chapter_pos + 9..]; // "/chapter/".len() = 9
			// Handle chapter numbers with potential trailing paths
			let chapter_num_str = if let Some(next_slash) = chapter_str.find('/') {
				&chapter_str[..next_slash]
			} else {
				chapter_str
			};
			
			if let Ok(chapter_num) = chapter_num_str.parse::<i32>() {
				// Improved date extraction with better selectors
				let mut date_found = false;
				
				// Try multiple selector strategies based on actual site structure
				let selector_strategies = [
					"div:last-child",           // Last div child of the link
					"span:last-child",          // Last span child of the link  
					"div",                      // Any div within the link
					"span",                     // Any span within the link
					".text-gray-400",           // Original gray text selector
					"[class*='text-']",         // Original text class selector
					"time",                     // Time elements if present
					"small"                     // Small text elements
				];
				
				for selector in &selector_strategies {
					let date_elements = link.select(selector);
					for date_elem in date_elements.array() {
						let date_elem = date_elem.as_node()?;
						let date_text_full = date_elem.text().read();
						let date_text = String::from(date_text_full.trim());
						
						// Enhanced relative date detection
						if is_relative_date(&date_text) {
							// Convert relative date to timestamp
							let timestamp = parse_relative_date(&date_text);
							
							// Update the corresponding chapter
							for chapter in &mut chapters {
								if chapter.chapter == chapter_num as f32 {
									chapter.date_updated = timestamp as f64;
									date_found = true;
									break;
								}
							}
							break;
						}
					}
					if date_found {
						break;
					}
				}
			}
		}
	}
	
	// Intelligent fallback: if any chapters still have date_updated = 0.0, set to current date
	use aidoku::std::current_date;
	let fallback_date = current_date();
	for chapter in &mut chapters {
		if chapter.date_updated == 0.0 {
			chapter.date_updated = fallback_date;
		}
	}
	
	// Sort chapters by number in descending order (latest first)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
	
	Ok(chapters)
}

// Enhanced detection of relative date strings
fn is_relative_date(text: &str) -> bool {
	if text.is_empty() || text.len() < 3 {
		return false;
	}
	
	let text_lower = text.to_lowercase();
	
	// Check for French relative time patterns
	text_lower.contains("minute") || text_lower.contains("min") ||
	text_lower.contains("heure") || text_lower.contains("hr") ||
	text_lower.contains("jour") || text_lower.contains("day") ||
	text_lower.contains("semaine") || text_lower.contains("week") ||
	text_lower.contains("mois") || text_lower.contains("month") ||
	text_lower.contains("an") || text_lower.contains("année") ||
	text_lower.contains("aujourd'hui") || text_lower.contains("hier") ||
	text_lower.contains("demain") || text_lower.contains("maintenant") ||
	// Additional patterns seen on the site
	text_lower.contains("il y a") ||
	// Numeric patterns with time units (e.g., "22 jours", "1 mois")
	(text_lower.chars().any(|c| c.is_ascii_digit()) && 
	 (text_lower.contains("jour") || text_lower.contains("mois") || 
	  text_lower.contains("heure") || text_lower.contains("semaine")))
}

// Convert relative date strings to timestamps with enhanced parsing
fn parse_relative_date(date_str: &str) -> i64 {
	use aidoku::std::current_date;
	
	let current_time = current_date();
	let date_lower = date_str.to_lowercase();
	
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
			number = n;
			break;
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
	
	// Calculate seconds to subtract based on unit with more accurate conversions
	let seconds_to_subtract = if date_lower.contains("minute") || date_lower.contains("min") {
		number * 60
	} else if date_lower.contains("heure") || date_lower.contains("hr") {
		number * 3600
	} else if date_lower.contains("jour") || date_lower.contains("day") {
		number * 86400
	} else if date_lower.contains("semaine") || date_lower.contains("week") {
		number * 604800 // 7 days
	} else if date_lower.contains("mois") || date_lower.contains("month") {
		number * 2629746 // 30.44 days (more accurate month)
	} else if date_lower.contains("an") || date_lower.contains("année") || date_lower.contains("year") {
		number * 31556952 // 365.25 days (accounting for leap years)
	} else {
		0
	};
	
	// Return timestamp, ensuring it's not negative
	let result_time = current_time - seconds_to_subtract as f64;
	if result_time < 0.0 {
		current_time as i64
	} else {
		result_time as i64
	}
}

// Parse page list with Next.js data extraction and hierarchical image search
pub fn parse_page_list(html: Node) -> Result<Vec<Page>> {
	// Extract Next.js page data from chapter page
	let page_data = extract_nextjs_chapter_data(&html)?;
	
	// Search for images in hierarchical order
	let image_data = extract_image_data_hierarchical(&page_data)?;
	
	// Get chapter page URL for referer
	let chapter_url = get_chapter_url_from_html(&html);
	
	// Convert image data to Page objects
	let mut pages: Vec<Page> = Vec::new();
	for image in image_data {
		let absolute_url = to_absolute_url(&image.original_url);
		
		pages.push(Page {
			index: image.order,
			url: chapter_url.clone(),
			base64: String::new(),
			text: absolute_url, // Image URL goes in text field
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

// Extract image data using hierarchical search
fn extract_image_data_hierarchical(page_data: &ObjectRef) -> Result<Vec<PageImageData>> {
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
	
	// Return empty vector if no images found
	Ok(Vec::new())
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

// Get chapter URL from HTML for referer header
fn get_chapter_url_from_html(_html: &Node) -> String {
	// TODO: Extract actual URL from HTML or use current page URL
	// For now, return base URL as fallback
	String::from(BASE_URL)
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
pub fn parse_manga_list(json: ObjectRef, search_query: String, page: i32) -> Result<MangaPageResult> {
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

			let manga_item = Manga {
				id: slug.clone(),
				cover,
				title: title.clone(),
				author: String::new(),
				artist: String::new(),
				description: description.clone(),
				url: String::new(),
				categories,
				status: MangaStatus::Unknown,
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