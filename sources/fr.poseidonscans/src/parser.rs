use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::html::Document,
	serde::Deserialize,
};
use core::cmp::Ordering;
use serde_json;
use crate::BASE_URL;

// Serde structures for Poseidon Scans API responses

#[derive(Deserialize, Debug)]
pub struct ApiResponse<T> {
	pub data: Vec<T>,
}

#[derive(Deserialize, Debug)]
pub struct MangaItem {
	pub slug: String,
	pub title: String,
	#[serde(default)]
	pub author: Option<String>,
	#[serde(default)]  
	pub artist: Option<String>,
	#[serde(default)]
	pub status: Option<String>,
	#[serde(default)]
	pub description: Option<String>,
	#[serde(default)]
	pub categories: Option<Vec<CategoryItem>>,
}

#[derive(Deserialize, Debug)]
pub struct CategoryItem {
	pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct LatestChapterResponse {
	pub data: Vec<LatestChapterItem>,
	#[serde(default)]
	pub pagination: Option<PaginationInfo>,
}

#[derive(Deserialize, Debug)]
pub struct LatestChapterItem {
	pub slug: String,
	pub title: String,
	#[serde(rename = "lastChapter", default)]
	pub last_chapter: Option<ChapterInfo>,
}

#[derive(Deserialize, Debug)]
pub struct ChapterInfo {
	#[serde(rename = "chapterNumber", default)]
	pub chapter_number: Option<f32>,
	#[serde(rename = "releaseDate", default)]
	pub release_date: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct PaginationInfo {
	#[serde(rename = "hasMore", default)]
	pub has_more: Option<bool>,
}

// Implementation methods for converting API responses to Aidoku types

impl MangaItem {
	pub fn to_manga(&self) -> Manga {
		let key = self.slug.clone();
		let title = self.title.clone();
		let cover = format!("{}/api/covers/{}.webp", BASE_URL, self.slug);
		
		let authors = self.author.as_ref()
			.map(|a| vec![a.clone()])
			.filter(|a| !a.is_empty());
			
		let artists = self.artist.as_ref()
			.map(|a| vec![a.clone()])
			.filter(|a| !a.is_empty());
		
		let tags = self.categories.as_ref()
			.map(|cats| cats.iter().map(|c| c.name.clone()).collect::<Vec<_>>())
			.filter(|t| !t.is_empty());
		
		let status = self.status.as_ref()
			.map(|s| parse_manga_status(s))
			.unwrap_or(MangaStatus::Unknown);
		
		let description = self.description.clone()
			.filter(|d| !d.is_empty() && d != "Aucune description.");

		Manga {
			key: key.clone(),
			title,
			cover: Some(cover),
			authors,
			artists,
			description,
			url: Some(format!("{}/serie/{}", BASE_URL, key)),
			tags,
			status,
			content_rating: ContentRating::Safe,
			viewer: Viewer::RightToLeft,
			chapters: None,
			next_update_time: None,
			update_strategy: UpdateStrategy::Never,
		}
	}
}

impl LatestChapterItem {
	pub fn to_manga(&self) -> Manga {
		let key = self.slug.clone();
		let title = self.title.clone();
		let cover = format!("{}/api/covers/{}.webp", BASE_URL, self.slug);

		Manga {
			key: key.clone(),
			title,
			cover: Some(cover),
			authors: None,
			artists: None,
			description: None,
			url: Some(format!("{}/serie/{}", BASE_URL, key)),
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::RightToLeft,
			chapters: None,
			next_update_time: None,
			update_strategy: UpdateStrategy::Never,
		}
	}
}

// Parse functions for different API endpoints

pub fn parse_manga_list(response: String, search_query: String, status_filter: Option<MangaStatus>, page: i32) -> Result<MangaPageResult> {
	let api_response: ApiResponse<MangaItem> = serde_json::from_str(&response)
		.map_err(|_| aidoku::AidokuError::JsonParseError)?;

	let mut all_mangas: Vec<Manga> = Vec::new();
	let query_lower = search_query.to_lowercase();

	for item in api_response.data {
		let manga = item.to_manga();
		
		// Apply search filter
		if !search_query.is_empty() && !manga.title.to_lowercase().contains(&query_lower) {
			continue;
		}
		
		// Apply status filter  
		if let Some(filter_status) = status_filter {
			if manga.status != filter_status {
				continue;
			}
		}
		
		all_mangas.push(manga);
	}

	// Client-side pagination (20 items per page)
	let page_size = 20;
	let start_index = ((page - 1) * page_size) as usize;
	let end_index = (start_index + page_size as usize).min(all_mangas.len());
	
	let paginated_mangas = if start_index < all_mangas.len() {
		all_mangas[start_index..end_index].to_vec()
	} else {
		Vec::new()
	};
	
	let has_next_page = end_index < all_mangas.len();

	Ok(MangaPageResult {
		entries: paginated_mangas,
		has_next_page,
	})
}

pub fn parse_latest_manga(response: String) -> Result<MangaPageResult> {
	let api_response: LatestChapterResponse = serde_json::from_str(&response)
		.map_err(|_| aidoku::AidokuError::JsonParseError)?;

	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in api_response.data {
		let manga = item.to_manga();
		mangas.push(manga);
	}
	
	let has_next_page = api_response.pagination
		.and_then(|p| p.has_more)
		.unwrap_or(false);

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page,
	})
}

pub fn parse_popular_manga(response: String) -> Result<MangaPageResult> {
	let api_response: ApiResponse<MangaItem> = serde_json::from_str(&response)
		.map_err(|_| aidoku::AidokuError::JsonParseError)?;

	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in api_response.data {
		let manga = item.to_manga();
		mangas.push(manga);
	}
	
	// Popular manga is always a fixed list with no pagination
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: false,
	})
}

// HTML parsing functions for details, chapters, and pages

pub fn parse_manga_details(manga_key: String, html: &Document) -> Result<Manga> {
	let mut title = manga_key.clone();
	let mut description = String::new();
	let authors: Option<Vec<String>> = None;
	let artists: Option<Vec<String>> = None;
	let mut tags: Option<Vec<String>> = None;
	let mut status = MangaStatus::Unknown;

	// Extract title from page
	if let Some(title_text) = html.select("h1").and_then(|els| els.first()).and_then(|el| el.text()) {
		if !title_text.is_empty() {
			title = title_text.trim().to_string();
		}
	}

	// Extract description
	if let Some(desc_text) = html.select("p.text-gray-300.leading-relaxed.whitespace-pre-line").and_then(|els| els.first()).and_then(|el| el.text()) {
		let desc = desc_text.trim().to_string();
		if !desc.is_empty() && desc != "Aucune description." {
			description = desc;
		}
	}

	// Extract genres/tags from HTML
	let mut genre_list: Vec<String> = Vec::new();
	if let Some(genre_elements) = html.select("a[href*='/genres/']") {
		for genre_element in genre_elements {
			if let Some(genre_text) = genre_element.text() {
				let genre = genre_text.trim().to_string();
				if !genre.is_empty() {
					genre_list.push(genre);
				}
			}
		}
	}
	if !genre_list.is_empty() {
		tags = Some(genre_list);
	}

	// Extract status from HTML
	if let Some(status_elements) = html.select(".status, .manga-status, [class*='status']") {
		for status_element in status_elements {
			if let Some(status_text) = status_element.text() {
				let status_str = status_text.trim();
				status = parse_manga_status(status_str);
				break;
			}
		}
	}

	let cover = format!("{}/api/covers/{}.webp", BASE_URL, manga_key);

	Ok(Manga {
		key: manga_key.clone(),
		title,
		cover: Some(cover),
		authors,
		artists,
		description: if description.is_empty() { None } else { Some(description) },
		url: Some(format!("{}/serie/{}", BASE_URL, manga_key)),
		tags,
		status,
		content_rating: ContentRating::Safe,
		viewer: Viewer::RightToLeft,
		chapters: None,
		next_update_time: None,
		update_strategy: UpdateStrategy::Never,
	})
}

pub fn parse_chapter_list(manga_key: String, html: &Document) -> Result<Vec<Chapter>> {
	// Use the same approach as the original implementation, adapted for modern Aidoku
	let manga_data = extract_nextjs_manga_details(&html)?;
	
	// Extract chapters array from JSON using the same logic as original
	let mut chapters = if let Some(chapters_array) = manga_data.get("chapters").and_then(|c| c.as_array()) {
		parse_chapters_from_json_array(chapters_array, &manga_key)
	} else if let Some(manga_obj) = manga_data.get("manga") {
		if let Some(chapters_array) = manga_obj.get("chapters").and_then(|c| c.as_array()) {
			parse_chapters_from_json_array(chapters_array, &manga_key)
		} else {
			// Fallback to HTML if Next.js data doesn't have chapters
			parse_chapter_list_from_html(html)?
		}
	} else if let Some(initial_data) = manga_data.get("initialData") {
		if let Some(chapters_array) = initial_data.get("chapters").and_then(|c| c.as_array()) {
			parse_chapters_from_json_array(chapters_array, &manga_key)
		} else if let Some(manga_obj) = initial_data.get("manga") {
			if let Some(chapters_array) = manga_obj.get("chapters").and_then(|c| c.as_array()) {
				parse_chapters_from_json_array(chapters_array, &manga_key)
			} else {
				// Fallback to HTML if no chapters found in nested structure
				parse_chapter_list_from_html(html)?
			}
		} else {
			// Fallback to HTML extraction
			parse_chapter_list_from_html(html)?
		}
	} else {
		// Fallback to HTML extraction if Next.js data is empty
		parse_chapter_list_from_html(html)?
	};
	
	// Extract chapter dates from HTML (like original implementation)
	extract_chapter_dates_from_html(&html, &mut chapters);
	
	// Sort chapters by number in descending order (latest first)
	chapters.sort_by(|a, b| {
		match (a.chapter_number, b.chapter_number) {
			(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
			(Some(_), None) => Ordering::Less,
			(None, Some(_)) => Ordering::Greater,
			(None, None) => Ordering::Equal,
		}
	});
	
	Ok(chapters)
}

// Simple Next.js extraction adapted from original implementation
fn extract_nextjs_manga_details(html: &Document) -> Result<serde_json::Value> {
	// First try __NEXT_DATA__ script tag (most reliable)
	if let Some(script_elements) = html.select("script#__NEXT_DATA__") {
		for script in script_elements {
			if let Some(script_content) = script.text() {
				if let Ok(root_json) = serde_json::from_str::<serde_json::Value>(&script_content) {
					// Try props.pageProps first (most common structure)
					if let Some(props) = root_json.get("props") {
						if let Some(page_props) = props.get("pageProps") {
							// Check if pageProps contains expected data structures
							if page_props.get("initialData").is_some() ||
							   page_props.get("manga").is_some() ||
							   page_props.get("chapters").is_some() {
								return Ok(page_props.clone());
							}
						}
					}
					
					// Try root level initialData (alternative structure)
					if let Some(initial_data) = root_json.get("initialData") {
						if initial_data.get("manga").is_some() ||
						   initial_data.get("chapters").is_some() {
							return Ok(initial_data.clone());
						}
					}
					
					// Try direct manga data at root level
					if root_json.get("manga").is_some() {
						return Ok(root_json);
					}
				}
			}
		}
	}
	
	// Return empty object if parsing fails - let fallback handle it
	Ok(serde_json::json!({}))
}

// Parse chapters from JSON array (adapted from original implementation)
fn parse_chapters_from_json_array(chapters_array: &Vec<serde_json::Value>, manga_key: &str) -> Vec<Chapter> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Parse each chapter from JSON (matching original logic but adapted for serde)
	for chapter_value in chapters_array {
		if let Some(chapter_obj) = chapter_value.as_object() {
			// Extract chapter number
			let chapter_number = if let Some(num) = chapter_obj.get("number") {
				if let Some(n) = num.as_f64() {
					n as f32
				} else if let Some(n) = num.as_i64() {
					n as f32
				} else {
					continue; // Skip if no valid chapter number
				}
			} else {
				continue;
			};
			
			// Check if premium but DON'T filter out completely - mark as locked instead
			// This is the key change from the original implementation
			let is_premium = chapter_obj.get("isPremium")
				.and_then(|v| v.as_bool())
				.unwrap_or(false);
			
			// Extract chapter title - simplified format: "Chapitre X"
			let chapter_title = format!("Chapitre {}", chapter_number);
			
			// Don't use JSON dates - will be extracted from HTML later for accuracy
			let date_uploaded = None;
			
			// Build chapter URL
			let chapter_id = if chapter_number == (chapter_number as i32) as f32 {
				format!("{}", chapter_number as i32)
			} else {
				format!("{}", chapter_number)
			};
			let url = format!("{}/serie/{}/chapter/{}", BASE_URL, manga_key, chapter_id);
			
			chapters.push(Chapter {
				key: chapter_id,
				title: Some(chapter_title),
				volume_number: None,
				chapter_number: Some(chapter_number),
				date_uploaded,
				scanlators: None,
				url: Some(url),
				language: Some("fr".to_string()),
				thumbnail: None,
				locked: is_premium, // Mark premium chapters as locked instead of filtering
			});
		}
	}
	
	chapters
}

// Parse complex self.__next_f.push() data structures like the original implementation
fn parse_nextjs_push_data(content: &str, manga_key: &str) -> Option<Vec<Chapter>> {
	// Look for all possible self.__next_f.push patterns - more comprehensive search
	let push_patterns = [
		"self.__next_f.push([1,",
		"self.__next_f.push([0,", 
		"self.__next_f.push([2,",
	];
	
	for push_pattern in &push_patterns {
		let mut start_pos = 0;
		while let Some(push_start) = content[start_pos..].find(push_pattern) {
			let actual_start = start_pos + push_start;
			start_pos = actual_start + 1;
			
			// Find the quoted string after [N,
			if let Some(quote_start) = content[actual_start..].find('"') {
				let string_start = actual_start + quote_start + 1;
				
				// Find the closing quote and bracket
				if let Some(quote_end) = find_closing_quote(&content[string_start..]) {
					let string_end = string_start + quote_end;
					let escaped_json = &content[string_start..string_end];
					
					// Skip very short strings that are unlikely to contain chapter data
					if escaped_json.len() < 100 {
						continue;
					}
					
					// Unescape the JSON string
					let unescaped_json = unescape_json_string(escaped_json);
					
					// Look for chapter data patterns - more comprehensive
					if unescaped_json.contains("chapters") && unescaped_json.contains("number") {
						// Try to find and parse the complete manga object
						if let Some(chapters) = extract_all_chapters_from_json(&unescaped_json, manga_key) {
							if !chapters.is_empty() {
								return Some(chapters);
							}
						}
					}
				}
			}
		}
	}
	
	None
}

// Extract all chapters from JSON string - more aggressive search
fn extract_all_chapters_from_json(json_str: &str, manga_key: &str) -> Option<Vec<Chapter>> {
	// Try multiple approaches to find the chapters array
	
	// Approach 1: Parse as complete JSON object
	if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(json_str) {
		if let Some(chapters) = extract_chapters_from_nextjs_value(&parsed, manga_key) {
			return Some(chapters);
		}
	}
	
	// Approach 2: Look for "chapters":[...] pattern directly
	if let Some(chapters_pos) = json_str.find("\"chapters\":[") {
		let after_chapters = &json_str[chapters_pos + 12..]; // 12 = len("\"chapters\":[")
		
		// Find the closing bracket for the chapters array
		if let Some(chapters_json) = extract_json_array(after_chapters) {
			let full_json = format!("{{\"chapters\":{}}}", chapters_json);
			if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&full_json) {
				if let Some(chapters) = extract_chapters_from_nextjs_value(&parsed, manga_key) {
					return Some(chapters);
				}
			}
		}
	}
	
	// Approach 3: Look for any pattern that looks like chapter data
	let mut chapters = Vec::new();
	let chapter_patterns = [
		"\"number\":", 
		"\"isPremium\":",
		"\"createdAt\":"
	];
	
	// Simple regex-like approach to find individual chapter objects
	let mut search_pos = 0;
	while let Some(obj_start) = json_str[search_pos..].find('{') {
		let absolute_start = search_pos + obj_start;
		if let Some(obj_json) = extract_json_object(&json_str, absolute_start) {
			// Check if this object looks like a chapter
			if chapter_patterns.iter().any(|pattern| obj_json.contains(pattern)) {
				if let Ok(chapter_obj) = serde_json::from_str::<serde_json::Value>(&obj_json) {
					if let Some(chapter) = parse_nextjs_chapter(&chapter_obj, manga_key) {
						chapters.push(chapter);
					}
				}
			}
		}
		search_pos = absolute_start + 1;
	}
	
	if !chapters.is_empty() {
		// Remove duplicates and sort
		chapters.dedup_by(|a, b| a.key == b.key);
		chapters.sort_by(|a, b| {
			match (a.chapter_number, b.chapter_number) {
				(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
				(Some(_), None) => Ordering::Less,
				(None, Some(_)) => Ordering::Greater,
				(None, None) => Ordering::Equal,
			}
		});
		return Some(chapters);
	}
	
	None
}

// Extract JSON array from string (find matching brackets)
fn extract_json_array(content: &str) -> Option<String> {
	if content.is_empty() || !content.starts_with('[') {
		return None;
	}
	
	let mut bracket_count = 0;
	let mut in_string = false;
	let mut escaped = false;
	
	for (i, ch) in content.char_indices() {
		if escaped {
			escaped = false;
			continue;
		}
		
		match ch {
			'\\' if in_string => escaped = true,
			'"' => in_string = !in_string,
			'[' if !in_string => bracket_count += 1,
			']' if !in_string => {
				bracket_count -= 1;
				if bracket_count == 0 {
					return Some(content[..=i].to_string());
				}
			},
			_ => {}
		}
	}
	
	None
}

// Find closing quote, handling escaped quotes
fn find_closing_quote(content: &str) -> Option<usize> {
	let mut i = 0;
	let chars: Vec<char> = content.chars().collect();
	
	while i < chars.len() {
		if chars[i] == '\\' && i + 1 < chars.len() {
			// Skip escaped character
			i += 2;
		} else if chars[i] == '"' {
			return Some(i);
		} else {
			i += 1;
		}
	}
	None
}

// Unescape JSON string (basic implementation)
fn unescape_json_string(escaped: &str) -> String {
	escaped
		.replace("\\\"", "\"")
		.replace("\\\\", "\\")
		.replace("\\n", "\n")
		.replace("\\r", "\r")
		.replace("\\t", "\t")
}

// Find the start of JSON object before a pattern position
fn find_json_object_start(content: &str, pattern_pos: usize) -> Option<usize> {
	let mut pos = pattern_pos;
	let chars: Vec<char> = content.chars().collect();
	
	// Go backwards to find the opening brace
	while pos > 0 {
		pos -= 1;
		if chars[pos] == '{' {
			// Check that this is likely the start of our object
			return Some(pos);
		}
		// Stop if we hit another closing brace (wrong object)
		if chars[pos] == '}' {
			break;
		}
	}
	None
}

// Extract JSON object from position to matching closing brace
fn extract_json_object(content: &str, start_pos: usize) -> Option<String> {
	let chars: Vec<char> = content.chars().collect();
	let mut brace_count = 0;
	let mut end_pos = start_pos;
	
	for i in start_pos..chars.len() {
		match chars[i] {
			'{' => brace_count += 1,
			'}' => {
				brace_count -= 1;
				if brace_count == 0 {
					end_pos = i + 1;
					break;
				}
			},
			_ => {}
		}
	}
	
	if brace_count == 0 && end_pos > start_pos {
		Some(content[start_pos..end_pos].to_string())
	} else {
		None
	}
}


fn extract_chapters_from_nextjs_value(data: &serde_json::Value, manga_key: &str) -> Option<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	// Try multiple paths to find chapters data - expanded like original implementation
	let possible_paths: Vec<&[&str]> = vec![
		&["props", "pageProps", "manga", "chapters"],
		&["props", "pageProps", "initialData", "manga", "chapters"],  
		&["props", "pageProps", "initialData", "chapters"],
		&["props", "pageProps", "series", "chapters"],
		&["manga", "chapters"],
		&["initialData", "manga", "chapters"],
		&["initialData", "chapters"], 
		&["series", "chapters"],
		&["chapters"],
	];

	for path in &possible_paths {
		if let Some(chapters_array) = get_nested_value(data, path) {
			if let Some(chapters_array) = chapters_array.as_array() {
				for chapter_value in chapters_array {
					if let Some(chapter) = parse_nextjs_chapter(chapter_value, manga_key) {
						chapters.push(chapter);
					}
				}
				if !chapters.is_empty() {
					break;
				}
			}
		}
	}

	// Also try direct search in the entire JSON if no structured path worked
	if chapters.is_empty() {
		if let Some(found_chapters) = search_chapters_recursively(data, manga_key) {
			chapters.extend(found_chapters);
		}
	}

	if chapters.is_empty() {
		None
	} else {
		// Sort by chapter number descending
		chapters.sort_by(|a, b| {
			match (a.chapter_number, b.chapter_number) {
				(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
				(Some(_), None) => Ordering::Less,
				(None, Some(_)) => Ordering::Greater,
				(None, None) => Ordering::Equal,
			}
		});
		Some(chapters)
	}
}

// Recursively search for chapters array in any part of the JSON
fn search_chapters_recursively(value: &serde_json::Value, manga_key: &str) -> Option<Vec<Chapter>> {
	match value {
		serde_json::Value::Object(obj) => {
			// Check if this object has a "chapters" key with array
			if let Some(serde_json::Value::Array(chapters_array)) = obj.get("chapters") {
				let mut chapters = Vec::new();
				for chapter_value in chapters_array {
					if let Some(chapter) = parse_nextjs_chapter(chapter_value, manga_key) {
						chapters.push(chapter);
					}
				}
				if !chapters.is_empty() {
					return Some(chapters);
				}
			}
			
			// Recursively search in all values of this object
			for (_, v) in obj {
				if let Some(found) = search_chapters_recursively(v, manga_key) {
					return Some(found);
				}
			}
		},
		serde_json::Value::Array(arr) => {
			// Recursively search in array elements
			for item in arr {
				if let Some(found) = search_chapters_recursively(item, manga_key) {
					return Some(found);
				}
			}
		},
		_ => {}
	}
	None
}

fn get_nested_value<'a>(value: &'a serde_json::Value, path: &[&str]) -> Option<&'a serde_json::Value> {
	let mut current = value;
	for key in path {
		current = current.get(key)?;
	}
	Some(current)
}

fn parse_nextjs_chapter(chapter_value: &serde_json::Value, manga_key: &str) -> Option<Chapter> {
	let chapter_obj = chapter_value.as_object()?;
	
	// Extract chapter number first to validate - try multiple field names
	let chapter_number = chapter_obj.get("number")
		.or_else(|| chapter_obj.get("chapterNumber"))
		.and_then(|v| {
			if let Some(n) = v.as_f64() {
				Some(n as f32)
			} else if let Some(n) = v.as_i64() {
				Some(n as f32)
			} else if let Some(s) = v.as_str() {
				s.parse::<f32>().ok()
			} else {
				None
			}
		})?; // Return None if no valid chapter number
	
	// Check if premium but DON'T filter out - mark as locked instead
	let is_premium = chapter_obj.get("isPremium")
		.and_then(|v| v.as_bool())
		.unwrap_or(false);
	
	// Extract chapter ID/slug - try multiple sources
	let chapter_id = chapter_obj.get("id")
		.and_then(|v| v.as_str())
		.map(|s| s.to_string())
		.or_else(|| {
			chapter_obj.get("slug")
				.and_then(|v| v.as_str())
				.map(|s| s.to_string())
		})
		.unwrap_or_else(|| {
			// Fallback: use chapter number as ID
			if chapter_number == (chapter_number as i32) as f32 {
				format!("{}", chapter_number as i32)
			} else {
				format!("{}", chapter_number)
			}
		});

	// Extract chapter title - simple format: "Chapitre X"
	let chapter_title = format!("Chapitre {}", chapter_number);

	// Don't use JSON dates - they will be overridden by HTML date extraction later
	// which provides more accurate relative dates like "2 heures", "1 jour"
	let date_uploaded = None;

	let url = format!("{}/serie/{}/chapter/{}", BASE_URL, manga_key, chapter_id);

	Some(Chapter {
		key: chapter_id,
		title: Some(chapter_title),
		volume_number: None,
		chapter_number: Some(chapter_number),
		date_uploaded,
		scanlators: None,
		url: Some(url),
		language: Some("fr".to_string()),
		thumbnail: None,
		locked: is_premium, // Mark premium chapters as locked instead of filtering
	})
}

fn parse_chapter_list_from_html(html: &Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut seen_chapter_ids: Vec<String> = Vec::new();

	// Updated selectors based on modern PoseidonScans structure
	let chapter_selectors = [
		"a[href*='/chapter/']",  // General chapter links
		".chapter-item a",       // Styled chapter items
		"*[href*='/chapter/']",  // Any element with chapter href
		"div a[href*='/serie/'][href*='/chapter/']", // Full serie + chapter path
	];

	for selector in &chapter_selectors {
		if let Some(chapter_elements) = html.select(selector) {
			for chapter_element in chapter_elements {
				if let Some(href_str) = chapter_element.attr("href") {
					// Extract chapter ID from URL
					if let Some(chapter_id) = extract_chapter_id_from_url(&href_str) {
						// Skip duplicates
						if seen_chapter_ids.contains(&chapter_id) {
							continue;
						}
						seen_chapter_ids.push(chapter_id.clone());

						// Extract chapter number from URL or ID first
						let chapter_number = extract_chapter_number_from_id(&chapter_id);

						// Generate clean title: "Chapitre X"
						let title = if let Some(ch_num) = chapter_number {
							format!("Chapitre {}", ch_num)
						} else {
							format!("Chapitre {}", chapter_id)
						};

						let url = if href_str.starts_with("http") {
							href_str.to_string()
						} else {
							format!("{}{}", BASE_URL, href_str)
						};

						// Use None for date_uploaded - will be filled by HTML date extraction later
						chapters.push(Chapter {
							key: chapter_id,
							title: Some(title),
							volume_number: None,
							chapter_number,
							date_uploaded: None,
							scanlators: None,
							url: Some(url),
							language: Some("fr".to_string()),
							thumbnail: None,
							locked: false,
						});
					}
				}
			}

			// Continue trying all selectors to get as many chapters as possible
		}
	}

	// Remove duplicates by key (in case multiple selectors found the same chapter)
	chapters.dedup_by(|a, b| a.key == b.key);

	// Sort chapters by chapter number (descending - newest first)
	chapters.sort_by(|a, b| {
		match (a.chapter_number, b.chapter_number) {
			(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
			(Some(_), None) => Ordering::Less,
			(None, Some(_)) => Ordering::Greater,
			(None, None) => Ordering::Equal,
		}
	});

	Ok(chapters)
}

// Extract chapter dates from HTML and associate them with chapters (ported from original implementation)
fn extract_chapter_dates_from_html(html: &Document, chapters: &mut Vec<Chapter>) {
	// Strategy 1: Search for all elements containing relative dates first, then match to chapters
	extract_dates_by_text_search(html, chapters);
	
	// Strategy 2: If strategy 1 fails, try link-based extraction
	extract_dates_by_link_association(html, chapters);
	
	// Strategy 3: JSON-LD schema.org fallback for chapters without dates
	extract_dates_from_jsonld_fallback(html, chapters);
}

// Extract dates by searching for relative date text patterns across the entire page
fn extract_dates_by_text_search(html: &Document, chapters: &mut Vec<Chapter>) {
	// Search for all elements containing relative date patterns
	if let Some(all_elements) = html.select("*") {
		for element in all_elements {
			if let Some(text) = element.text() {
				let text_trimmed = text.trim();
				
				// Check if this text looks like a relative date
				if !text_trimmed.is_empty() && is_relative_date(text_trimmed) {
					// Try to find a nearby chapter link to associate this date with
					if let Some(chapter_number) = find_nearby_chapter_number(&element) {
						let timestamp = parse_relative_date(text_trimmed);
						
						// Update the matching chapter
						for chapter in chapters.iter_mut() {
							if let Some(ch_num) = chapter.chapter_number {
								if (ch_num - chapter_number).abs() < 0.1 {
									chapter.date_uploaded = Some(timestamp);
									break;
								}
							}
						}
					}
				}
			}
		}
	}
}

// Helper function to find chapter number in nearby elements (parent, siblings, children)
fn find_nearby_chapter_number(element: &aidoku::imports::html::Element) -> Option<f32> {
	// Look for href attributes in this element and its children
	if let Some(links) = element.select("a[href*='/chapter/'], *[href*='/chapter/']") {
		for link in links {
			if let Some(href) = link.attr("href") {
				if let Some(chapter_num) = extract_chapter_number_from_url(&href) {
					return Some(chapter_num);
				}
			}
		}
	}
	
	// Also check if current element itself has href
	if let Some(href) = element.attr("href") {
		if !href.is_empty() {
			if let Some(chapter_num) = extract_chapter_number_from_url(&href) {
				return Some(chapter_num);
			}
		}
	}
	
	None
}

// Fallback: Extract dates by direct link association (original method, improved)
fn extract_dates_by_link_association(html: &Document, chapters: &mut Vec<Chapter>) {
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
		if let Some(chapter_links) = html.select(link_selector) {
			// Process each chapter link to extract its date
			for chapter_link in chapter_links {
				if let Some(href) = chapter_link.attr("href") {
					// Extract chapter number from URL
					if let Some(chapter_number) = extract_chapter_number_from_url(&href) {
						// Look for date within this specific chapter link with broader search
						if let Some(date_elements) = chapter_link.select("*") {
							for date_element in date_elements {
								if let Some(date_text) = date_element.text() {
									let date_text_trimmed = date_text.trim();
									
									// Enhanced validation for relative dates
									if !date_text_trimmed.is_empty() && is_relative_date(date_text_trimmed) {
										// Convert to timestamp
										let timestamp = parse_relative_date(date_text_trimmed);
										
										// Find matching chapter in our list and update its date
										for chapter in chapters.iter_mut() {
											if let Some(ch_num) = chapter.chapter_number {
												if (ch_num - chapter_number).abs() < 0.1 {  // Float comparison
													chapter.date_uploaded = Some(timestamp);
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
	
	let text_lower = text.to_lowercase();
	
	// Specific patterns seen on PoseidonScans: "22 jours", "1 mois", "3 mois", "2 heures"
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
		"22 jours", "1 semaine", "2 semaines", "3 semaines", "2 heures", "1 heure"
	];
	
	exact_matches.iter().any(|&pattern| text_lower == pattern || text_lower.contains(pattern))
}

// Convert relative date strings to timestamps with enhanced parsing
fn parse_relative_date(date_str: &str) -> i64 {
	use aidoku::imports::std::current_date;
	
	let current_time = current_date();
	let date_lower = date_str.to_lowercase();
	
	// Handle special cases first
	if date_lower.contains("aujourd'hui") || date_lower.contains("maintenant") {
		return current_time as i64;
	}
	if date_lower.contains("hier") {
		return (current_time as i64) - 86400;
	}
	if date_lower.contains("demain") {
		return (current_time as i64) + 86400;
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
	let current_time_i64 = current_time as i64;
	let result_time = current_time_i64 - seconds_to_subtract;
	
	// Ensure result is reasonable (not negative, not too far in past)
	let ten_years_ago = current_time_i64 - (31556952 * 10); // Max 10 years ago
	if result_time < 0 || result_time < ten_years_ago {
		0  // Invalid date
	} else {
		result_time
	}
}

// JSON-LD schema.org fallback for chapters without dates
fn extract_dates_from_jsonld_fallback(html: &Document, chapters: &mut Vec<Chapter>) {
	// Look for JSON-LD script with schema.org data
	if let Some(jsonld_scripts) = html.select("script[type=\"application/ld+json\"]") {
		for script in jsonld_scripts {
			if let Some(script_content) = script.text() {
				// Try to parse as JSON to find date fields
				if let Ok(json_val) = serde_json::from_str::<serde_json::Value>(&script_content) {
					if let Some(json_obj) = json_val.as_object() {
						// Look for dateModified or datePublished fields
						if let Some(date_modified) = json_obj.get("dateModified").and_then(|v| v.as_str()) {
							if let Some(timestamp) = parse_iso_date_string(date_modified) {
								// Apply this date to chapters that don't have dates yet
								apply_fallback_date_to_chapters(chapters, timestamp);
								return;
							}
						}
						
						if let Some(date_published) = json_obj.get("datePublished").and_then(|v| v.as_str()) {
							if let Some(timestamp) = parse_iso_date_string(date_published) {
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
}

// Apply fallback date only to chapters that don't have dates yet (date_uploaded == None)
fn apply_fallback_date_to_chapters(chapters: &mut Vec<Chapter>, fallback_timestamp: i64) {
	for chapter in chapters.iter_mut() {
		if chapter.date_uploaded.is_none() {
			chapter.date_uploaded = Some(fallback_timestamp);
		}
	}
}

// Parse ISO date string to timestamp (simplified version)
fn parse_iso_date_string(date_str: &str) -> Option<i64> {
	use aidoku::imports::std::current_date;
	
	// Very basic ISO date parsing for fallback
	// For production, this should be more robust
	if date_str.contains("2025") || date_str.contains("2024") {
		// Use current date as reasonable fallback for schema.org dates
		Some(current_date() as i64)
	} else {
		None
	}
}

pub fn parse_page_list(html: &Document, _chapter_url: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	// Extract images from HTML - try multiple selectors
	let image_selectors = [
		"img[alt*='Chapter Image']",
		"img[src*='/chapter/']",
		"img[src*='/images/']",
		"img[data-src]",
		"main img",
		".chapter-content img",
		".manga-reader img",
	];

	for selector in &image_selectors {
		if let Some(img_elements) = html.select(selector) {
			for img_element in img_elements {
				// Get image URL from various attributes
				let image_url = img_element.attr("src")
					.or_else(|| img_element.attr("data-src"))
					.or_else(|| img_element.attr("data-original"))
					.or_else(|| img_element.attr("data-lazy"));

				if let Some(url) = image_url {
					if !url.is_empty() && !url.contains("placeholder") && !url.contains("loading") {
						let absolute_url = if url.starts_with("http") {
							url
						} else if url.starts_with("/") {
							format!("{}{}", BASE_URL, url)
						} else {
							format!("{}/{}", BASE_URL, url)
						};

						pages.push(Page {
							content: PageContent::url(absolute_url),
							thumbnail: None,
							has_description: false,
							description: None,
						});
					}
				}
			}
		}

		// If we found images with this selector, stop trying others
		if !pages.is_empty() {
			break;
		}
	}

	Ok(pages)
}

// Helper functions

fn parse_manga_status(status: &str) -> MangaStatus {
	let status_lower = status.to_lowercase();
	
	if status_lower.contains("en cours") || status_lower.contains("ongoing") {
		MangaStatus::Ongoing
	} else if status_lower.contains("terminé") || status_lower.contains("completed") {
		MangaStatus::Completed
	} else if status_lower.contains("pause") || status_lower.contains("hiatus") {
		MangaStatus::Hiatus
	} else if status_lower.contains("annulé") || status_lower.contains("cancelled") {
		MangaStatus::Cancelled
	} else {
		MangaStatus::Unknown
	}
}

fn extract_chapter_id_from_url(url: &str) -> Option<String> {
	// Extract chapter ID from URL pattern like "/serie/manga-slug/chapter/123"
	if let Some(chapter_pos) = url.find("/chapter/") {
		let after_chapter = &url[chapter_pos + 9..]; // 9 = len("/chapter/")
		if let Some(end_pos) = after_chapter.find('?').or_else(|| after_chapter.find('#')) {
			Some(after_chapter[..end_pos].to_string())
		} else {
			Some(after_chapter.to_string())
		}
	} else {
		None
	}
}

fn extract_chapter_number_from_title(title: &str) -> Option<f32> {
	// Try to extract chapter number from title
	let title_lower = title.to_lowercase();
	
	// Pattern: "Chapitre 123" or "Chapter 123"
	if let Some(chap_pos) = title_lower.find("chapitre").or_else(|| title_lower.find("chapter")) {
		let after_chap = &title[chap_pos..];
		for word in after_chap.split_whitespace().skip(1) {
			if let Ok(num) = word.parse::<f32>() {
				return Some(num);
			}
		}
	}
	
	// Pattern: numbers in the title
	for word in title.split_whitespace() {
		if let Ok(num) = word.parse::<f32>() {
			return Some(num);
		}
	}
	
	None
}

fn extract_chapter_number_from_id(chapter_id: &str) -> Option<f32> {
	// Try to parse chapter ID as number
	chapter_id.parse::<f32>().ok()
}

