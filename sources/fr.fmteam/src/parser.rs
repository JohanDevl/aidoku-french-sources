use aidoku::{
	prelude::*, std::{ObjectRef, String, StringRef, Vec, current_date},
	Manga, MangaPageResult, Chapter, MangaContentRating, MangaStatus, MangaViewer
};

use crate::BASE_URL;

pub fn parse_manga_listing(json: ObjectRef, listing_type: &str, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	// FMTeam API returns {comics: [...]} structure
	let comics = json.get("comics").as_array()?;
	
	let mut filtered_comics: Vec<ObjectRef> = Vec::new();
	
	if listing_type == "Populaire" {
		// Sort by rating/views for popular (take highest rated first)
		let mut comic_vec: Vec<ObjectRef> = Vec::new();
		for comic in comics {
			comic_vec.push(comic.as_object()?);
		}
		// Simple popularity filter - take first 20 (could be improved with actual sorting)
		filtered_comics = comic_vec.into_iter().take(20).collect();
	} else {
		// For latest, filter comics that have last_chapter and sort by date
		for comic in comics {
			let comic_obj = comic.as_object()?;
			if comic_obj.get("last_chapter").is_some() {
				filtered_comics.push(comic_obj);
			}
		}
		
		// Take only a subset for pagination simulation
		let start_index = ((page - 1) * 20) as usize;
		let end_index = (start_index + 20).min(filtered_comics.len());
		if start_index < filtered_comics.len() {
			filtered_comics = filtered_comics[start_index..end_index].to_vec();
		} else {
			filtered_comics.clear();
		}
	}

	for comic in filtered_comics {
		// Use real slug from API instead of creating artificial ones
		let title = comic.get("title").as_string()?.read();
		let id = if comic.get("slug").is_some() {
			comic.get("slug").as_string()?.read()
		} else {
			// Fallback to creating slug from title if slug field missing
			title.to_lowercase().replace(" ", "-").replace("'", "")
		};
		
		let cover = if comic.get("thumbnail").is_some() {
			let thumbnail_url = comic.get("thumbnail").as_string()?.read();
			// Use full URL if it starts with http, otherwise prepend base URL
			if thumbnail_url.starts_with("http") {
				Some(thumbnail_url)
			} else {
				Some(format!("{}{}", BASE_URL, thumbnail_url))
			}
		} else {
			None
		};

		// Parse status from Italian/French/English
		let status_str = comic.get("status").as_string()?.read();
		let status = match status_str.get(0..7).unwrap_or("").to_lowercase().as_str() {
			"ongoing" | "en cour" | "in cors" => MangaStatus::Ongoing,
			"complet" | "termina" => MangaStatus::Completed,
			"licenzi" => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		};

		mangas.push(Manga {
			id,
			title,
			cover,
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	// Simple pagination simulation
	let has_next_page = if listing_type == "Populaire" {
		false  // Popular shows only top results
	} else {
		mangas.len() == 20  // Has more if we got a full page
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more: has_next_page,
	})
}

pub fn parse_manga_list(json: ObjectRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	let comics = json.get("comics").as_array()?;
	
	for comic in comics {
		let comic_obj = comic.as_object()?;
		
		// Use real slug from API instead of creating artificial ones
		let title = comic_obj.get("title").as_string()?.read();
		let id = if comic_obj.get("slug").is_some() {
			comic_obj.get("slug").as_string()?.read()
		} else {
			// Fallback to creating slug from title if slug field missing
			title.to_lowercase().replace(" ", "-").replace("'", "")
		};
		
		let cover = if comic_obj.get("thumbnail").is_some() {
			let thumbnail_url = comic_obj.get("thumbnail").as_string()?.read();
			// Use full URL if it starts with http, otherwise prepend base URL
			if thumbnail_url.starts_with("http") {
				Some(thumbnail_url)
			} else {
				Some(format!("{}{}", BASE_URL, thumbnail_url))
			}
		} else {
			None
		};

		// Parse status 
		let status_str = comic_obj.get("status").as_string()?.read();
		let status = match status_str.get(0..7).unwrap_or("").to_lowercase().as_str() {
			"ongoing" | "en cour" | "in cors" => MangaStatus::Ongoing,
			"complet" | "termina" => MangaStatus::Completed,
			"licenzi" => MangaStatus::Cancelled,
			_ => MangaStatus::Unknown,
		};

		mangas.push(Manga {
			id,
			title,
			cover,
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll,
			..Default::default()
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,  // FMTeam API doesn't seem to have pagination info
	})
}

pub fn parse_search_list(json: ObjectRef) -> Result<MangaPageResult> {
	// FMTeam search returns a single comic wrapped in {comics: [comic]}
	// but we need to handle it as a list
	parse_manga_list(json)
}

pub fn parse_manga_details(manga_id: String, json: ObjectRef) -> Result<Manga> {	
	// /comics/[slug] endpoint returns {"comic": {...}} structure
	let comic = json.get("comic").as_object()?;
	
	// Get cover image using thumbnail field
	let cover = if comic.get("thumbnail").is_some() {
		let thumbnail_url = comic.get("thumbnail").as_string()?.read();
		if thumbnail_url.starts_with("http") {
			Some(thumbnail_url)
		} else {
			Some(format!("{}{}", BASE_URL, thumbnail_url))
		}
	} else {
		None
	};
	
	// Get title
	let title = comic.get("title").as_string()?.read();

	// Get description
	let description = if comic.get("description").is_some() && !comic.get("description").as_string()?.read().is_empty() {
		Some(comic.get("description").as_string()?.read())
	} else {
		Some("Aucune description disponible.".into())
	};

	// Get URL
	let url = Some(format!("{}/comics/{}", BASE_URL, manga_id));

	// Get manga status
	let status_str = comic.get("status").as_string()?.read();
	let status = match status_str.get(0..7).unwrap_or("").to_lowercase().as_str() {
		"ongoing" | "en cour" | "in cors" => MangaStatus::Ongoing,
		"complet" | "termina" => MangaStatus::Completed,
		"licenzi" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	// Get categories (genres) - now called categories
	let mut categories: Vec<String> = Vec::new();
	if comic.get("genres").is_some() {
		for item in comic.get("genres").as_array()? {
			let genre = item.as_object()?;
			categories.push(genre.get("name").as_string()?.read());
		}
	}

	// Get author if available
	let author = if comic.get("author").is_some() {
		Some(comic.get("author").as_string()?.read())
	} else {
		None
	};

	// Get artist if available  
	let artist = if comic.get("artist").is_some() {
		Some(comic.get("artist").as_string()?.read())
	} else {
		None
	};

	Ok(Manga {
		id: manga_id,
		title,
		cover,
		author,
		artist,
		description,
		url,
		categories: if categories.is_empty() { None } else { Some(categories) },
		status,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll,
		..Default::default()
	})
}

pub fn parse_chapter_list(manga_id: String, json: ObjectRef) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// /comics/[slug] endpoint returns {"comic": {...}} structure
	let comic = json.get("comic").as_object()?;
	
	// Parse complete chapters array instead of just last_chapter
	if comic.get("chapters").is_some() {
		for chapter_item in comic.get("chapters").as_array()? {
			let chapter_obj = chapter_item.as_object()?;
			
			// Get chapter number
			let chapter_num = if chapter_obj.get("chapter").as_int().is_ok() {
				Some(chapter_obj.get("chapter").as_int()? as f32)
			} else if chapter_obj.get("chapter").as_float().is_ok() {
				Some(chapter_obj.get("chapter").as_float()? as f32)
			} else {
				Some(1.0)  // Default if no chapter number
			};
			
			// Create chapter key from manga_id and chapter number
			let id = format!("{}/{}", manga_id, chapter_num.unwrap_or(1.0) as i32);
			
			// Get chapter title
			let title = if chapter_obj.get("title").is_some() && !chapter_obj.get("title").as_string()?.read().is_empty() {
				Some(chapter_obj.get("title").as_string()?.read())
			} else {
				Some(format!("Chapter {}", chapter_num.unwrap_or(1.0)))
			};
			
			// Parse date if available
			let date_updated = if chapter_obj.get("date").is_some() {
				let date_str = chapter_obj.get("date").as_string()?.read();
				Some(StringRef::from(&date_str)
					.0
					.as_date("yyyy-MM-dd'T'HH:mm:ss", Some("fr"), None)
					.unwrap_or(current_date()))
			} else {
				Some(current_date())
			};
			
			// Get teams if available
			let scanlator = if chapter_obj.get("teams").is_some() {
				let mut team_names: Vec<String> = Vec::new();
				for team in chapter_obj.get("teams").as_array()? {
					let team_obj = team.as_object()?;
					if team_obj.get("name").is_some() {
						team_names.push(team_obj.get("name").as_string()?.read());
					}
				}
				if !team_names.is_empty() {
					Some(team_names.join(", "))
				} else {
					Some("FMTeam".into())
				}
			} else {
				Some("FMTeam".into())
			};
			
			let chapter_url = Some(format!("{}/read/{}/fr/ch/{}", BASE_URL, manga_id, chapter_num.unwrap_or(1.0) as i32));

			chapters.push(Chapter{
				id,
				title,
				volume: None, // No volume info available
				chapter: chapter_num,
				date_updated,
				scanlator,
				url: chapter_url,
				lang: "fr".into(),
				..Default::default()
			});
		}
	}

	Ok(chapters)
}