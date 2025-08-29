use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::std::current_date,
	prelude::*,
};

use crate::BASE_URL;
use crate::API_URL;

// Helper function to extract JSON value by key
fn extract_json_value(json: &str, key: &str) -> Option<String> {
	// Simple JSON extraction using string manipulation - handle both compact and spaced JSON
	let (search_pos, offset) = if let Some(pos) = json.find(&format!("\"{}\":", key)) {
		// Compact JSON format: "key":value
		(pos, key.len() + 3) // Skip `"key":`
	} else if let Some(pos) = json.find(&format!("\"{}\": ", key)) {
		// Spaced JSON format: "key": value
		(pos, key.len() + 4) // Skip `"key": `
	} else {
		return None;
	};
	
	let start = search_pos + offset;
	
	if let Some(content) = json.get(start..) {
		if content.starts_with('"') {
			// String value
			if let Some(end) = content[1..].find('"') {
				return content.get(1..end + 1).map(|s| s.to_string());
			}
		} else if content.starts_with('[') {
			// Array value - find matching bracket (handle strings properly)
			let mut bracket_count = 0;
			let mut end_pos = 0;
			let mut in_string = false;
			let mut escape_next = false;
			
			for (i, c) in content.chars().enumerate() {
				if escape_next {
					escape_next = false;
				} else if c == '\\' {
					escape_next = true;
				} else if c == '"' && !escape_next {
					in_string = !in_string;
				}
				
				if !in_string {
					match c {
						'[' => bracket_count += 1,
						']' => {
							bracket_count -= 1;
							if bracket_count == 0 {
								end_pos = i + 1;
								break;
							}
						},
						_ => {}
					}
				}
			}
			
			if end_pos > 0 {
				return content.get(0..end_pos).map(|s| s.to_string());
			}
		} else if content.starts_with('{') {
			// Object value - find matching brace (handle strings properly)
			let mut brace_count = 0;
			let mut end_pos = 0;
			let mut in_string = false;
			let mut escape_next = false;
			
			for (i, c) in content.chars().enumerate() {
				if escape_next {
					escape_next = false;
				} else if c == '\\' {
					escape_next = true;
				} else if c == '"' && !escape_next {
					in_string = !in_string;
				}
				
				if !in_string {
					match c {
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
			}
			if end_pos > 0 {
				return content.get(0..end_pos).map(|s| s.to_string());
			}
		} else {
			// Number or boolean value
			let end = content.find(',')
				.or_else(|| content.find('}'))
				.or_else(|| content.find(']'))
				.unwrap_or(content.len());
			return content.get(0..end).map(|s| s.trim().to_string());
		}
	}
	None
}

// Extract array of objects from JSON
fn extract_json_array(json: &str, key: &str) -> Vec<String> {
	if let Some(array_str) = extract_json_value(json, key) {
		if array_str.starts_with('[') && array_str.ends_with(']') {
			let content = &array_str[1..array_str.len()-1]; // Remove brackets
			let mut objects = Vec::new();
			let mut current_object = String::new();
			let mut brace_count = 0;
			let mut in_string = false;
			let mut escape_next = false;
			
			for c in content.chars() {
				if escape_next {
					escape_next = false;
				} else if c == '\\' {
					escape_next = true;
				} else if c == '"' && !escape_next {
					in_string = !in_string;
				}
				
				if !in_string {
					match c {
						'{' => brace_count += 1,
						'}' => {
							brace_count -= 1;
							if brace_count == 0 {
								if !current_object.trim().is_empty() {
									objects.push(current_object.trim().to_string());
									current_object.clear();
								}
							}
						},
						',' if brace_count == 0 => {
							if !current_object.trim().is_empty() {
								objects.push(current_object.trim().to_string());
								current_object.clear();
							}
							continue;
						},
						_ => {}
					}
				}
				
				current_object.push(c);
			}
			
			if !current_object.trim().is_empty() {
				objects.push(current_object.trim().to_string());
			}
			
			return objects;
		}
	}
	Vec::new()
}

fn parse_manga_status(status_str: &str) -> MangaStatus {
	match status_str {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	}
}

pub fn parse_manga_listing(response: String, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();


	
	let has_more = if listing_type == "Populaire" {
		// For the "top" section, the structure is: { "top": [...] }
		let items = extract_json_array(&response, "top");
		for item in items {
			if let Some(slug) = extract_json_value(&item, "slug") {
				if slug == "unknown" { continue; }
				
				let title = extract_json_value(&item, "title").unwrap_or_else(|| "Unknown Title".to_string());
				let cover_image = extract_json_value(&item, "coverImage").unwrap_or_else(|| "".to_string());
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
		// Top section has no pagination
		false
	} else {
		// For the "latest" section, the structure is: { "pagination": {...}, "latest": [...] }
		
		let items = extract_json_array(&response, "latest");
		
		for item in &items {
			// Dans le JSON, les objets utilisent "_id" au lieu de "id" et parfois "slug"
			let id_field = extract_json_value(item, "_id").unwrap_or_else(|| "".to_string());
			let slug = extract_json_value(item, "slug");
			let title = extract_json_value(item, "title").unwrap_or_else(|| "Unknown Title".to_string());
			
			// Utiliser soit slug soit _id comme identifiant
			let final_id = if let Some(s) = slug {
				if s != "unknown" { s } else { continue; }
			} else if !id_field.is_empty() {
				id_field
			} else {
				continue;
			};
			
			let cover_image = extract_json_value(item, "coverImage").unwrap_or_else(|| "".to_string());
			let cover = if !cover_image.is_empty() {
				Some(format!("{}/{}", API_URL, cover_image))
			} else { None };

			let status_str = extract_json_value(item, "status").unwrap_or_else(|| "Unknown".to_string());
			let status = parse_manga_status(&status_str);

			let manga_type = extract_json_value(item, "type").unwrap_or_else(|| "Unknown".to_string());
			let viewer = if manga_type == "Manga" {
				Viewer::RightToLeft
			} else {
				Viewer::Vertical
			};

			mangas.push(Manga {
				key: final_id,
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
		
		// Check if there are more pages
		if let Some(pagination_str) = extract_json_value(&response, "pagination") {
			let current_page = extract_json_value(&pagination_str, "currentPage")
				.and_then(|s| s.parse::<i32>().ok())
				.unwrap_or(1);
			let total_pages = extract_json_value(&pagination_str, "totalPages")
				.and_then(|s| s.parse::<i32>().ok())
				.unwrap_or(1);
			current_page < total_pages
		} else {
			false
		}
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_list(response: String) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	let items = extract_json_array(&response, "mangas");
	for item in items {
		if let Some(slug) = extract_json_value(&item, "slug") {
			if slug == "unknown" { continue; }
			
			let title = extract_json_value(&item, "title").unwrap_or_else(|| "Unknown Title".to_string());
			let cover_image = extract_json_value(&item, "coverImage").unwrap_or_else(|| "".to_string());
			let cover = if !cover_image.is_empty() {
				Some(format!("{}/{}", API_URL, cover_image))
			} else {
				None
			};
			
			let status = if let Some(status_str) = extract_json_value(&item, "status") {
				parse_manga_status(&status_str)
			} else {
				MangaStatus::Unknown
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
				viewer: Viewer::default(),
				chapters: None,
				next_update_time: None,
				update_strategy: UpdateStrategy::Never,
			});
		}
	}

	// Check pagination for general list
	let has_more = if let Some(pagination_str) = extract_json_value(&response, "pagination") {
		if let Some(has_next_str) = extract_json_value(&pagination_str, "hasNextPage") {
			has_next_str.parse::<bool>().unwrap_or(false)
		} else {
			let current_page = extract_json_value(&pagination_str, "page")
				.and_then(|s| s.parse::<i32>().ok())
				.unwrap_or(1);
			let total_pages = extract_json_value(&pagination_str, "totalPages")
				.and_then(|s| s.parse::<i32>().ok())
				.unwrap_or(1);
			current_page < total_pages
		}
	} else {
		false
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_search_list(response: String) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Search structure: { "mangas": [...], "pagination": {...} }
	let items = extract_json_array(&response, "mangas");
	for item in items {
		if let Some(slug) = extract_json_value(&item, "slug") {
			if slug == "unknown" { continue; }
			
			let title = extract_json_value(&item, "title").unwrap_or_else(|| "Unknown Title".to_string());
			let cover_image = extract_json_value(&item, "coverImage").unwrap_or_else(|| "".to_string());
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

	// Check pagination for searches if it exists
	let has_more = if let Some(pagination_str) = extract_json_value(&response, "pagination") {
		let current_page = extract_json_value(&pagination_str, "page")
			.and_then(|s| s.parse::<i32>().ok())
			.unwrap_or(0);
		let total_pages = extract_json_value(&pagination_str, "totalPages")
			.and_then(|s| s.parse::<i32>().ok())
			.unwrap_or(0);
		current_page < total_pages
	} else {
		false
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_details(manga_id: String, response: String) -> Result<Manga> {
	if let Some(manga_str) = extract_json_value(&response, "manga") {
		// Get cover image
		let cover_image = extract_json_value(&manga_str, "coverImage").unwrap_or_else(|| "".to_string());
		let cover = if !cover_image.is_empty() {
			Some(format!("{}/{}", API_URL, cover_image))
		} else {
			None
		};
		
		// Get title
		let title = extract_json_value(&manga_str, "title").unwrap_or_else(|| "Unknown Title".to_string());

		// Get description (with default value)
		let description = if let Some(synopsis) = extract_json_value(&manga_str, "synopsis") {
			if !synopsis.is_empty() {
				Some(synopsis)
			} else {
				Some("Aucune description disponible.".to_string())
			}
		} else {
			Some("Aucune description disponible.".to_string())
		};

		// Get URL
		let url = Some(format!("{}/manga/{}", BASE_URL, manga_id));

		// Get manga status
		let status = if let Some(status_str) = extract_json_value(&manga_str, "status") {
			parse_manga_status(&status_str)
		} else {
			MangaStatus::Unknown
		};

		// Get tags (genres)
		let tags = if let Some(_genres_array) = extract_json_value(&manga_str, "genres") {
			let genre_objects = extract_json_array(&manga_str, "genres");
			let mut genre_names = Vec::new();
			for genre in genre_objects {
				if let Some(name) = extract_json_value(&genre, "name") {
					genre_names.push(name);
				}
			}
			if genre_names.is_empty() { None } else { Some(genre_names) }
		} else {
			None
		};

		Ok(Manga {
			key: manga_id,
			title,
			cover,
			authors: None,
			artists: None,
			description,
			url,
			tags,
			status,
			content_rating: ContentRating::Safe,
			viewer: Viewer::default(),
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
	
	let items = extract_json_array(&response, "chapters");
	for item in items {
		// Check price - only take free chapters (price == 0)
		let price = extract_json_value(&item, "price")
			.and_then(|s| s.parse::<i32>().ok())
			.unwrap_or(0);
		if price != 0 {
			continue;
		}
		
		// Chapter number can be an integer or a float/string
		let chapter_number = extract_json_value(&item, "number")
			.and_then(|s| s.parse::<f32>().ok())
			.unwrap_or(1.0);
		
		let key = format!("{}", chapter_number);
		let title = Some(format!("Chapter {}", chapter_number));
		let url = Some(format!("{}/manga/{}/chapitre/{}", BASE_URL, manga_id, chapter_number));

		// Parse date if available
		let date_uploaded = if let Some(_date_str) = extract_json_value(&item, "createdAt") {
			// For simplicity, we'll use current timestamp since date parsing is complex
			// In a real implementation, you'd want to parse the ISO date
			Some(current_date())
		} else {
			Some(current_date())
		};

		chapters.push(Chapter {
			key,
			title,
			volume_number: Some(-1.0),
			chapter_number: Some(chapter_number),
			date_uploaded,
			scanlators: None,
			url,
			language: Some("fr".to_string()),
			thumbnail: None,
			locked: false,
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(response: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	if let Some(chapter_str) = extract_json_value(&response, "chapter") {
		let images = extract_json_array(&chapter_str, "images");
		for image in images {
			// Remove quotes if present
			let image_path = image.trim_matches('"');
			let image_url = format!("{}/{}", API_URL, image_path);
			pages.push(Page {
				content: PageContent::url(image_url),
				thumbnail: None,
				has_description: false,
				description: None,
			});
		}
	}

	Ok(pages)
}