use aidoku::{
	error::Result, prelude::*, std::{
		current_date, ObjectRef, String, StringRef, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;
use crate::API_URL;

pub fn parse_manga_listing(json: ObjectRef, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let has_more = if listing_type == "Populaire" {
		// For the "top" section, the structure is: { "top": [...] }
		for item in json.get("top").as_array()? {
			let manga = item.as_object()?;

			if manga.get("slug").as_string()?.read() == "unknown" {
				continue
			}
			
			let id = manga.get("slug").as_string()?.read();
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());

			mangas.push(Manga {
				id,
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
		// Top section has no pagination
		false
	} else {
		// For the "latest" section, the structure is: { "pagination": {...}, "latest": [...] }
		for item in json.get("latest").as_array()? {
			let manga = item.as_object()?;

			if manga.get("slug").as_string()?.read() == "unknown" {
				continue
			}
			
			let id = manga.get("slug").as_string()?.read();
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());

			mangas.push(Manga {
				id,
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
		// Check if there are more pages
		let pagination = json.get("pagination").as_object()?;
		let current_page = pagination.get("currentPage").as_int()?;
		let total_pages = pagination.get("totalPages").as_int()?;
		current_page < total_pages
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_list(json: ObjectRef) -> Result<MangaPageResult>  {
	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in json.get("mangas").as_array()? {
		let manga = item.as_object()?;

		if manga.get("slug").as_string()?.read() == "unknown" {
			continue
		}
		
		let id = manga.get("slug").as_string()?.read();
		let title = manga.get("title").as_string()?.read();
		let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());
		let status_str = manga.get("status").as_string()?.read();
		let status = match status_str.as_str() {
			"Ongoing" => MangaStatus::Ongoing,
			"Completed" => MangaStatus::Completed,
			"Hiatus" => MangaStatus::Hiatus,
			_ => MangaStatus::Unknown,
		};
		let manga_type = manga.get("type").as_string()?.read();
		let viewer = match manga_type.as_str() {
			"Manga" => MangaViewer::Rtl,
			_ => MangaViewer::Scroll,
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
			viewer
		});
	}

	// Check pagination for general list
	let has_more = if json.get("pagination").is_some() {
		let pagination = json.get("pagination").as_object()?;
		if pagination.get("hasNextPage").is_some() {
			pagination.get("hasNextPage").as_bool()?
		} else if pagination.get("page").is_some() && pagination.get("totalPages").is_some() {
			let current_page = pagination.get("page").as_int()?;
			let total_pages = pagination.get("totalPages").as_int()?;
			current_page < total_pages
		} else {
			false
		}
	} else {
		false
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_search_list(json: ObjectRef) -> Result<MangaPageResult>  {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Search structure: { "mangas": [...], "pagination": {...} }
	for item in json.get("mangas").as_array()? {
		let manga = item.as_object()?;

		if manga.get("slug").as_string()?.read() == "unknown" {
			continue
		}
		
		let id = manga.get("slug").as_string()?.read();
		let title = manga.get("title").as_string()?.read();
		let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());

		mangas.push(Manga {
			id,
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

	// Check pagination for searches if it exists
	let has_more = if json.get("pagination").is_some() {
		let pagination = json.get("pagination").as_object()?;
		let current_page = pagination.get("page").as_int().unwrap_or(0);
		let total_pages = pagination.get("totalPages").as_int().unwrap_or(0);
		current_page < total_pages
	} else {
		false
	};

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_details(manga_id: String, json: ObjectRef) -> Result<Manga> {	
	let manga = json.get("manga").as_object()?;

	// Get cover image
	let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());
	
	// Get title
	let title = manga.get("title").as_string()?.read();

	// Get description (with default value)
	let description = if manga.get("synopsis").is_some() && !manga.get("synopsis").as_string()?.read().is_empty() {
		manga.get("synopsis").as_string()?.read()
	} else {
		String::from("Aucune description disponible.")
	};

	// Get URL
	let url = format!("{}/manga/{}", String::from(BASE_URL), manga_id);

	// Get manga status
	let status_str = manga.get("status").as_string()?.read();
	let status = match status_str.as_str() {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};

    // Get categories (genres)
	let mut categories: Vec<String> = Vec::new();
	for item in manga.get("genres").as_array()? {
		let genre = item.as_object()?;
		categories.push(genre.get("name").as_string()?.read());
	}

	// Get manga type
	let manga_type = manga.get("type").as_string()?.read();
	let viewer = match manga_type.as_str() {
		"Manga" => MangaViewer::Rtl,
		_ => MangaViewer::Scroll,
	};

	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Safe,
		viewer
	})
}

pub fn parse_chapter_list(manga_id: String, json: ObjectRef) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	for item in json.get("chapters").as_array()? {
		let chapter_object = item.as_object()?;
		
		// Check price - only take free chapters (price == 0)
		let price = chapter_object.get("price").as_int().unwrap_or(0);
		if price != 0 {
			continue;
		}
		
		// Chapter number can be an integer or a float/string
		let chapter_number = if chapter_object.get("number").as_float().is_ok() {
			chapter_object.get("number").as_float()? as f32
		} else {
			chapter_object.get("number").as_int()? as f32
		};
		
		let id = format!("{}", chapter_number);
		let date_str = chapter_object.get("createdAt").as_string()?.read();
		let mut date_updated = StringRef::from(&date_str)
			.0
			.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSZ", Some("fr"), None)
			.unwrap_or(-1.0);
	
		if date_updated == -1.0 {
			date_updated = current_date();
		}
		
		let title = format!("Chapter {}", chapter_number);
		let url = format!("{}/manga/{}/chapitre/{}", String::from(BASE_URL), manga_id, chapter_number);

		chapters.push(Chapter{
			id,
			title,
			volume: -1.0,
			chapter: chapter_number,
			date_updated,
			scanlator: String::from(""),
			url,
			lang: String::from("fr"),
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(json: ObjectRef) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for (index, item) in json.get("chapter").as_object()?.get("images").as_array()?.enumerate() {
		pages.push(Page {
			index: index as i32,
			url: format!("{}/{}", String::from(API_URL), item.as_string()?.read()),
			base64: String::new(),
			text: String::new(),
		});
	}

	Ok(pages)
}