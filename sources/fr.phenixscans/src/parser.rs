use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, string::ToString, format},
	prelude::ValueRef,
	util::current_date,
};

use crate::BASE_URL;
use crate::API_URL;

pub fn parse_manga_listing(json: ValueRef, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	let has_more = if listing_type == "Populaire" {
		// For the "top" section, the structure is: { "top": [...] }
		for item in json.get("top").as_array()? {
			let manga = item.as_object()?;

			if manga.get("slug").as_string()?.read() == "unknown" {
				continue
			}
			
			let key = manga.get("slug").as_string()?.read();
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/{}", API_URL, manga.get("coverImage").as_string()?.read());

			mangas.push(Manga {
				key,
				title,
				cover: Some(cover),
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
		// Top section has no pagination
		false
	} else {
		// For the "latest" section, the structure is: { "pagination": {...}, "latest": [...] }
		for item in json.get("latest").as_array()? {
			let manga = item.as_object()?;

			if manga.get("slug").as_string()?.read() == "unknown" {
				continue
			}
			
			let key = manga.get("slug").as_string()?.read();
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/{}", API_URL, manga.get("coverImage").as_string()?.read());

			mangas.push(Manga {
				key,
				title,
				cover: Some(cover),
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
		// Check if there are more pages
		let pagination = json.get("pagination").as_object()?;
		let current_page = pagination.get("currentPage").as_int()?;
		let total_pages = pagination.get("totalPages").as_int()?;
		current_page < total_pages
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_list(json: ValueRef) -> Result<MangaPageResult>  {
	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in json.get("mangas").as_array()? {
		let manga = item.as_object()?;

		if manga.get("slug").as_string()?.read() == "unknown" {
			continue
		}
		
		let key = manga.get("slug").as_string()?.read();
		let title = manga.get("title").as_string()?.read();
		let cover = format!("{}/{}", API_URL, manga.get("coverImage").as_string()?.read());
		let status_str = manga.get("status").as_string()?.read();
		let status = match status_str.as_str() {
			"Ongoing" => MangaStatus::Ongoing,
			"Completed" => MangaStatus::Completed,
			"Hiatus" => MangaStatus::Hiatus,
			_ => MangaStatus::Unknown,
		};
		let manga_type = manga.get("type").as_string()?.read();
		let viewer = Viewer::default();

		mangas.push(Manga {
			key,
			title,
			cover: Some(cover),
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
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_search_list(json: ValueRef) -> Result<MangaPageResult>  {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Search structure: { "mangas": [...], "pagination": {...} }
	for item in json.get("mangas").as_array()? {
		let manga = item.as_object()?;

		if manga.get("slug").as_string()?.read() == "unknown" {
			continue
		}
		
		let key = manga.get("slug").as_string()?.read();
		let title = manga.get("title").as_string()?.read();
		let cover = format!("{}/{}", String::from(API_URL), manga.get("coverImage").as_string()?.read());

		mangas.push(Manga {
			key,
			title,
			cover: Some(cover),
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
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_details(manga_id: String, json: ValueRef) -> Result<Manga> {	
	let manga = json.get("manga").as_object()?;

	// Get cover image
	let cover = format!("{}/{}", API_URL, manga.get("coverImage").as_string()?.read());
	
	// Get title
	let title = manga.get("title").as_string()?.read();

	// Get description (with default value)
	let description = if manga.get("synopsis").is_some() && !manga.get("synopsis").as_string()?.read().is_empty() {
		Some(manga.get("synopsis").as_string()?.read())
	} else {
		Some("Aucune description disponible.".to_string())
	};

	// Get URL
	let url = Some(format!("{}/manga/{}", BASE_URL, manga_id));

	// Get manga status
	let status_str = manga.get("status").as_string()?.read();
	let status = match status_str.as_str() {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};

    // Get tags (genres)
	let mut tags: Vec<String> = Vec::new();
	for item in manga.get("genres").as_array()? {
		let genre = item.as_object()?;
		tags.push(genre.get("name").as_string()?.read());
	}

	// Get manga type
	let manga_type = manga.get("type").as_string()?.read();
	let viewer = match manga_type.as_str() {
		"Manga" => Viewer::Rtl,
		_ => Viewer::Scroll,
	};

	Ok(Manga {
		key: manga_id,
		title,
		cover: Some(cover),
		authors: None,
		artists: None,
		description,
		url,
		tags: Some(tags),
		status,
		content_rating: ContentRating::Safe,
		viewer,
		chapters: None,
		next_update_time: None,
		update_strategy: UpdateStrategy::Never,
	})
}

pub fn parse_chapter_list(manga_id: String, json: ValueRef) -> Result<Vec<Chapter>> {
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
		
		let key = format!("{}", chapter_number);
		let date_str = chapter_object.get("createdAt").as_string()?.read();
		let mut date_uploaded = date_str.as_date("yyyy-MM-dd'T'HH:mm:ss.SSSZ", Some("fr"), None)
			.unwrap_or(-1.0);
	
		if date_uploaded == -1.0 {
			date_uploaded = current_date();
		}
		
		let title = Some(format!("Chapter {}", chapter_number));
		let url = Some(format!("{}/manga/{}/chapitre/{}", BASE_URL, manga_id, chapter_number));

		chapters.push(Chapter{
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

pub fn parse_page_list(json: ValueRef) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();

	for item in json.get("chapter").as_object()?.get("images").as_array()? {
		let image_url = format!("{}/{}", API_URL, item.as_string()?.read());
		pages.push(Page {
			content: PageContent::url(image_url),
			thumbnail: None,
			has_description: false,
			description: None,
		});
	}

	Ok(pages)
}