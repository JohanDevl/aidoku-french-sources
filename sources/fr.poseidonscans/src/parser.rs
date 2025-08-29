use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString, vec},
	imports::{html::Document, std::current_date},
	serde::Deserialize,
};
use core::cmp::Ordering;
use serde_json;
use crate::{BASE_URL, API_URL};

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
	let mut authors: Option<Vec<String>> = None;
	let mut artists: Option<Vec<String>> = None;
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

pub fn parse_chapter_list(_manga_key: String, html: &Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut seen_chapter_ids: Vec<String> = Vec::new();

	// Try more specific selectors first, then fallback to general ones
	let chapter_selectors = [
		".chapter-list a[href*='/chapter/']",
		".chapters a[href*='/chapter/']", 
		"a[href*='/chapter/']",
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

						let mut title = String::new();
						let mut chapter_number: Option<f32> = None;

						// Try to get chapter title
						if let Some(title_text) = chapter_element.text() {
							title = title_text.trim().to_string();
						}

						// Extract chapter number
						chapter_number = extract_chapter_number_from_title(&title)
							.or_else(|| extract_chapter_number_from_id(&chapter_id));

						let url = if href_str.starts_with("http") {
							href_str
						} else {
							format!("{}{}", BASE_URL, href_str)
						};

						// Estimate date based on chapter number for better sorting
						let date_uploaded = {
							let base_date = current_date() as i64;
							if let Some(ch_num) = chapter_number {
								// Assume chapters are released weekly, subtract weeks based on chapter number
								let estimated_days_ago = (200.0 - ch_num.min(200.0)) as i64 * 7;
								base_date - (estimated_days_ago * 24 * 60 * 60)
							} else {
								base_date
							}
						};

						chapters.push(Chapter {
							key: chapter_id,
							title: Some(title),
							volume_number: None,
							chapter_number,
							date_uploaded: Some(date_uploaded),
							scanlators: None,
							url: Some(url),
							language: Some("fr".to_string()),
							thumbnail: None,
							locked: false,
						});
					}
				}
			}

			// If we found chapters with this selector, stop trying others
			if !chapters.is_empty() {
				break;
			}
		}
	}

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
	use aidoku::alloc::string::ToString;
	
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

