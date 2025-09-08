use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString},
	imports::std::current_date,
	serde::Deserialize,
};

use chrono::DateTime;

use serde_json;

use crate::BASE_URL;
use crate::API_URL;

// Serde structures for PhenixScans API
#[derive(Deserialize, Debug)]
struct MangaItem {
	#[serde(rename = "_id", default)]
	id: Option<String>,
	#[serde(default)]
	slug: Option<String>,
	title: String,
	#[serde(rename = "coverImage")]
	cover_image: String,
	#[serde(default)]
	status: Option<String>,
	#[serde(rename = "type", default)]
	manga_type: Option<String>,
}

#[derive(Deserialize, Debug)]
struct Pagination {
	#[serde(rename = "currentPage", default)]
	current_page: Option<i32>,
	#[serde(rename = "totalPages", default)]
	total_pages: Option<i32>,
	#[serde(default)]
	page: Option<i32>,
	#[serde(rename = "hasNextPage", default)]
	has_next_page: Option<bool>,
}

#[derive(Deserialize, Debug)]
struct ListingResponse {
	#[serde(default)]
	top: Option<Vec<MangaItem>>,
	#[serde(default)]
	latest: Option<Vec<MangaItem>>,
	#[serde(default)]
	pagination: Option<Pagination>,
}

#[derive(Deserialize, Debug)]
struct MangaListResponse {
	mangas: Vec<MangaItem>,
	#[serde(default)]
	pagination: Option<Pagination>,
}

#[derive(Deserialize, Debug)]
struct Genre {
	name: String,
}

#[derive(Deserialize, Debug)]
struct MangaDetails {
	title: String,
	#[serde(rename = "coverImage")]
	cover_image: String,
	#[serde(default)]
	synopsis: Option<String>,
	status: String,
	#[serde(rename = "type")]
	manga_type: String,
	#[serde(default)]
	genres: Option<Vec<Genre>>,
}

#[derive(Deserialize, Debug)]
struct MangaDetailsResponse {
	manga: MangaDetails,
}

#[derive(Deserialize, Debug)]
struct ChapterItem {
	number: serde_json::Value, // Can be int or float
	#[serde(default)]
	price: Option<i32>,
	#[serde(rename = "createdAt", default)]
	created_at: Option<String>,
}

#[derive(Deserialize, Debug)]
struct ChapterListResponse {
	chapters: Vec<ChapterItem>,
}

#[derive(Deserialize, Debug)]
struct ChapterDetails {
	images: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct PageListResponse {
	chapter: ChapterDetails,
}

fn parse_manga_status(status_str: &str) -> MangaStatus {
	match status_str {
		"Ongoing" => MangaStatus::Ongoing,
		"Completed" => MangaStatus::Completed,
		"Hiatus" => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	}
}

fn extract_chapter_number(value: &serde_json::Value) -> f32 {
	match value {
		serde_json::Value::Number(n) => {
			if let Some(f) = n.as_f64() {
				f as f32
			} else if let Some(i) = n.as_i64() {
				i as f32
			} else {
				1.0
			}
		}
		serde_json::Value::String(s) => s.parse::<f32>().unwrap_or(1.0),
		_ => 1.0,
	}
}

impl MangaItem {
	fn get_key(&self) -> Option<String> {
		if let Some(slug) = &self.slug {
			if slug != "unknown" {
				return Some(slug.clone());
			}
		}
		self.id.clone()
	}

	fn to_manga(&self) -> Option<Manga> {
		let key = self.get_key()?;
		let cover = if !self.cover_image.is_empty() {
			Some(format!("{}/{}", API_URL, self.cover_image))
		} else {
			None
		};

		let status = if let Some(status_str) = &self.status {
			parse_manga_status(status_str)
		} else {
			MangaStatus::Unknown
		};

		let viewer = if let Some(manga_type) = &self.manga_type {
			if manga_type == "Manga" {
				Viewer::RightToLeft
			} else {
				Viewer::Vertical
			}
		} else {
			Viewer::Vertical
		};

		Some(Manga {
			key,
			title: self.title.clone(),
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
		})
	}
}

pub fn parse_manga_listing(response: String, listing_type: &str) -> Result<MangaPageResult> {
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner un résultat vide au lieu d'échouer
		return Ok(MangaPageResult {
			entries: Vec::new(),
			has_next_page: false,
		});
	}

	let listing_data: ListingResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner un résultat vide
			return Ok(MangaPageResult {
				entries: Vec::new(),
				has_next_page: false,
			});
		}
	};

	let mut mangas: Vec<Manga> = Vec::new();

	let has_more = if listing_type == "Populaire" {
		// For the "top" section
		if let Some(top_items) = listing_data.top {
			for item in top_items {
				if let Some(manga) = item.to_manga() {
					mangas.push(manga);
				}
			}
		}
		false // Top section has no pagination
	} else {
		// For the "latest" section
		if let Some(latest_items) = listing_data.latest {
			for item in latest_items {
				if let Some(manga) = item.to_manga() {
					mangas.push(manga);
				}
			}
		}
		
		// Check pagination
		if let Some(pagination) = listing_data.pagination {
			let current = pagination.current_page.unwrap_or(1);
			let total = pagination.total_pages.unwrap_or(1);
			current < total
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
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner un résultat vide au lieu d'échouer
		return Ok(MangaPageResult {
			entries: Vec::new(),
			has_next_page: false,
		});
	}

	let manga_data: MangaListResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner un résultat vide
			return Ok(MangaPageResult {
				entries: Vec::new(),
				has_next_page: false,
			});
		}
	};

	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in manga_data.mangas {
		if let Some(manga) = item.to_manga() {
			mangas.push(manga);
		}
	}

	// Check pagination
	let has_more = if let Some(pagination) = manga_data.pagination {
		if let Some(has_next) = pagination.has_next_page {
			has_next
		} else {
			let current = pagination.page.unwrap_or(1);
			let total = pagination.total_pages.unwrap_or(1);
			current < total
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
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner un résultat vide au lieu d'échouer
		return Ok(MangaPageResult {
			entries: Vec::new(),
			has_next_page: false,
			});
	}

	let search_data: MangaListResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner un résultat vide
			return Ok(MangaPageResult {
				entries: Vec::new(),
				has_next_page: false,
			});
		}
	};

	let mut mangas: Vec<Manga> = Vec::new();
	
	for item in search_data.mangas {
		if let Some(manga) = item.to_manga() {
			mangas.push(manga);
		}
	}

	// Check pagination for searches
	let has_more = if let Some(pagination) = search_data.pagination {
		let current = pagination.page.unwrap_or(0);
		let total = pagination.total_pages.unwrap_or(0);
		current < total
	} else {
		false
	};

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: has_more,
	})
}

pub fn parse_manga_details(manga_id: String, response: String) -> Result<Manga> {
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner un manga minimal au lieu d'échouer
		return Ok(Manga {
			key: manga_id.clone(),
			title: "Titre indisponible".to_string(),
			cover: None,
			authors: None,
			artists: None,
			description: Some("Détails temporairement indisponibles.".to_string()),
			url: Some(format!("{}/manga/{}", BASE_URL, manga_id)),
			tags: None,
			status: MangaStatus::Unknown,
			content_rating: ContentRating::Safe,
			viewer: Viewer::Vertical,
			chapters: None,
			next_update_time: None,
			update_strategy: UpdateStrategy::Never,
		});
	}

	let details_data: MangaDetailsResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner un manga minimal
			return Ok(Manga {
				key: manga_id.clone(),
				title: "Titre indisponible".to_string(),
				cover: None,
				authors: None,
				artists: None,
				description: Some("Détails temporairement indisponibles.".to_string()),
				url: Some(format!("{}/manga/{}", BASE_URL, manga_id)),
				tags: None,
				status: MangaStatus::Unknown,
				content_rating: ContentRating::Safe,
				viewer: Viewer::Vertical,
				chapters: None,
				next_update_time: None,
				update_strategy: UpdateStrategy::Never,
			});
		}
	};

	let manga_details = details_data.manga;

	// Get cover image
	let cover = Some(format!("{}/{}", API_URL, manga_details.cover_image));
	
	// Get description (with default value)
	let description = if let Some(synopsis) = manga_details.synopsis {
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
	let status = parse_manga_status(&manga_details.status);

	// Get tags (genres)
	let tags = if let Some(genres) = manga_details.genres {
		let genre_names: Vec<String> = genres.into_iter().map(|g| g.name).collect();
		if genre_names.is_empty() { None } else { Some(genre_names) }
	} else {
		None
	};

	// Get manga type
	let viewer = if manga_details.manga_type == "Manga" {
		Viewer::RightToLeft
	} else {
		Viewer::Vertical
	};

	Ok(Manga {
		key: manga_id,
		title: manga_details.title,
		cover,
		authors: None,
		artists: None,
		description,
		url,
		tags,
		status,
		content_rating: ContentRating::Safe,
		viewer,
		chapters: None,
		next_update_time: None,
		update_strategy: UpdateStrategy::Never,
	})
}

pub fn parse_chapter_list(manga_id: String, response: String) -> Result<Vec<Chapter>> {
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner une liste vide au lieu d'échouer
		return Ok(Vec::new());
	}

	let chapters_data: ChapterListResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner une liste vide
			return Ok(Vec::new());
		}
	};

	let mut chapters: Vec<Chapter> = Vec::new();
	
	for item in chapters_data.chapters {
		// Check price - only take free chapters (price == 0)
		let price = item.price.unwrap_or(0);
		if price != 0 {
			continue;
		}
		
		// Chapter number can be an integer or a float/string
		let chapter_number = extract_chapter_number(&item.number);
		
		let key = format!("{}", chapter_number);
		let title = Some(format!("Chapitre {}", chapter_number));
		let url = Some(format!("{}/manga/{}/chapitre/{}", BASE_URL, manga_id, chapter_number));

		// Parse date if available (using chrono like modern sources)
		let date_uploaded = if let Some(date_str) = &item.created_at {
			DateTime::parse_from_rfc3339(date_str)
				.ok()
				.map(|d| d.timestamp())
				.or_else(|| Some(current_date()))
		} else {
			Some(current_date())
		};

		chapters.push(Chapter {
			key,
			title,
			volume_number: None, // Remove volume number since it's not used
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
	// Vérifier si la réponse ressemble à du HTML ou une erreur
	if response.trim_start().starts_with('<') || response.contains("403 Forbidden") || response.contains("Access Denied") {
		// Retourner une liste vide au lieu d'échouer
		return Ok(Vec::new());
	}

	let pages_data: PageListResponse = match serde_json::from_str(&response) {
		Ok(data) => data,
		Err(_) => {
			// Si le parsing JSON échoue, retourner une liste vide
			return Ok(Vec::new());
		}
	};

	let mut pages: Vec<Page> = Vec::new();

	for image_path in pages_data.chapter.images {
		let image_url = format!("{}/{}", API_URL, image_path);
		pages.push(Page {
			content: PageContent::url(image_url),
			thumbnail: None,
			has_description: false,
			description: None,
		});
	}

	Ok(pages)
}