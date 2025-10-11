use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result,
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString, vec, collections::{BTreeMap, BTreeSet}},
	imports::html::Document,
	serde::Deserialize,
};
use core::cmp::Ordering;
use serde_json;
use crate::BASE_URL;

const PAGE_SIZE: i32 = 20;
const PAGE_SIZE_USIZE: usize = PAGE_SIZE as usize;
const CHAPTER_PREFIX_LEN: usize = 9;

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
	#[serde(default)]
	pub r#type: Option<String>,
	#[serde(default, rename = "createdAt")]
	pub created_at: Option<String>,
	#[serde(default, rename = "latestChapterCreatedAt")]
	pub latest_chapter_created_at: Option<String>,
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

pub fn parse_manga_list(
	response: String, 
	search_query: String, 
	status_filter: Option<String>, 
	type_filter: Option<String>, 
	genre_filter: Option<String>, 
	sort_filter: Option<String>, 
	page: i32
) -> Result<MangaPageResult> {
	let api_response: ApiResponse<MangaItem> = serde_json::from_str(&response)
		.map_err(|_| aidoku::AidokuError::JsonParseError)?;

	let mut all_mangas: Vec<Manga> = Vec::new();
	let query_lower = search_query.to_lowercase();

	for item in &api_response.data {
		let manga = item.to_manga();
		
		// Apply search filter
		if !search_query.is_empty() && !manga.title.to_lowercase().contains(&query_lower) {
			continue;
		}
		
		// Apply status filter
		if let Some(ref status_str) = status_filter {
			let filter_status = parse_manga_status(status_str);
			if manga.status != filter_status {
				continue;
			}
		}
		
		// Apply type filter (check if manga contains the type in description or if it's a different type)
		if let Some(ref type_str) = type_filter {
			// Extract type from the original API data if available
			let manga_type = extract_manga_type(&item);
			if !type_matches(&manga_type, type_str) {
				continue;
			}
		}
		
		// Apply genre filter
		if let Some(ref genre_str) = genre_filter {
			if !manga_has_genre(&manga, genre_str) {
				continue;
			}
		}
		
		all_mangas.push(manga);
	}
	
	// Apply sorting
	if let Some(ref sort_str) = sort_filter {
		apply_sorting(&mut all_mangas, sort_str, &api_response.data);
	}

	// Client-side pagination
	let start_index = ((page - 1) * PAGE_SIZE) as usize;
	let end_index = (start_index + PAGE_SIZE_USIZE).min(all_mangas.len());
	
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

	// Extract title from page - try multiple selectors for robustness
	// Order matters: most specific selectors first to avoid sr-only elements
	let title_selectors = [
		"h1.text-2xl.font-bold.text-white",  // Visible h1 with specific classes from PoseidonScans
		"h1.font-bold.text-white",            // Alternative visible h1 pattern
		"h1:not(.sr-only):not([class*=\"sr-only\"])",  // Any h1 that's not screen-reader-only
		"[data-testid=\"manga-title\"]",
		".manga-title",
		"h1.entry-title",
		"meta[property=\"og:title\"]",
		"title"
	];

	let mut title_found = false;
	for selector in &title_selectors {
		if let Some(title_element) = html.select(selector).and_then(|els| els.first()) {
			let mut title_text = if selector.contains("meta") {
				title_element.attr("content").map(|s| s.to_string()).unwrap_or_default()
			} else if *selector == "title" {
				// Extract from title tag, removing site name suffix
				title_element.text().unwrap_or_default()
					.replace(" | Poseidon Scans", "")
					.replace("Lire ", "")
					.replace(" scan VF / FR gratuit en ligne", "")
					.trim()
					.to_string()
			} else {
				title_element.text().unwrap_or_default().trim().to_string()
			};

			// Clean up any HTML comments or problematic content
			title_text = title_text
				.replace("<!-- -->", "")  // Remove HTML comments
				.replace("Lire ", "")      // Remove "Lire" prefix if still present
				.replace(" scan VF / FR gratuit en ligne", "")  // Remove suffix
				.trim()
				.to_string();

			// Validate the title is not a placeholder or empty
			if !title_text.is_empty()
				&& title_text != manga_key
				&& !title_text.starts_with("[Image")
				&& !title_text.contains("#")
				&& title_text.len() > 2 {
				title = title_text;
				title_found = true;
				break;
			}
		}
	}

	// Try to extract from JSON-LD as additional fallback
	if !title_found {
		if let Ok(manga_data) = extract_jsonld_manga_details(html) {
			if let Some(title_text) = manga_data.get("name").and_then(|t| t.as_str()) {
				if !title_text.is_empty() && title_text != manga_key {
					title = title_text.to_string();
				}
			}
		}
	}

	// Extract description
	if let Some(desc_text) = html.select("p.text-gray-300.leading-relaxed.whitespace-pre-line").and_then(|els| els.first()).and_then(|el| el.text()) {
		let desc = desc_text.trim().to_string();
		if !desc.is_empty() && desc != "Aucune description." {
			description = desc;
		}
	}

	// Extract tags from JSON-LD and status from HTML
	let mut tag_list: Vec<String> = Vec::new();
	
	// Extract genres from JSON-LD (most reliable source)
	if let Ok(manga_data) = extract_jsonld_manga_details(html) {
		if let Some(genres) = manga_data.get("genre").and_then(|g| g.as_array()) {
			for genre in genres {
				if let Some(genre_str) = genre.as_str() {
					let genre_clean = genre_str.trim().to_string();
					if !genre_clean.is_empty() {
						tag_list.push(genre_clean);
					}
				}
			}
		}
	}
	
	// Extract status from HTML and add as tag
	let mut status_found = false;

	// Method 1: Look for status paragraph with "Status:" text
	if let Some(status_elements) = html.select("p") {
		for status_element in status_elements {
			if let Some(status_html) = status_element.html() {
				if status_html.contains("Status:") {
					if let Some(status_text) = status_element.text() {
						let status_str = status_text.replace("Status:", "").trim().to_string();
						if !status_str.is_empty() {
							status = parse_manga_status(&status_str);
							status_found = true;
							break;
						}
					}
				}
			}
		}
	}
	
	// Method 2: Look for status badge/span (green=en cours, red=terminé, yellow=en pause)
	if !status_found {
		if let Some(status_spans) = html.select("span.bg-green-500\\/20, span.bg-red-500\\/20, span.bg-yellow-500\\/20") {
			for status_span in status_spans {
				if let Some(status_text) = status_span.text() {
					let status_str = status_text.trim().to_string();
					if status_str == "en cours" || status_str == "terminé" || status_str == "en pause" {
						status = parse_manga_status(&status_str);
						status_found = true;
						break;
					}
				}
			}
		}
	}
	
	// Fallback: look for any span with status-like content
	if !status_found {
		if let Some(status_spans) = html.select("span") {
			for status_span in status_spans {
				if let Some(status_text) = status_span.text() {
					let status_str = status_text.trim().to_string();
					if status_str == "en cours" || status_str == "terminé" || status_str == "en pause" {
						status = parse_manga_status(&status_str);
						break;
					}
				}
			}
		}
	}
	
	if !tag_list.is_empty() {
		tags = Some(tag_list);
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

// JSON-LD extraction - the ACTUAL approach PoseidonScans uses
fn extract_jsonld_manga_details(html: &Document) -> Result<serde_json::Value> {
	// Look for JSON-LD scripts with type="application/ld+json"
	if let Some(script_elements) = html.select("script[type=\"application/ld+json\"]") {
		for script in script_elements {
			if let Some(content) = script.data() {
				if !content.trim().is_empty() {
					if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
						// Check if this is a ComicSeries (manga) JSON-LD
						if let Some(type_value) = json_data.get("@type") {
							if let Some(type_str) = type_value.as_str() {
								if type_str == "ComicSeries" {
									return Ok(json_data);
								}
							}
						}
					}
				}
			}
		}
	}

	Ok(serde_json::json!({}))
}

// Detect premium chapter IDs from __NEXT_DATA__ or HTML
// Returns a set of chapter IDs that are premium
fn detect_premium_chapters_from_html(html: &Document) -> BTreeSet<String> {
	let mut premium_ids = BTreeSet::new();

	// Method 1: Try to extract from __NEXT_DATA__ (Next.js hydration data)
	if let Some(script_elements) = html.select("script#__NEXT_DATA__") {
		for script in script_elements {
			if let Some(content) = script.data() {
				if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
					// Try to navigate to chapters data
					// Possible paths: props.pageProps.chapters, props.pageProps.initialData.chapters, etc.
					let possible_paths = [
						&json_data["props"]["pageProps"]["chapters"],
						&json_data["props"]["pageProps"]["initialData"]["chapters"],
						&json_data["props"]["pageProps"]["manga"]["chapters"],
						&json_data["pageProps"]["chapters"],
					];

					for chapters_data in &possible_paths {
						if let Some(chapters_array) = chapters_data.as_array() {
							for chapter in chapters_array.iter() {
								// Look for premium indicators in chapter data
								let is_premium = chapter.get("isPremium")
									.and_then(|v| v.as_bool())
									.unwrap_or(false)
									|| chapter.get("premium")
									.and_then(|v| v.as_bool())
									.unwrap_or(false)
									|| chapter.get("locked")
									.and_then(|v| v.as_bool())
									.unwrap_or(false);

								if is_premium {
									// Try to get chapter number or ID
									let chapter_id = chapter.get("number")
										.and_then(|v| v.as_i64())
										.map(|n| format!("{}", n))
										.or_else(|| chapter.get("id").and_then(|v| v.as_str()).map(|s| s.to_string()))
										.or_else(|| chapter.get("chapterNumber").and_then(|v| v.as_i64()).map(|n| format!("{}", n)));

									if let Some(id) = chapter_id {
										premium_ids.insert(id);
									}
								}
							}

							if !premium_ids.is_empty() {
								return premium_ids;
							}
						}
					}
				}
			}
		}
	}

	// Method 2: Fallback to HTML parsing (won't work for client-rendered content)
	if let Some(chapter_links) = html.select("a[href*='/chapter/']") {
		for link in chapter_links {
			let chapter_id = if let Some(href) = link.attr("href") {
				extract_chapter_id_from_url(&href)
			} else {
				None
			};

			let class_attr_str = link.attr("class").unwrap_or_default();
			let has_amber_class = class_attr_str.contains("amber") || class_attr_str.contains("border-amber-500");

			let html_content_str = link.html().unwrap_or_default();
			let html_lower = html_content_str.to_lowercase();
			let has_premium_text = html_lower.contains("premium") || html_lower.contains("accès anticipé");

			let text_content_str = link.text().unwrap_or_default();
			let has_premium_in_text = text_content_str.to_uppercase().contains("PREMIUM");

			if has_amber_class || has_premium_text || has_premium_in_text {
				if let Some(chapter_id) = chapter_id {
					premium_ids.insert(chapter_id);
				}
			}
		}
	}

	premium_ids
}

pub fn parse_chapter_list(_manga_key: String, html: &Document) -> Result<Vec<Chapter>> {
	// Extract JSON-LD data using the ACTUAL approach PoseidonScans uses
	let manga_data = extract_jsonld_manga_details(html)?;

	// Extract chapters from JSON-LD "hasPart" array
	let chapters_array = if let Some(has_part) = manga_data.get("hasPart").and_then(|c| c.as_array()) {
		has_part
	} else {
		return Ok(parse_chapter_list_from_html(html)?);
	};

	// Get premium chapter IDs from HTML (O(1) parse, no HTTP requests)
	let premium_chapter_ids = detect_premium_chapters_from_html(html);

	let mut chapters: Vec<Chapter> = Vec::new();

	// Parse each ComicIssue from JSON-LD
	for chapter_value in chapters_array {
		if let Some(chapter_obj) = chapter_value.as_object() {
			// Check if this is a ComicIssue
			if let Some(type_value) = chapter_obj.get("@type") {
				if let Some(type_str) = type_value.as_str() {
					if type_str != "ComicIssue" {
						continue; // Skip non-ComicIssue entries
					}
				}
			}

			// Extract chapter number from issueNumber
			let chapter_number = if let Some(num) = chapter_obj.get("issueNumber") {
				if let Some(n) = num.as_f64() {
					n as f32
				} else if let Some(n) = num.as_i64() {
					n as f32
				} else {
					continue;
				}
			} else {
				continue;
			};

			// Extract chapter title - clean format: "Chapitre X"
			let chapter_title = format!("Chapitre {}", chapter_number);

			// Extract chapter URL directly from JSON-LD (already complete)
			let url = chapter_obj.get("url")
				.and_then(|u| u.as_str())
				.unwrap_or_default()
				.to_string();

			// Extract chapter ID from URL
			let chapter_id = if chapter_number == (chapter_number as i32) as f32 {
				format!("{}", chapter_number as i32)
			} else {
				format!("{}", chapter_number)
			};

			// Skip if this chapter is premium (using the pre-computed set)
			if premium_chapter_ids.contains(&chapter_id) {
				continue;
			}

			chapters.push(Chapter {
				key: chapter_id,
				title: Some(chapter_title),
				volume_number: None,
				chapter_number: Some(chapter_number),
				date_uploaded: None,
				scanlators: None,
				url: Some(url),
				language: Some("fr".to_string()),
				thumbnail: None,
				locked: false,
			});
		}
	}

	// Sort chapters by number in descending order (newest first)
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

fn parse_chapter_list_from_html(html: &Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let mut seen_chapter_ids: BTreeSet<String> = BTreeSet::new();

	// Use the specific PoseidonScans chapter list structure
	// Chapters are in <a> elements with href containing /chapter/
	let chapter_selector = "a[href*='/chapter/']";
	
	if let Some(chapter_elements) = html.select(chapter_selector) {
		for chapter_element in chapter_elements {
			if let Some(href_str) = chapter_element.attr("href") {
				// Extract chapter ID from URL
				if let Some(chapter_id) = extract_chapter_id_from_url(&href_str) {
					// Skip duplicates - O(1) lookup with HashSet
					if !seen_chapter_ids.insert(chapter_id.clone()) {
						continue; // insert() returns false if element was already present
					}

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

pub fn parse_page_list(html: &Document, _chapter_url: String) -> Result<Vec<Page>> {
	// Try JSON-LD extraction first (like for chapters)
	if let Ok(pages) = extract_pages_from_jsonld(html) {
		if !pages.is_empty() {
			return Ok(pages);
		}
	}
	
	// Try __NEXT_DATA__ extraction as backup
	if let Ok(pages) = extract_pages_from_nextdata(html) {
		if !pages.is_empty() {
			return Ok(pages);
		}
	}
	
	// Fallback to HTML extraction
	extract_pages_from_html(html)
}

// Extract pages from JSON-LD (schema.org structured data)
fn extract_pages_from_jsonld(html: &Document) -> Result<Vec<Page>> {
	if let Some(script_elements) = html.select("script[type=\"application/ld+json\"]") {
		for script in script_elements {
			if let Some(content) = script.data() {
				if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
					// Look for chapter-specific JSON-LD with images
					if let Some(type_value) = json_data.get("@type") {
						if let Some(type_str) = type_value.as_str() {
							if type_str == "ComicIssue" || type_str == "Chapter" {
								if let Some(images) = json_data.get("images").and_then(|i| i.as_array()) {
									return parse_images_from_json_array(images);
								}
							}
						}
					}
				}
			}
		}
	}
	
	Ok(Vec::new())
}

// Extract pages from __NEXT_DATA__ script tag (backup method)
fn extract_pages_from_nextdata(html: &Document) -> Result<Vec<Page>> {
	if let Some(script_elements) = html.select("script#__NEXT_DATA__") {
		for script in script_elements {
			if let Some(content) = script.data() {
				if let Ok(json_data) = serde_json::from_str::<serde_json::Value>(&content) {
					// Try different paths to find images
					if let Some(props) = json_data.get("props") {
						if let Some(page_props) = props.get("pageProps") {
							if let Some(initial_data) = page_props.get("initialData") {
								if let Some(images) = initial_data.get("images").and_then(|i| i.as_array()) {
									let pages = parse_images_from_json_array(images)?;
									if !pages.is_empty() {
										return Ok(pages);
									}
								}
							}
							if let Some(images) = page_props.get("images").and_then(|i| i.as_array()) {
								let pages = parse_images_from_json_array(images)?;
								if !pages.is_empty() {
									return Ok(pages);
								}
							}
							if let Some(chapter) = page_props.get("chapter") {
								if let Some(images) = chapter.get("images").and_then(|i| i.as_array()) {
									let pages = parse_images_from_json_array(images)?;
									if !pages.is_empty() {
										return Ok(pages);
									}
								}
							}
						}
					}
					// Direct paths
					if let Some(images) = json_data.get("images").and_then(|i| i.as_array()) {
						let pages = parse_images_from_json_array(images)?;
						if !pages.is_empty() {
							return Ok(pages);
						}
					}
					if let Some(chapter) = json_data.get("chapter") {
						if let Some(images) = chapter.get("images").and_then(|i| i.as_array()) {
							let pages = parse_images_from_json_array(images)?;
							if !pages.is_empty() {
								return Ok(pages);
							}
						}
					}
				}
			}
		}
	}
	
	Ok(Vec::new())
}

// Extract pages from HTML as fallback
fn extract_pages_from_html(html: &Document) -> Result<Vec<Page>> {
	let mut pages: Vec<(usize, Page)> = Vec::new(); // Store with order for sorting

	// First try the new PoseidonScans structure with API endpoints
	if let Some(img_elements) = html.select("img[src*='/api/chapters']") {
		for img_element in img_elements {
			if let Some(src) = img_element.attr("src") {
				if !src.is_empty() && !src.contains("placeholder") && !src.contains("loading") {
					let absolute_url = if src.starts_with("/") {
						format!("{}{}", BASE_URL, src)
					} else {
						src.to_string()
					};

					// Get order from parent div's data-order attribute
					let mut order = 0;
					if let Some(parent) = img_element.parent() {
						// Look for data-order attribute in parent or parent's parent
						let parent_order = parent.attr("data-order");
						if let Some(order_str) = parent_order {
							order = order_str.parse().unwrap_or(0);
						} else if let Some(grandparent) = parent.parent() {
							if let Some(order_str) = grandparent.attr("data-order") {
								order = order_str.parse().unwrap_or(0);
							}
						}
					}

					pages.push((order, Page {
						content: PageContent::url(absolute_url),
						thumbnail: None,
						has_description: false,
						description: None,
					}));
				}
			}
		}

		// Sort by order and return
		if !pages.is_empty() {
			pages.sort_by(|a, b| a.0.cmp(&b.0));
			let ordered_pages: Vec<Page> = pages.into_iter().map(|(_, page)| page).collect();
			return Ok(ordered_pages);
		}
	}

	// Fallback to old selectors if new structure not found
	let mut fallback_pages: Vec<Page> = Vec::new();
	let image_selectors = [
		"img[alt*='Chapter Image']",
		"img[src*='/chapter/']", 
		"img[src*='/images/']",
		"img[data-src]",
		"main img",
		".chapter-content img",
		".manga-reader img",
		"img[src*='poseidon']", // PoseidonScans specific
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

						fallback_pages.push(Page {
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
		if !fallback_pages.is_empty() {
			break;
		}
	}

	Ok(fallback_pages)
}

// Parse images from JSON array (common for both JSON-LD and __NEXT_DATA__)
fn parse_images_from_json_array(images_array: &Vec<serde_json::Value>) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	for image_value in images_array.iter() {
		if let Some(image_obj) = image_value.as_object() {
			// Try different possible image URL fields
			let image_url = image_obj.get("url")
				.or_else(|| image_obj.get("src"))
				.or_else(|| image_obj.get("original"))
				.or_else(|| image_obj.get("originalUrl"))
				.and_then(|u| u.as_str());
			
			if let Some(url) = image_url {
				let absolute_url = if url.starts_with("http") {
					url.to_string()
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
		} else if let Some(url_str) = image_value.as_str() {
			// Sometimes images are just strings
			let absolute_url = if url_str.starts_with("http") {
				url_str.to_string()
			} else if url_str.starts_with("/") {
				format!("{}{}", BASE_URL, url_str)
			} else {
				format!("{}/{}", BASE_URL, url_str)
			};
			
			pages.push(Page {
				content: PageContent::url(absolute_url),
				thumbnail: None,
				has_description: false,
				description: None,
			});
		}
	}
	
	Ok(pages)
}

// Helper functions for filtering

// Extract manga type from API data (Manga, Manhwa, Manhua)
fn extract_manga_type(item: &MangaItem) -> String {
	// Use the direct type field from API
	if let Some(ref manga_type) = item.r#type {
		match manga_type.to_uppercase().as_str() {
			"MANHWA" => "Manhwa".to_string(),
			"MANHUA" => "Manhua".to_string(),
			"MANGA" => "Manga".to_string(),
			_ => "Manga".to_string(),
		}
	} else {
		// Fallback to default
		"Manga".to_string()
	}
}

// Check if manga type matches filter
fn type_matches(manga_type: &str, filter_type: &str) -> bool {
	manga_type.to_lowercase() == filter_type.to_lowercase()
}

// Check if manga has the specified genre
fn manga_has_genre(manga: &Manga, genre_filter: &str) -> bool {
	if let Some(ref tags) = manga.tags {
		let genre_lower = genre_filter.to_lowercase();
		for tag in tags {
			let tag_lower = tag.to_lowercase();
			// Exact match or contains match
			if tag_lower == genre_lower || tag_lower.contains(&genre_lower) || genre_lower.contains(&tag_lower) {
				return true;
			}
		}
	}
	false
}

// Apply sorting to manga list
fn apply_sorting(mangas: &mut Vec<Manga>, sort_option: &str, manga_items: &[MangaItem]) {
	// Create a map for quick lookup of API data by manga key (slug)
	let item_map: BTreeMap<&str, &MangaItem> = manga_items
		.iter()
		.map(|item| (item.slug.as_str(), item))
		.collect();

	match sort_option {
		"Titre A-Z" => {
			mangas.sort_by(|a, b| a.title.to_lowercase().cmp(&b.title.to_lowercase()));
		}
		"Titre Z-A" => {
			mangas.sort_by(|a, b| b.title.to_lowercase().cmp(&a.title.to_lowercase()));
		}
		"Date d'ajout" => {
			// Sort by createdAt date (newest first)
			mangas.sort_by(|a, b| {
				let a_date = item_map.get(a.key.as_str()).and_then(|item| item.created_at.as_ref());
				let b_date = item_map.get(b.key.as_str()).and_then(|item| item.created_at.as_ref());
				
				match (a_date, b_date) {
					(Some(a_str), Some(b_str)) => b_str.cmp(a_str), // Reverse for newest first
					(Some(_), None) => Ordering::Less,
					(None, Some(_)) => Ordering::Greater,
					(None, None) => Ordering::Equal,
				}
			});
		}
		"Dernière mise à jour" | _ => {
			// Sort by latestChapterCreatedAt date (newest first) or keep API order
			mangas.sort_by(|a, b| {
				let a_date = item_map.get(a.key.as_str()).and_then(|item| item.latest_chapter_created_at.as_ref());
				let b_date = item_map.get(b.key.as_str()).and_then(|item| item.latest_chapter_created_at.as_ref());
				
				match (a_date, b_date) {
					(Some(a_str), Some(b_str)) => b_str.cmp(a_str), // Reverse for newest first
					(Some(_), None) => Ordering::Less,
					(None, Some(_)) => Ordering::Greater,
					(None, None) => Ordering::Equal,
				}
			});
		}
	}
}

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
		let after_chapter = &url[chapter_pos + CHAPTER_PREFIX_LEN..];
		if let Some(end_pos) = after_chapter.find('?').or_else(|| after_chapter.find('#')) {
			Some(after_chapter[..end_pos].to_string())
		} else {
			Some(after_chapter.to_string())
		}
	} else {
		None
	}
}

fn extract_chapter_number_from_id(chapter_id: &str) -> Option<f32> {
	// Try to parse chapter ID as number
	chapter_id.parse::<f32>().ok()
}