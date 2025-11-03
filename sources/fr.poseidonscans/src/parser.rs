use crate::BASE_URL;
use aidoku::{
	alloc::{
		collections::BTreeSet,
		format,
		string::ToString,
		vec, String, Vec,
	},
	imports::html::Document,
	serde::Deserialize,
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result,
	UpdateStrategy, Viewer,
};
use chrono::{DateTime, NaiveDateTime};
use core::cmp::Ordering;
use serde_json;

const CHAPTER_PREFIX: &str = "/chapter/";
const CHAPTER_PREFIX_LEN: usize = CHAPTER_PREFIX.len();

fn calculate_content_rating(tags: &Option<Vec<String>>) -> ContentRating {
	if let Some(tags) = tags {
		for tag in tags {
			let tag_lower = tag.to_lowercase();
			match tag_lower.as_str() {
				"adult" | "adulte" | "mature" | "hentai" | "smut" | "érotique" => {
					return ContentRating::NSFW;
				}
				"ecchi" | "suggestif" | "suggestive" => {
					return ContentRating::Suggestive;
				}
				_ => {}
			}
		}
	}
	ContentRating::Safe
}

fn calculate_viewer(tags: &Option<Vec<String>>) -> Viewer {
	if let Some(tags) = tags {
		for tag in tags {
			let tag_lower = tag.to_lowercase();
			match tag_lower.as_str() {
				"manhwa" | "manhua" | "webtoon" | "scroll" | "vertical" => {
					return Viewer::Vertical;
				}
				"manga" => {
					return Viewer::RightToLeft;
				}
				_ => {}
			}
		}
	}
	Viewer::RightToLeft
}

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

		let authors = self
			.author
			.as_ref()
			.map(|a| vec![a.clone()])
			.filter(|a| !a.is_empty());

		let artists = self
			.artist
			.as_ref()
			.map(|a| vec![a.clone()])
			.filter(|a| !a.is_empty());

		let tags = self
			.categories
			.as_ref()
			.map(|cats| cats.iter().map(|c| c.name.clone()).collect::<Vec<_>>())
			.filter(|t| !t.is_empty());

		let status = self
			.status
			.as_ref()
			.map(|s| parse_manga_status(s))
			.unwrap_or(MangaStatus::Unknown);

		let description = self
			.description
			.clone()
			.filter(|d| !d.is_empty() && d != "Aucune description.");

		let content_rating = calculate_content_rating(&tags);
		let viewer = calculate_viewer(&tags);

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
			content_rating,
			viewer,
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

pub fn parse_latest_manga(response: String) -> Result<MangaPageResult> {
	let api_response: LatestChapterResponse = match serde_json::from_str(&response) {
		Ok(resp) => resp,
		Err(_) => return Ok(MangaPageResult {
			entries: Vec::new(),
			has_next_page: false,
		}),
	};

	let mut mangas: Vec<Manga> = Vec::new();

	for item in api_response.data {
		let manga = item.to_manga();
		mangas.push(manga);
	}

	let has_next_page = api_response
		.pagination
		.and_then(|p| p.has_more)
		.unwrap_or(false);

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page,
	})
}

pub fn parse_popular_manga(response: String) -> Result<MangaPageResult> {
	let api_response: ApiResponse<MangaItem> = match serde_json::from_str(&response) {
		Ok(resp) => resp,
		Err(_) => return Ok(MangaPageResult {
			entries: Vec::new(),
			has_next_page: false,
		}),
	};

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
	let artists: Option<Vec<String>> = None;
	let mut tags: Option<Vec<String>> = None;
	let mut status = MangaStatus::Unknown;

	// Extract title from page - try multiple selectors for robustness
	// Order matters: most specific selectors first to avoid sr-only elements
	let title_selectors = [
		"h1.text-2xl.font-bold.text-white", // Visible h1 with specific classes from PoseidonScans
		"h1.font-bold.text-white",          // Alternative visible h1 pattern
		"h1:not(.sr-only):not([class*=\"sr-only\"])", // Any h1 that's not screen-reader-only
		"[data-testid=\"manga-title\"]",
		".manga-title",
		"h1.entry-title",
		"meta[property=\"og:title\"]",
		"title",
	];

	let mut title_found = false;
	for selector in &title_selectors {
		if let Some(title_element) = html.select(selector).and_then(|els| els.first()) {
			let mut title_text = if selector.contains("meta") {
				title_element
					.attr("content")
					.map(|s| s.to_string())
					.unwrap_or_default()
			} else if *selector == "title" {
				// Extract from title tag, removing site name suffix
				title_element
					.text()
					.unwrap_or_default()
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
				&& title_text.len() > 2
			{
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
	if let Some(desc_text) = html
		.select("p.text-gray-300.leading-relaxed.whitespace-pre-line")
		.and_then(|els| els.first())
		.and_then(|el| el.text())
	{
		let desc = desc_text.trim().to_string();
		if !desc.is_empty() && desc != "Aucune description." {
			description = desc;
		}
	}

	// Extract author from HTML
	if let Some(span_elements) = html.select("span") {
		for span_element in span_elements {
			if let Some(span_html) = span_element.html() {
				if span_html.contains("Auteur:") {
					// Find the nested bold span with the author name
					if let Some(author_span) = span_element.select("span.text-gray-300.font-bold").and_then(|els| els.first()) {
						if let Some(author_text) = author_span.text() {
							let author = author_text.trim().to_string();
							if !author.is_empty() {
								authors = Some(vec![author]);
								break;
							}
						}
					}
				}
			}
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

	// Method 2: Look for status badge/span (green=en cours, red=terminé, yellow=en
	if !status_found {
		if let Some(status_spans) =
			html.select("span.bg-green-500\\/20, span.bg-red-500\\/20, span.bg-yellow-500\\/20")
		{
			for status_span in status_spans {
				if let Some(status_text) = status_span.text() {
					let status_str = status_text.trim().to_string();
					if status_str == "en cours"
						|| status_str == "terminé"
						|| status_str == "en pause"
					{
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
					if status_str == "en cours"
						|| status_str == "terminé"
						|| status_str == "en pause"
					{
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

	let content_rating = calculate_content_rating(&tags);
	let viewer = calculate_viewer(&tags);

	Ok(Manga {
		key: manga_key.clone(),
		title,
		cover: Some(cover),
		authors,
		artists,
		description: if description.is_empty() {
			None
		} else {
			Some(description)
		},
		url: Some(format!("{}/serie/{}", BASE_URL, manga_key)),
		tags,
		status,
		content_rating,
		viewer,
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

// Parse ISO 8601 date string to Unix timestamp
// Format: "2025-10-11T13:49:51.543Z" or "$D2025-10-11T13:49:51.543Z"
fn parse_iso_date(date_str: &str) -> Option<i64> {
	// Remove $D prefix if present
	let clean_date_str = if date_str.starts_with("$D") {
		&date_str[2..]
	} else {
		date_str
	};

	// Try parsing as RFC3339 first (standard ISO 8601 with timezone)
	if let Ok(dt) = DateTime::parse_from_rfc3339(clean_date_str) {
		return Some(dt.timestamp());
	}

	// Try parsing without timezone (assume UTC)
	if let Ok(dt) = NaiveDateTime::parse_from_str(clean_date_str, "%Y-%m-%dT%H:%M:%S%.fZ") {
		return Some(dt.and_utc().timestamp());
	}

	// Try without milliseconds
	if let Ok(dt) = NaiveDateTime::parse_from_str(clean_date_str, "%Y-%m-%dT%H:%M:%SZ") {
		return Some(dt.and_utc().timestamp());
	}

	None
}

// Parse chapters from Next.js RSC streaming data (self.__next_f.push)
// This is the primary method as it contains isPremium field
fn parse_chapters_from_nextdata(html: &Document, manga_key: &str) -> Result<Vec<Chapter>> {
	// );

	// First try to find scripts with self.__next_f.push (RSC streaming format)
	if let Some(script_elements) = html.select("script") {

		for script in script_elements {
			if let Some(content) = script.data() {
				// Check if this script contains __next_f.push calls
				if content.contains("self.__next_f.push") {
					// );

					// Debug: check if contains "chapters" word
					let has_chapters_word = content.contains("chapters");
					let has_ispremium_word = content.contains("isPremium");

					// If this script contains both keywords, try parsing push() calls as JSON
					if has_chapters_word && has_ispremium_word {

						// RSC format: self.__next_f.push([id, "json_string"])
						// Strategy: Parse each push() call as JSON, extract the string, parse it, search for chapters

						// Helper function to recursively search for "chapters" array in JSON
						// Depth limiting prevents excessive recursion with malicious or deeply nested JSON
						fn find_chapters(val: &serde_json::Value, depth: usize) -> Option<&serde_json::Value> {
							// Limit recursion depth to prevent stack overflow
							if depth > 10 {
								return None;
							}

							if let Some(obj) = val.as_object() {
								if let Some(chapters) = obj.get("chapters") {
									if chapters.is_array() {
										return Some(chapters);
									}
								}
								for (_key, nested) in obj.iter() {
									if let Some(found) = find_chapters(nested, depth + 1) {
										return Some(found);
									}
								}
							} else if let Some(arr) = val.as_array() {
								for item in arr {
									if let Some(found) = find_chapters(item, depth + 1) {
										return Some(found);
									}
								}
							}
							None
						}

						// Find all self.__next_f.push( calls
						let mut search_start = 0;
						while let Some(push_start) = content[search_start..].find("self.__next_f.push(") {
							let absolute_push_start = search_start + push_start;
							let after_push = &content[absolute_push_start + 19..]; // Skip "self.__next_f.push("

							// Find the matching closing parenthesis
							let mut paren_count = 1;
							let mut in_string = false;
							let mut escape_next = false;
							let mut push_end = None;

							for (i, ch) in after_push.char_indices() {
								if escape_next {
									escape_next = false;
									continue;
								}

								match ch {
									'\\' => escape_next = true,
									'"' => in_string = !in_string,
									'(' if !in_string => paren_count += 1,
									')' if !in_string => {
										paren_count -= 1;
										if paren_count == 0 {
											push_end = Some(i);
											break;
										}
									},
									_ => {}
								}
							}

							if let Some(end) = push_end {
								let push_content = &after_push[..end];

								// Parse the push() arguments as JSON array: [id, "json_string"]
								match serde_json::from_str::<serde_json::Value>(push_content) {
									Ok(push_array) => {
										// Extract the second element (index 1) which is the JSON string
										if let Some(json_string) = push_array.get(1).and_then(|v| v.as_str()) {

											// RSC format: The string starts with "id:" prefix (e.g., "5:[...]")
											// We need to skip this prefix to get the actual JSON
											let actual_json = if let Some(colon_pos) = json_string.find(':') {
												&json_string[colon_pos + 1..]
											} else {
												json_string
											};


											// Parse the JSON string
											match serde_json::from_str::<serde_json::Value>(actual_json) {
												Ok(parsed_data) => {
													// Try to find chapters array in parsed data
													if let Some(chapters_value) = find_chapters(&parsed_data, 0) {

														// Parse the chapters array
														match serde_json::from_value::<Vec<serde_json::Value>>(chapters_value.clone()) {
															Ok(chapters_array) => {
														let mut chapters: Vec<Chapter> = Vec::new();

														for chapter in chapters_array.iter() {
															let chapter_number = chapter
																.get("number")
																.and_then(|v| {
																	if let Some(n) = v.as_f64() {
																		Some(n as f32)
																	} else if let Some(n) = v.as_i64() {
																		Some(n as f32)
																	} else {
																		None
																	}
																});

															if let Some(ch_num) = chapter_number {
																let is_premium = chapter
																	.get("isPremium")
																	.and_then(|v| v.as_bool())
																	.unwrap_or(false);


																let chapter_title = format!("Chapitre {}", ch_num);

																// Use chapter number as key (for URL construction)
																let chapter_key = if ch_num == (ch_num as i32) as f32 {
																	format!("{}", ch_num as i32)
																} else {
																	format!("{}", ch_num)
																};

																let url = format!(
																	"{}/serie/{}/chapter/{}",
																	BASE_URL,
																	manga_key,
																	chapter_key
																);

																// Parse createdAt date
																let date_uploaded = chapter
																	.get("createdAt")
																	.and_then(|v| v.as_str())
																	.and_then(|date_str| parse_iso_date(date_str));

																chapters.push(Chapter {
																	key: chapter_key,
																	title: Some(chapter_title),
																	volume_number: None,
																	chapter_number: Some(ch_num),
																	date_uploaded,
																	scanlators: None,
																	url: Some(url),
																	language: Some("fr".to_string()),
																	thumbnail: None,
																	locked: is_premium,
																});
															}
														}

														if !chapters.is_empty() {

															let min_premium_chapter = chapters
																.iter()
																.filter(|ch| ch.locked)
																.filter_map(|ch| ch.chapter_number)
																.min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

															if let Some(min_num) = min_premium_chapter {
																for chapter in &mut chapters {
																	if let Some(ch_num) = chapter.chapter_number {
																		if ch_num >= min_num {
																			chapter.locked = true;
																		}
																	}
																}
															} else {
															}

															chapters.sort_by(|a, b| {
																match (a.chapter_number, b.chapter_number) {
																	(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
																	(Some(_), None) => Ordering::Less,
																	(None, Some(_)) => Ordering::Greater,
																	(None, None) => Ordering::Equal,
																}
															});

															return Ok(chapters);
														}
													}
													Err(_e) => {
													}
												}
											} else {
											}
										}
										Err(_e) => {
										}
									}
								} else {
								}
							}
							Err(_e) => {
							}
						}

								search_start = absolute_push_start + 19 + end + 1;
							} else {
								break;
							}
						}

					}
				}
			}
		}
	}


	if let Some(script_elements) = html.select("script#__NEXT_DATA__") {

		let mut script_count = 0;
		for script in script_elements {
			script_count += 1;
			// );

			match script.data() {
				Some(content) => {
					// );

					// Log first 200 chars of content for debugging
					// };

					match serde_json::from_str::<serde_json::Value>(&content) {
						Ok(json_data) => {

							let possible_paths = [
								&json_data["props"]["pageProps"]["chapters"],
								&json_data["props"]["pageProps"]["initialData"]["chapters"],
								&json_data["props"]["pageProps"]["manga"]["chapters"],
								&json_data["pageProps"]["chapters"],
							];

							for (_idx, chapters_data) in possible_paths.iter().enumerate() {
								if let Some(chapters_array) = chapters_data.as_array() {
									let mut chapters: Vec<Chapter> = Vec::new();

									for chapter in chapters_array.iter() {
										let chapter_number = chapter.get("number").and_then(|v| {
											if let Some(n) = v.as_f64() {
												Some(n as f32)
											} else if let Some(n) = v.as_i64() {
												Some(n as f32)
											} else {
												None
											}
										});

										if chapter_number.is_none() {
											continue;
										}

										let ch_num = chapter_number.unwrap();

										let is_premium = chapter
											.get("isPremium")
											.and_then(|v| v.as_bool())
											.unwrap_or(false);

										// );

										// Use chapter number as key (for URL construction)
										let chapter_key = if ch_num == (ch_num as i32) as f32 {
											format!("{}", ch_num as i32)
										} else {
											format!("{}", ch_num)
										};

										let chapter_title = format!("Chapitre {}", ch_num);

										let url = format!(
											"{}/serie/{}/chapter/{}",
											BASE_URL, manga_key, chapter_key
										);

										// Parse createdAt date
										let date_uploaded = chapter
											.get("createdAt")
											.and_then(|v| v.as_str())
											.and_then(|date_str| parse_iso_date(date_str));

										chapters.push(Chapter {
											key: chapter_key,
											title: Some(chapter_title),
											volume_number: None,
											chapter_number: Some(ch_num),
											date_uploaded,
											scanlators: None,
											url: Some(url),
											language: Some("fr".to_string()),
											thumbnail: None,
											locked: is_premium,
										});
									}

									if !chapters.is_empty() {
										// );

										let min_premium_chapter = chapters
											.iter()
											.filter(|ch| ch.locked)
											.filter_map(|ch| ch.chapter_number)
											.min_by(|a, b| {
												a.partial_cmp(b).unwrap_or(Ordering::Equal)
											});

										if let Some(min_num) = min_premium_chapter {
											for chapter in &mut chapters {
												if let Some(ch_num) = chapter.chapter_number {
													if ch_num >= min_num {
														chapter.locked = true;
													}
												}
											}
										} else {
											// 	"[PoseidonScans] No premium chapters detected"
											// );
										}

										chapters.sort_by(|a, b| {
											match (a.chapter_number, b.chapter_number) {
												(Some(a_num), Some(b_num)) => b_num
													.partial_cmp(&a_num)
													.unwrap_or(Ordering::Equal),
												(Some(_), None) => Ordering::Less,
												(None, Some(_)) => Ordering::Greater,
												(None, None) => Ordering::Equal,
											}
										});

										return Ok(chapters);
									}
								} else {
								}
							}
						}
						Err(_e) => {
							// );
						}
					}
				}
				None => {
				}
			}
		}

		if script_count == 0 {
		}
	} else {
	}

	Ok(Vec::new())
}

// Detect premium chapter IDs from __NEXT_DATA__ or HTML
fn detect_premium_chapters_from_html(html: &Document) -> BTreeSet<String> {
	let mut premium_ids = BTreeSet::new();

	// Method 1: Try to extract from __NEXT_DATA__ (Next.js hydration data)
	if let Some(script_elements) = html.select("script#__NEXT_DATA__") {
		let mut script_count = 0;
		for script in script_elements {
			script_count += 1;
			// );

			match script.data() {
				Some(content) => {
					// );
					match serde_json::from_str::<serde_json::Value>(&content) {
						Ok(json_data) => {
							// Try to navigate to chapters data
							let possible_paths = [
								&json_data["props"]["pageProps"]["chapters"],
								&json_data["props"]["pageProps"]["initialData"]["chapters"],
								&json_data["props"]["pageProps"]["manga"]["chapters"],
								&json_data["pageProps"]["chapters"],
							];

							for (_idx, chapters_data) in possible_paths.iter().enumerate() {
								if let Some(chapters_array) = chapters_data.as_array() {
									for chapter in chapters_array.iter() {
										// Look for premium indicators in chapter data
										let is_premium = chapter
											.get("isPremium")
											.and_then(|v| v.as_bool())
											.unwrap_or(false) || chapter
											.get("premium")
											.and_then(|v| v.as_bool())
											.unwrap_or(false) || chapter
											.get("locked")
											.and_then(|v| v.as_bool())
											.unwrap_or(false);

										if is_premium {
											// Try to get chapter number or ID
											let chapter_id = chapter
												.get("number")
												.and_then(|v| v.as_i64())
												.map(|n| format!("{}", n))
												.or_else(|| {
													chapter
														.get("id")
														.and_then(|v| v.as_str())
														.map(|s| s.to_string())
												})
												.or_else(|| {
													chapter
														.get("chapterNumber")
														.and_then(|v| v.as_i64())
														.map(|n| format!("{}", n))
												});

											if let Some(id) = chapter_id {
												// );
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
						Err(_e) => {
							// );
						}
					}
				}
				None => {
				}
			}
		}

		if script_count == 0 {
			// 	"[PoseidonScans] Premium detection: __NEXT_DATA__ found but no scripts iterated"
			// );
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
			let has_amber_class =
				class_attr_str.contains("amber") || class_attr_str.contains("border-amber-500");

			let html_content_str = link.html().unwrap_or_default();
			let html_lower = html_content_str.to_lowercase();
			let has_premium_text =
				html_lower.contains("premium") || html_lower.contains("accès anticipé");

			let text_content_str = link.text().unwrap_or_default();
			let has_premium_in_text = text_content_str.to_uppercase().contains("PREMIUM");

			if has_amber_class || has_premium_text || has_premium_in_text {
				if let Some(chapter_id) = chapter_id {
					// );
					premium_ids.insert(chapter_id);
				}
			}
		}
	}

	// );
	premium_ids
}

pub fn parse_chapter_list(manga_key: String, html: &Document) -> Result<Vec<Chapter>> {
	// );

	// Try __NEXT_DATA__ first (contains isPremium field)
	if let Ok(chapters) = parse_chapters_from_nextdata(html, &manga_key) {
		if !chapters.is_empty() {
			// );
			return Ok(chapters);
		} else {
		}
	}

	let manga_data = extract_jsonld_manga_details(html)?;

	// Extract chapters from JSON-LD "hasPart" array
	let chapters_array =
		if let Some(has_part) = manga_data.get("hasPart").and_then(|c| c.as_array()) {
			// );
			has_part
		} else {
			return Ok(parse_chapter_list_from_html(html)?);
		};

	// Get premium chapter IDs from HTML (O(1) parse, no HTTP requests)
	let premium_chapter_ids = detect_premium_chapters_from_html(html);
	// );

	let mut chapters: Vec<Chapter> = Vec::new();

	// Parse each ComicIssue from JSON-LD
	for chapter_value in chapters_array {
		if let Some(chapter_obj) = chapter_value.as_object() {
			// Check if this is a ComicIssue
			if let Some(type_value) = chapter_obj.get("@type") {
				if let Some(type_str) = type_value.as_str() {
					if type_str != "ComicIssue" {
						continue;
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

			// Extract chapter title
			let chapter_title = format!("Chapitre {}", chapter_number);

			// Extract chapter URL directly from JSON-LD
			let url = chapter_obj
				.get("url")
				.and_then(|u| u.as_str())
				.unwrap_or_default()
				.to_string();

			// Use chapter number as key (for URL construction)
			let chapter_key = if chapter_number == (chapter_number as i32) as f32 {
				format!("{}", chapter_number as i32)
			} else {
				format!("{}", chapter_number)
			};

			// Check if this chapter is premium (for fallback detection)
			let is_locked = premium_chapter_ids.contains(&chapter_key);

			chapters.push(Chapter {
				key: chapter_key,
				title: Some(chapter_title),
				volume_number: None,
				chapter_number: Some(chapter_number),
				date_uploaded: None,
				scanlators: None,
				url: Some(url),
				language: Some("fr".to_string()),
				thumbnail: None,
				locked: is_locked,
			});
		}
	}

	// Post-processing: mark all chapters >= min premium as locked
	let min_premium_chapter = chapters
		.iter()
		.filter(|ch| ch.locked)
		.filter_map(|ch| ch.chapter_number)
		.min_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal));

	if let Some(min_num) = min_premium_chapter {
		for chapter in &mut chapters {
			if let Some(ch_num) = chapter.chapter_number {
				if ch_num >= min_num {
					chapter.locked = true;
				}
			}
		}
	}

	// Sort chapters by number in descending order (newest first)
	chapters.sort_by(|a, b| match (a.chapter_number, b.chapter_number) {
		(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
		(Some(_), None) => Ordering::Less,
		(None, Some(_)) => Ordering::Greater,
		(None, None) => Ordering::Equal,
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
				// Extract chapter number from URL
				if let Some(chapter_id) = extract_chapter_id_from_url(&href_str) {
					if !seen_chapter_ids.insert(chapter_id.clone()) {
						continue; // insert() returns false if element was already
					}

					// Extract chapter number from URL or ID first
					let chapter_number = extract_chapter_number_from_id(&chapter_id);

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

					// Use chapter_id directly as key (it's already the chapter number from URL)
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
	chapters.sort_by(|a, b| match (a.chapter_number, b.chapter_number) {
		(Some(a_num), Some(b_num)) => b_num.partial_cmp(&a_num).unwrap_or(Ordering::Equal),
		(Some(_), None) => Ordering::Less,
		(None, Some(_)) => Ordering::Greater,
		(None, None) => Ordering::Equal,
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
								if let Some(images) =
									json_data.get("images").and_then(|i| i.as_array())
								{
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
								if let Some(images) =
									initial_data.get("images").and_then(|i| i.as_array())
								{
									let pages = parse_images_from_json_array(images)?;
									if !pages.is_empty() {
										return Ok(pages);
									}
								}
							}
							if let Some(images) =
								page_props.get("images").and_then(|i| i.as_array())
							{
								let pages = parse_images_from_json_array(images)?;
								if !pages.is_empty() {
									return Ok(pages);
								}
							}
							if let Some(chapter) = page_props.get("chapter") {
								if let Some(images) =
									chapter.get("images").and_then(|i| i.as_array())
								{
									let pages = parse_images_from_json_array(images)?;
									if !pages.is_empty() {
										return Ok(pages);
									}
								}
							}
						}
					}
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

					pages.push((
						order,
						Page {
							content: PageContent::url(absolute_url),
							thumbnail: None,
							has_description: false,
							description: None,
						},
					));
				}
			}
		}

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
				let image_url = img_element
					.attr("src")
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
			let image_url = image_obj
				.get("url")
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

// Parse the /series HTML page to extract manga list
pub fn parse_series_page(html: &Document) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	// Select all manga links
	let manga_selector = "a[href^=\"/serie/\"]";

	if let Some(manga_elements) = html.select(manga_selector) {
		for manga_element in manga_elements {
			if let Some(href) = manga_element.attr("href") {
				// Extract slug from href="/serie/{slug}"
				let slug = if href.starts_with("/serie/") {
					&href[7..] // Skip "/serie/"
				} else {
					continue;
				};

				// Extract title from h2 element
				let title = if let Some(h2_element) = manga_element.select("h2").and_then(|els| els.first()) {
					h2_element.text().unwrap_or_default().trim().to_string()
				} else {
					slug.to_string()
				};

				// Extract status - it's typically right after the h2, in a sibling div
				let mut status = MangaStatus::Unknown;
				if let Some(h2_element) = manga_element.select("h2").and_then(|els| els.first()) {
					// Try to find the status div that's a sibling to h2
					if let Some(parent) = h2_element.parent() {
						if let Some(status_elements) = parent.select("div") {
							// The first div after h2 usually contains the status
							for status_elem in status_elements {
								let status_text = status_elem.text().unwrap_or_default().trim().to_string();
								// Check if it matches a known status
								if status_text == "en cours" || status_text == "terminé" ||
								   status_text == "en pause" || status_text == "annulé" {
									status = parse_manga_status(&status_text);
									break;
								}
							}
						}
					}
				}

				// Extract tags/categories - collect all text from divs that look like genre tags
				let mut tags_vec: Vec<String> = Vec::new();
				if let Some(all_divs) = manga_element.select("div") {
					for div_elem in all_divs {
						let text = div_elem.text().unwrap_or_default().trim().to_string();
						// Filter out non-tag elements (status, chapter count, etc)
						if !text.is_empty() &&
						   text != "en cours" && text != "terminé" && text != "en pause" && text != "annulé" &&
						   !text.contains("chapitre") && text.len() < 50 {
							// This looks like it could be a tag
							if !tags_vec.contains(&text) &&
							   (text.starts_with(char::is_uppercase) || text.starts_with("É") || text.starts_with("À")) {
								tags_vec.push(text);
							}
						}
					}
				}
				let tags = if tags_vec.is_empty() {
					None
				} else {
					Some(tags_vec)
				};

				// Build cover URL
				let cover = format!("{}/api/covers/{}.webp", BASE_URL, slug);

				let content_rating = calculate_content_rating(&tags);
				let viewer = calculate_viewer(&tags);

				mangas.push(Manga {
					key: slug.to_string(),
					title,
					cover: Some(cover),
					authors: None,
					artists: None,
					description: None,
					url: Some(format!("{}/serie/{}", BASE_URL, slug)),
					tags,
					status,
					content_rating,
					viewer,
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
	}

	// Detect if there's a next page by looking for "Suivant" (Next) link
	let has_next_page = html
		.select("a")
		.and_then(|links| {
			for link in links {
				if let Some(text) = link.text() {
					if text.trim() == "Suivant" {
						return Some(true);
					}
				}
			}
			Some(false)
		})
		.unwrap_or(false);

	Ok(MangaPageResult {
		entries: mangas,
		has_next_page,
	})
}

pub fn parse_series_and_filter(
	html: Document,
	query: Option<String>,
	status_filter: Option<String>,
	type_filter: Option<String>,
	genre_filter: Option<String>,
	sort_filter: Option<String>,
	page: i32,
) -> Result<MangaPageResult> {
	// Parse HTML page to get all mangas
	let parsed_result = parse_series_page(&html)?;
	let mut mangas = parsed_result.entries;

	// Apply query filter (case-insensitive title search)
	if let Some(q) = query {
		if !q.is_empty() {
			let query_lower = q.to_lowercase();
			mangas.retain(|m| m.title.to_lowercase().contains(&query_lower));
		}
	}

	// Apply status filter
	if let Some(status) = status_filter {
		let target_status = parse_manga_status(&status);
		mangas.retain(|m| m.status == target_status);
	}

	// Apply type filter (MANGA, MANHWA, MANHUA, WEBTOON)
	if let Some(type_val) = type_filter {
		mangas.retain(|m| {
			if let Some(ref tags) = m.tags {
				tags.iter().any(|t| t == &type_val)
			} else {
				false
			}
		});
	}

	// Apply genre filter
	if let Some(genre) = genre_filter {
		mangas.retain(|m| {
			if let Some(ref tags) = m.tags {
				tags.iter().any(|t| t == &genre)
			} else {
				false
			}
		});
	}

	// Apply sorting
	match sort_filter.as_deref() {
		Some("alphabetical") => {
			mangas.sort_by(|a, b| a.title.cmp(&b.title));
		}
		Some("popular") => {
			// Popular sorting - would need view count data
			// For now, keep default order which is likely already sorted by popularity
		}
		Some("created") => {
			// Creation date sorting - would need creation date data
			// Keep default order
		}
		Some("recent") | None => {
			// Default: recent updates (already in API order)
		}
		_ => {}
	}

	// Client-side pagination (20 items per page)
	let items_per_page = 20;
	let start_idx = ((page - 1) * items_per_page) as usize;
	let end_idx = (start_idx + items_per_page as usize).min(mangas.len());

	let has_next_page = end_idx < mangas.len();
	let paginated_mangas = if start_idx < mangas.len() {
		mangas[start_idx..end_idx].to_vec()
	} else {
		Vec::new()
	};

	Ok(MangaPageResult {
		entries: paginated_mangas,
		has_next_page,
	})
}
