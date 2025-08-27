use aidoku::{
	error::Result, prelude::*, std::{
		current_date, ObjectRef, String, StringRef, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;

pub fn parse_manga_listing(json: ObjectRef, listing_type: &str, page: i32) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	// FMTeam API returns all comics in an array, we need to filter and sort
	let comics = json.0.as_array()?;
	
	let mut filtered_comics: Vec<ObjectRef> = Vec::new();
	
	if listing_type == "Populaire" {
		// For popular, we could sort by some metric (for now just take first ones)
		for comic in comics {
			filtered_comics.push(comic.as_object()?);
			if filtered_comics.len() >= 20 {
				break;
			}
		}
	} else {
		// For latest, filter comics that have chapters and sort by latest chapter date
		for comic in comics {
			let comic_obj = comic.as_object()?;
			if comic_obj.get("chapters").as_array().is_ok() {
				let chapters = comic_obj.get("chapters").as_array()?;
				if chapters.len() > 0 {
					filtered_comics.push(comic_obj);
				}
			}
		}
		
		// Take only a subset for pagination simulation
		let start_index = ((page - 1) * 20) as usize;
		let end_index = (start_index + 20).min(filtered_comics.len());
		filtered_comics = filtered_comics[start_index..end_index].to_vec();
	}

	for comic in filtered_comics {
		let id = comic.get("id").as_string()?.read();
		let title = comic.get("title").as_string()?.read();
		let cover = if comic.get("cover").is_some() {
			format!("{}{}", String::from(BASE_URL), comic.get("cover").as_string()?.read())
		} else {
			String::from("")
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
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
	}

	// Simple pagination simulation
	let has_more = if listing_type == "Populaire" {
		false  // Popular shows only top results
	} else {
		mangas.len() == 20  // Has more if we got a full page
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_list(json: ObjectRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	let comics = json.0.as_array()?;
	
	for comic in comics {
		let comic_obj = comic.as_object()?;
		
		let id = comic_obj.get("id").as_string()?.read();
		let title = comic_obj.get("title").as_string()?.read();
		let cover = if comic_obj.get("cover").is_some() {
			format!("{}{}", String::from(BASE_URL), comic_obj.get("cover").as_string()?.read())
		} else {
			String::from("")
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
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,  // FMTeam API doesn't seem to have pagination info
	})
}

pub fn parse_search_list(json: ObjectRef) -> Result<MangaPageResult> {
	// Search uses same structure as regular list
	parse_manga_list(json)
}

pub fn parse_manga_details(manga_id: String, json: ObjectRef) -> Result<Manga> {	
	// Get cover image
	let cover = if json.get("cover").is_some() {
		format!("{}{}", String::from(BASE_URL), json.get("cover").as_string()?.read())
	} else {
		String::from("")
	};
	
	// Get title
	let title = json.get("title").as_string()?.read();

	// Get description
	let description = if json.get("description").is_some() && !json.get("description").as_string()?.read().is_empty() {
		json.get("description").as_string()?.read()
	} else {
		String::from("Aucune description disponible.")
	};

	// Get URL
	let url = format!("{}/comics/{}", String::from(BASE_URL), manga_id);

	// Get manga status
	let status_str = json.get("status").as_string()?.read();
	let status = match status_str.get(0..7).unwrap_or("").to_lowercase().as_str() {
		"ongoing" | "en cour" | "in cors" => MangaStatus::Ongoing,
		"complet" | "termina" => MangaStatus::Completed,
		"licenzi" => MangaStatus::Cancelled,
		_ => MangaStatus::Unknown,
	};

	// Get categories (genres)
	let mut categories: Vec<String> = Vec::new();
	if json.get("genres").is_some() {
		for item in json.get("genres").as_array()? {
			let genre = item.as_object()?;
			categories.push(genre.get("name").as_string()?.read());
		}
	}

	// Get author if available
	let author = if json.get("author").is_some() {
		json.get("author").as_string()?.read()
	} else {
		String::new()
	};

	// Get artist if available  
	let artist = if json.get("artist").is_some() {
		json.get("artist").as_string()?.read()
	} else {
		String::new()
	};

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

pub fn parse_chapter_list(manga_id: String, json: ObjectRef) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	for item in json.get("chapters").as_array()? {
		let chapter_obj = item.as_object()?;
		
		// Get chapter and subchapter numbers
		let chapter_num = chapter_obj.get("chapter").as_int()? as f32;
		let subchapter_num = chapter_obj.get("subchapter").as_int().unwrap_or(0) as f32;
		
		// Calculate final chapter number (chapter.subchapter format)
		let final_chapter_num = if subchapter_num > 0.0 {
			chapter_num + (subchapter_num / 10.0)
		} else {
			chapter_num
		};
		
		let id = chapter_obj.get("id").as_string()?.read();
		let chapter_title = if chapter_obj.get("title").is_some() && !chapter_obj.get("title").as_string()?.read().is_empty() {
			chapter_obj.get("title").as_string()?.read()
		} else {
			format!("Chapter {}", final_chapter_num)
		};
		
		// Parse date
		let date_str = chapter_obj.get("date").as_string()?.read();
		let mut date_updated = StringRef::from(&date_str)
			.0
			.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSSSS", Some("it"), None)
			.unwrap_or(-1.0);

		if date_updated == -1.0 {
			date_updated = current_date();
		}
		
		// Get scanlator teams
		let mut scanlator_teams: Vec<String> = Vec::new();
		if chapter_obj.get("teams").is_some() {
			for team in chapter_obj.get("teams").as_array()? {
				let team_obj = team.as_object()?;
				scanlator_teams.push(team_obj.get("name").as_string()?.read());
			}
		}
		let scanlator = scanlator_teams.join(", ");

		let chapter_url = format!("{}/comics/{}/chapters/{}", String::from(BASE_URL), manga_id, id);

		chapters.push(Chapter{
			id,
			title: chapter_title,
			volume: -1.0,
			chapter: final_chapter_num,
			date_updated,
			scanlator,
			url: chapter_url,
			lang: String::from("fr"),
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(json: ObjectRef) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in json.get("pages").as_array()?.enumerate() {
		let page_obj = item.as_object()?;
		let page_url = format!("{}{}", String::from(BASE_URL), page_obj.get("url").as_string()?.read());
		
		pages.push(Page {
			index: index as i32,
			url: page_url,
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}