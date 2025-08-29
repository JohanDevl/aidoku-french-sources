use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString},
	imports::std::current_date,
	prelude::*,
};

use crate::BASE_URL;
use crate::API_URL;

fn parse_manga_status(status_str: &str) -> MangaStatus {
	match status_str {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	}
}

// Helper function to extract simple JSON value by key
fn get_json_string_value(json: &str, key: &str) -> Option<String> {
	let search_key = format!("\"{}\":", key);
	if let Some(pos) = json.find(&search_key) {
		let start = pos + search_key.len();
		if let Some(content) = json.get(start..) {
			if let Some(quote_start) = content.find('"') {
				if let Some(quote_end) = content[quote_start + 1..].find('"') {
					return Some(content[quote_start + 1..quote_start + 1 + quote_end].to_string());
				}
			}
		}
	}
	None
}

// Helper function to extract simple JSON int value by key
fn get_json_int_value(json: &str, key: &str) -> Option<i32> {
	let search_key = format!("\"{}\":", key);
	if let Some(pos) = json.find(&search_key) {
		let start = pos + search_key.len();
		if let Some(content) = json.get(start..) {
			let trimmed = content.trim_start();
			if let Some(comma_pos) = trimmed.find(',') {
				if let Ok(num) = trimmed[..comma_pos].trim().parse::<i32>() {
					return Some(num);
				}
			} else if let Some(brace_pos) = trimmed.find('}') {
				if let Ok(num) = trimmed[..brace_pos].trim().parse::<i32>() {
					return Some(num);
				}
			}
		}
	}
	None
}

// Extract array content (simple approach)
fn get_json_array_content(json: &str, key: &str) -> Option<String> {
	let search_key = format!("\"{}\":[", key);
	if let Some(start_pos) = json.find(&search_key) {
		let array_start = start_pos + search_key.len() - 1; // Keep the [
		let content = &json[array_start..];
		
		// Find matching closing bracket
		let mut bracket_count = 0;
		let mut in_string = false;
		let mut escaped = false;
		
		for (i, c) in content.char_indices() {
			if escaped {
				escaped = false;
				continue;
			}
			
			match c {
				'\\' if in_string => escaped = true,
				'"' => in_string = !in_string,
				'[' if !in_string => bracket_count += 1,
				']' if !in_string => {
					bracket_count -= 1;
					if bracket_count == 0 {
						return Some(content[..=i].to_string());
					}
				}
				_ => {}
			}
		}
	}
	None
}

// Parse individual manga objects from JSON array content
fn parse_manga_objects(array_content: &str) -> Vec<String> {
	let mut objects = Vec::new();
	
	// Remove outer brackets if present
	let content = if array_content.starts_with('[') && array_content.ends_with(']') {
		&array_content[1..array_content.len() - 1]
	} else {
		array_content
	};
	
	let mut current_object = String::new();
	let mut brace_count = 0;
	let mut in_string = false;
	let mut escaped = false;
	
	for c in content.chars() {
		if escaped {
			escaped = false;
			current_object.push(c);
			continue;
		}
		
		match c {
			'\\' if in_string => {
				escaped = true;
				current_object.push(c);
			}
			'"' => {
				in_string = !in_string;
				current_object.push(c);
			}
			'{' if !in_string => {
				brace_count += 1;
				current_object.push(c);
			}
			'}' if !in_string => {
				brace_count -= 1;
				current_object.push(c);
				if brace_count == 0 {
					objects.push(current_object.trim().to_string());
					current_object.clear();
				}
			}
			',' if !in_string && brace_count == 0 => {
				// Skip comma between objects
			}
			_ => {
				current_object.push(c);
			}
		}
	}
	
	objects
}

pub fn parse_manga_listing(response: String, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let has_more = if listing_type == "Populaire" {
		// For the "top" section
		if let Some(top_array) = get_json_array_content(&response, "top") {
			let manga_objects = parse_manga_objects(&top_array);
			for manga_json in manga_objects {
				if let Some(slug) = get_json_string_value(&manga_json, "slug") {
					if slug == "unknown" { continue; }
					
					let title = get_json_string_value(&manga_json, "title").unwrap_or_else(|| "Unknown Title".to_string());
					let cover_image = get_json_string_value(&manga_json, "coverImage").unwrap_or_else(|| "".to_string());
					let cover = if !cover_image.is_empty() {
						Some(format!("{}/{}", API_URL, cover_image))
					} else {
						None
					};

					mangas.push(Manga {
						key: slug,
						title,
						cover,
						authors: None,
						artists: None,
						description: None,
						url: None,
						tags: None,
						status: MangaStatus::Unknown,
						content_rating: ContentRating::Safe,
						viewer: Viewer::default(),
						chapters: None,
						next_update_time: None,
						update_strategy: UpdateStrategy::Never,
					});
				}
			}
		}
		false // Top section has no pagination
	} else {
		// For the "latest" section
		if let Some(latest_array) = get_json_array_content(&response, "latest") {
			let manga_objects = parse_manga_objects(&latest_array);
			for manga_json in manga_objects {
				// Use slug or _id as key
				let key = if let Some(slug) = get_json_string_value(&manga_json, "slug") {
					if slug == "unknown" { continue; }
					slug
				} else if let Some(id) = get_json_string_value(&manga_json, "_id") {
					id
				} else {
					continue;
				};
				
				let title = get_json_string_value(&manga_json, "title").unwrap_or_else(|| "Unknown Title".to_string());
				let cover_image = get_json_string_value(&manga_json, "coverImage").unwrap_or_else(|| "".to_string());
				let cover = if !cover_image.is_empty() {
					Some(format!("{}/{}", API_URL, cover_image))
				} else {
					None
				};

				let status = if let Some(status_str) = get_json_string_value(&manga_json, "status") {
					parse_manga_status(&status_str)
				} else {
					MangaStatus::Unknown
				};

				let viewer = if let Some(manga_type) = get_json_string_value(&manga_json, "type") {
					if manga_type == "Manga" {
						Viewer::RightToLeft
					} else {
						Viewer::Vertical
					}
				} else {
					Viewer::Vertical
				};

				mangas.push(Manga {
					key,
					title,
					cover,
					authors: None,
					artists: None,
					description: None,
					url: None,
					tags: None,
					status,
					content_rating: ContentRating::Safe,
					viewer,
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
		
		// Check pagination
		let current_page = get_json_int_value(&response, "currentPage").unwrap_or(1);
		let total_pages = get_json_int_value(&response, "totalPages").unwrap_or(1);
		current_page < total_pages
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_list(response: String) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	if let Some(mangas_array) = get_json_array_content(&response, "mangas") {
		let manga_objects = parse_manga_objects(&mangas_array);
		for manga_json in manga_objects {
			if let Some(slug) = get_json_string_value(&manga_json, "slug") {
				if slug == "unknown" { continue; }
				
				let title = get_json_string_value(&manga_json, "title").unwrap_or_else(|| "Unknown Title".to_string());
				let cover_image = get_json_string_value(&manga_json, "coverImage").unwrap_or_else(|| "".to_string());
				let cover = if !cover_image.is_empty() {
					Some(format!("{}/{}", API_URL, cover_image))
				} else {
					None
				};
				
				let status = if let Some(status_str) = get_json_string_value(&manga_json, "status") {
					parse_manga_status(&status_str)
				} else {
					MangaStatus::Unknown
				};

				let viewer = if let Some(manga_type) = get_json_string_value(&manga_json, "type") {
					if manga_type == "Manga" {
						Viewer::RightToLeft
					} else {
						Viewer::Vertical
					}
				} else {
					Viewer::Vertical
				};

				mangas.push(Manga {
					key: slug,
					title,
					cover,
					authors: None,
					artists: None,
					description: None,
					url: None,
					tags: None,
					status,
					content_rating: ContentRating::Safe,
					viewer,
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
	}

	// Check pagination
	let has_more = if let Some(has_next) = get_json_string_value(&response, "hasNextPage") {
		has_next == "true"
	} else {
		let current_page = get_json_int_value(&response, "page").unwrap_or(1);
		let total_pages = get_json_int_value(&response, "totalPages").unwrap_or(1);
		current_page < total_pages
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_search_list(response: String) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Search structure: { "mangas": [...], "pagination": {...} }
	if let Some(mangas_array) = get_json_array_content(&response, "mangas") {
		let manga_objects = parse_manga_objects(&mangas_array);
		for manga_json in manga_objects {
			if let Some(slug) = get_json_string_value(&manga_json, "slug") {
				if slug == "unknown" { continue; }
				
				let title = get_json_string_value(&manga_json, "title").unwrap_or_else(|| "Unknown Title".to_string());
				let cover_image = get_json_string_value(&manga_json, "coverImage").unwrap_or_else(|| "".to_string());
				let cover = if !cover_image.is_empty() {
					Some(format!("{}/{}", API_URL, cover_image))
				} else {
					None
				};

				mangas.push(Manga {
					key: slug,
					title,
					cover,
					authors: None,
					artists: None,
					description: None,
					url: None,
					tags: None,
					status: MangaStatus::Unknown,
					content_rating: ContentRating::Safe,
					viewer: Viewer::default(),
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
	}

	// Check pagination
	let current_page = get_json_int_value(&response, "page").unwrap_or(0);
	let total_pages = get_json_int_value(&response, "totalPages").unwrap_or(0);
	let has_more = current_page < total_pages;

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_details(manga_id: String, response: String) -> Result<Manga> {
	// Find manga object in response
	if let Some(manga_start) = response.find("\"manga\":{") {
		let manga_content = &response[manga_start + 8..]; // Skip "manga":{
		
		// Find the end of this manga object
		let mut brace_count = 1;
		let mut in_string = false;
		let mut escaped = false;
		let mut end_pos = 0;
		
		for (i, c) in manga_content.char_indices() {
			if escaped {
				escaped = false;
				continue;
			}
			
			match c {
				'\\' if in_string => escaped = true,
				'"' => in_string = !in_string,
				'{' if !in_string => brace_count += 1,
				'}' if !in_string => {
					brace_count -= 1;
					if brace_count == 0 {
						end_pos = i;
						break;
					}
				}
				_ => {}
			}
		}
		
		let manga_json = &manga_content[..end_pos];
		
		// Parse manga details
		let title = get_json_string_value(&manga_json, "title").unwrap_or_else(|| "Unknown Title".to_string());
		let cover_image = get_json_string_value(&manga_json, "coverImage").unwrap_or_else(|| "".to_string());
		let cover = if !cover_image.is_empty() {
			Some(format!("{}/{}", API_URL, cover_image))
		} else {
			None
		};
		
		let description = get_json_string_value(&manga_json, "synopsis").map(|s| if s.is_empty() {
			"Aucune description disponible.".to_string()
		} else {
			s
		}).or_else(|| Some("Aucune description disponible.".to_string()));
		
		let status = if let Some(status_str) = get_json_string_value(&manga_json, "status") {
			parse_manga_status(&status_str)
		} else {
			MangaStatus::Unknown
		};
		
		let viewer = if let Some(manga_type) = get_json_string_value(&manga_json, "type") {
			if manga_type == "Manga" {
				Viewer::RightToLeft
			} else {
				Viewer::Vertical
			}
		} else {
			Viewer::Vertical
		};
		
		Ok(Manga {
			key: manga_id.clone(),
			title,
			cover,
			authors: None,
			artists: None,
			description,
			url: Some(format!("{}/manga/{}", BASE_URL, manga_id)),
			tags: None, // Simplified - could parse genres if needed
			status,
			content_rating: ContentRating::Safe,
			viewer,
			chapters: None,
			next_update_time: None,
			update_strategy: UpdateStrategy::Never,
		})
	} else {
		Ok(Manga {
			key: manga_id.clone(),
			title: "Unknown Title".to_string(),
			cover: None,
			authors: None,
			artists: None,
			description: Some("Aucune description disponible.".to_string()),
			url: Some(format!("{}/manga/{}", BASE_URL, manga_id)),
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::default(),
			chapters: None,
			next_update_time: None,
			update_strategy: UpdateStrategy::Never,
		})
	}
}

pub fn parse_chapter_list(manga_id: String, response: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	if let Some(chapters_array) = get_json_array_content(&response, "chapters") {
		let chapter_objects = parse_manga_objects(&chapters_array);
		for chapter_json in chapter_objects {
			// Check price - only take free chapters
			let price = get_json_int_value(&chapter_json, "price").unwrap_or(0);
			if price != 0 {
				continue;
			}
			
			// Get chapter number
			let chapter_number = if let Some(num_str) = get_json_string_value(&chapter_json, "number") {
				num_str.parse::<f32>().unwrap_or(1.0)
			} else {
				get_json_int_value(&chapter_json, "number").unwrap_or(1) as f32
			};
			
			let key = format!("{}", chapter_number);
			let title = Some(format!("Chapter {}", chapter_number));
			let url = Some(format!("{}/manga/{}/chapitre/{}", BASE_URL, manga_id, chapter_number));

			chapters.push(Chapter {
				key,
				title,
				volume_number: Some(-1.0),
				chapter_number: Some(chapter_number),
				date_uploaded: Some(current_date()),
				scanlators: None,
				url,
				language: Some("fr".to_string()),
				thumbnail: None,
				locked: false,
			});
		}
	}

	Ok(chapters)
}

pub fn parse_page_list(response: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	// Find chapter object and its images array
	if let Some(images_array) = get_json_array_content(&response, "images") {
		// Parse the images array - it should contain strings
		let content = if images_array.starts_with('[') && images_array.ends_with(']') {
			&images_array[1..images_array.len() - 1]
		} else {
			&images_array
		};
		
		let mut current_string = String::new();
		let mut in_string = false;
		let mut escaped = false;
		
		for c in content.chars() {
			if escaped {
				escaped = false;
				current_string.push(c);
				continue;
			}
			
			match c {
				'\\' if in_string => escaped = true,
				'"' => {
					if in_string {
						// End of string - add to pages
						let image_url = format!("{}/{}", API_URL, current_string);
						pages.push(Page {
							content: PageContent::url(image_url),
							thumbnail: None,
							has_description: false,
							description: None,
						});
						current_string.clear();
					}
					in_string = !in_string;
				}
				',' if !in_string => {
					// Skip commas outside strings
				}
				_ if in_string => {
					current_string.push(c);
				}
				_ => {}
			}
		}
	}

	Ok(pages)
}