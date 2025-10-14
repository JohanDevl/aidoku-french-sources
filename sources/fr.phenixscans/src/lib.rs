#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult,
	Page, Result, Source,
	alloc::{String, Vec},
	imports::{net::Response, std::send_partial_result},
	prelude::*,
	AidokuError,
};

mod parser;
mod helper;

pub const BASE_URL: &str = "https://phenix-scans.com";
pub const API_URL: &str = "https://phenix-scans.com/api";
const MAX_RETRIES: u32 = 3;
const PAGE_LIMIT: i32 = 20;

struct PhenixScans;

impl PhenixScans {
	fn make_api_request_with_retry(&self, url: &str) -> Result<Response> {
		let mut attempt = 0;

		loop {
			let request = helper::build_request(url)?;

			let response = match request.send() {
				Ok(resp) => resp,
				Err(e) => {
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::RequestError(e));
					}
					attempt += 1;
					continue;
				}
			};
			
			match response.status_code() {
				200..=299 => return Ok(response),
				301 | 302 => return Err(AidokuError::message("Redirect not supported")),
				401 => return Err(AidokuError::message("Authentication required")),
				403 => {
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Cloudflare block (403)"));
					}
					attempt += 1;
					continue;
				},
				404 => return Err(AidokuError::message("Not found")),
				408 => {
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Request timeout"));
					}
					attempt += 1;
					continue;
				},
				429 => {
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Rate limited (429)"));
					}
					attempt += 1;
					continue;
				},
				500 => return Err(AidokuError::message("Internal server error")),
				502 | 503 | 504 => {
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Server error"));
					}
					attempt += 1;
					continue;
				},
				_ => return Err(AidokuError::message("Request failed")),
			}
		}
	}
	
	fn get_api_json_robust(&self, url: &str) -> Result<String> {
		let response = self.make_api_request_with_retry(url)?;
		let json_string = response.get_string()?;
		helper::validate_json_response(&json_string)?;
		Ok(json_string)
	}
}

impl Source for PhenixScans {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// Process filters to build API query parameters
		let mut query_params = String::new();
		let mut genre_ids: Vec<String> = Vec::new();
		
		for filter in filters {
			match filter {
				FilterValue::Select { id, value } => {
					match id.as_str() {
						"status" => {
							match value.as_str() {
								"En cours" => query_params.push_str("&status=Ongoing"),
								"Terminé" => query_params.push_str("&status=Completed"),
								"En pause" => query_params.push_str("&status=Hiatus"),
								_ => continue,
							}
						}
						"type" => {
							match value.as_str() {
								"Manga" => query_params.push_str("&type=Manga"),
								"Manhwa" => query_params.push_str("&type=Manhwa"),
								"Manhua" => query_params.push_str("&type=Manhua"),
								_ => continue,
							}
						}
						"Genres" => {
							if !value.is_empty() && value != "Tout" {
								genre_ids.push(value);
							}
						}
						_ => continue,
					}
				}
				_ => continue,
			}
		}

		// Build final URL based on search query and filters
		if let Some(search_query) = query {
			// Search endpoint with query
			let url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			let response = self.get_api_json_robust(&url)?;

			parser::parse_search_list(&response)
		} else {
			// Listing/filtering endpoint with parameters
			let genre_param = if genre_ids.is_empty() {
				String::new()
			} else {
				format!("&genre={}", genre_ids.join(","))
			};
			
			let url = format!("{}/front/manga?page={}&limit={}{}{}",
				API_URL, page, PAGE_LIMIT, query_params, genre_param);
			let response = self.get_api_json_robust(&url)?;

			parser::parse_manga_list(&response)
		}
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details || needs_chapters {
			// Utiliser le vrai endpoint API avec headers Cloudflare
			let url = format!("{}/front/manga/{}", API_URL, manga.key);
			let response = self.get_api_json_robust(&url)?;
			
			if needs_details {
				if let Ok(detailed_manga) = parser::parse_manga_details(&manga.key, &response) {
					manga.title = detailed_manga.title;
					manga.authors = detailed_manga.authors;
					manga.artists = detailed_manga.artists;
					manga.description = detailed_manga.description;
					manga.url = detailed_manga.url;
					manga.cover = detailed_manga.cover;
					manga.tags = detailed_manga.tags;
					manga.status = detailed_manga.status;
					manga.content_rating = detailed_manga.content_rating;
					manga.viewer = detailed_manga.viewer;
				}
				send_partial_result(&manga);
			}

			if needs_chapters {
				if let Ok(chapters) = parser::parse_chapter_list(&manga.key, &response) {
					manga.chapters = Some(chapters);
				}
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Utiliser le vrai endpoint API pour les pages avec headers Cloudflare
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = self.get_api_json_robust(&url)?;
		parser::parse_page_list(&response)
	}
}

impl ListingProvider for PhenixScans {
	fn get_manga_list(
		&self,
		listing: Listing,
		page: i32,
	) -> Result<MangaPageResult> {
		// Utiliser les vrais endpoints API homepage
		let url = if listing.name == "Dernières Sorties" {
			format!("{}/front/homepage?page={}&section=latest&limit={}", API_URL, page, PAGE_LIMIT)
		} else if listing.name == "Populaire" {
			format!("{}/front/homepage?section=top", API_URL)
		} else {
			return Err(aidoku::AidokuError::message("Unimplemented listing"));
		};
		
		let response = self.get_api_json_robust(&url)?;

		parser::parse_manga_listing(&response, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);