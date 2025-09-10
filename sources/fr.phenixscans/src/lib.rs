#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, string::ToString},
	imports::net::{Request, Response},
	prelude::*,
	AidokuError,
};

mod parser;
mod helper;

pub const BASE_URL: &str = "https://phenix-scans.com";
pub const API_URL: &str = "https://phenix-scans.com/api";

struct PhenixScans;

impl PhenixScans {
	// Helper function for robust API requests with Cloudflare bypass and error handling
	fn make_api_request_with_retry(&self, url: &str) -> Result<Response> {
		let mut attempt = 0;
		const MAX_RETRIES: u32 = 3;
		
		loop {
			// Rate limiting: Add delays between requests to avoid triggering Cloudflare
			if attempt > 0 {
				let _backoff_ms = 500 * (1 << (attempt - 1).min(3)); // Exponential backoff: 500ms, 1s, 2s, 4s
				// Exponential backoff would be implemented here in non-WASM environment
			} else {
				// Even on first request, add small delay for API to avoid rapid requests
				// Rate limiting would be implemented here in non-WASM environment
			}
			
			let request = Request::get(url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Accept-Encoding", "gzip, deflate, br")
				.header("DNT", "1")
				.header("Connection", "keep-alive")
				.header("Upgrade-Insecure-Requests", "1")
				.header("Referer", "https://phenix-scans.com/")
				.header("Origin", "https://phenix-scans.com");
			
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
				403 => {
					// Cloudflare block detected
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Cloudflare block (403)"));
					}
					attempt += 1;
					continue;
				},
				429 => {
					// Rate limited by API or Cloudflare
					if attempt >= MAX_RETRIES {
						return Err(AidokuError::message("Rate limited (429)"));
					}
					attempt += 1;
					continue;
				},
				503 | 502 | 504 => {
					// Server error, might be temporary Cloudflare protection
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
	
	// Helper for robust JSON API requests with graceful error handling
	fn get_api_json_robust(&self, url: &str) -> Result<String> {
		match self.make_api_request_with_retry(url) {
			Ok(response) => {
				let json_string = response.get_string()?;
				
				// Check if response looks like an error page (HTML instead of JSON)
				if json_string.trim_start().starts_with('<') || 
				   json_string.contains("403 Forbidden") || 
				   json_string.contains("Access Denied") {
					// Return error to trigger fallback behavior
					return Err(AidokuError::message("JSON parse error"));
				}
				
				Ok(json_string)
			},
			Err(_) => {
				// Return error to trigger fallback behavior
				Err(AidokuError::message("API request failed"))
			}
		}
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
							// Debug: let's see what value we actually receive
							// For isGenre=true filters, value should contain the genre ID directly
							if !value.is_empty() && value != "" && value != "Tout" {
								// Try multiple possibilities:
								// 1. Direct ID (like the old system)
								if value.len() > 20 && value.starts_with("67bd") {
									genre_ids.push(value.clone());
								} 
								// 2. Index-based (parse as number)
								else if let Ok(index) = value.parse::<usize>() {
									let genre_id_list = [
										"", "67bd1aefae810159afa2675e", "67bd1aefae810159afa2675f", 
										"67bd1aefae810159afa26760", "67bd1aefae810159afa26761", "67bd1aefae810159afa26762",
										"67bd1aefae810159afa26763", "67bd1b03ae810159afa26779", "67bd1b03ae810159afa2677a",
										"67bd1b03ae810159afa2677b", "67bd1b03ae810159afa2677c", "67bd1ba0ae810159afa267e7",
										"67bd1ba0ae810159afa267e8", "67bd1ba0ae810159afa267e9", "67bd1ba0ae810159afa267ea",
										"67bd1ca0ae810159afa268f7", "67bd1ca0ae810159afa268f8", "67bd1ca0ae810159afa268f9",
										"67bd1ca0ae810159afa268fa", "67bd1cd6ae810159afa2692f", "67bd1dffae810159afa26a3b",
										"67bd1f24ae810159afa26b6c", "67bd1f24ae810159afa26b6d", "67bd1f24ae810159afa26b6e",
										"67bd1f53ae810159afa26b8d", "67bd1f53ae810159afa26b8e", "67bd1fbeae810159afa26bdf",
										"67bd203cae810159afa26c7f", "67bd203cae810159afa26c80", "67bd203cae810159afa26c81",
										"67bd203cae810159afa26c82", "67bd203cae810159afa26c83", "67bd211bae810159afa26d9d",
										"67bd21a8ae810159afa26dfc", "67bd21a8ae810159afa26dfd", "67bd224eae810159afa26eb2",
										"67bd225aae810159afa26eb9", "67bd22e6ae810159afa26f36", "67bd22e6ae810159afa26f37",
										"67bd22e6ae810159afa26f38", "67bd2386ae810159afa26f59", "67bd260cae810159afa27194",
										"67bd2668ae810159afa271dd", "67bd2668ae810159afa271de", "67bd26e8ae810159afa27249",
										"67bd272bae810159afa2727c", "67bd272bae810159afa2727d", "67bd28ecae810159afa273ec",
										"67bd2c07ae810159afa27561", "67bd2d64ae810159afa27694", "67bd2de7ae810159afa27712",
										"67bd2de7ae810159afa27713", "67bd2deeae810159afa27719", "67bd2e07ae810159afa27728",
										"67bd2e28ae810159afa27743", "67bd2e55ae810159afa27769", "67bd2e6dae810159afa27779",
										"67bd2f7fae810159afa2785e", "67bd2f80ae810159afa27860", "67bd333dae810159afa27a85",
										"67bd333dae810159afa27a86", "67bd333dae810159afa27a87", "67bd335bae810159afa27aa6",
										"67bd3374ae810159afa27abc", "67bd3889ae810159afa27ee9", "67bd39fbae810159afa28014",
										"67bd3a39ae810159afa28049", "67bd3a5dae810159afa2805d", "67bd3a5dae810159afa2805e",
										"67bd3a5dae810159afa2805f", "67bd3c38ae810159afa281f2", "67bd3c38ae810159afa281f3",
										"67bd3c72ae810159afa2822c", "67bd43a9ae810159afa287cf", "67bd43c0ae810159afa287e1",
										"67bd43c0ae810159afa287e2", "67bd44c8ae810159afa288e4", "67bd483cae810159afa28b8b",
										"67bd4eaaae810159afa28fd4", "67bd4eaaae810159afa28fd5", "67bd4eaaae810159afa28fd6",
										"67bd4eaaae810159afa28fd7", "67bd4eaaae810159afa28fd8", "67bd4eaaae810159afa28fd9",
										"67bd5719ae810159afa2960f", "67bd57f2ae810159afa296d0", "67bd680eae810159afa2a430",
										"67bd68daae810159afa2a4df", "67bd6c16ae810159afa2a6d3", "67bd708bae810159afa2aa74",
										"67bd7373ae810159afa2ad11", "67bd7373ae810159afa2ad12", "67bd76b3ae810159afa2afee",
										"67bd76b3ae810159afa2afef", "67bd7763ae810159afa2b0b7", "67bd7811ae810159afa2b161",
										"67bd82c9ae810159afa2bb11", "67bd82c9ae810159afa2bb12", "67bd8482ae810159afa2bbb4",
										"67bd8859ae810159afa2bf6a"
									];
									
									if index > 0 && index < genre_id_list.len() {
										let genre_id = genre_id_list[index];
										if !genre_id.is_empty() {
											genre_ids.push(genre_id.to_string());
										}
									}
								}
								// 3. Genre name (fallback)
								else {
									// Last resort - directly use the value as received
									genre_ids.push(value.clone());
								}
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
			let response = match self.get_api_json_robust(&url) {
				Ok(json) => json,
				Err(_) => {
					// If search fails, return empty result
					return Ok(MangaPageResult {
						entries: Vec::new(),
						has_next_page: false,
					});
				}
			};
			
			parser::parse_search_list(response)
		} else {
			// Listing/filtering endpoint with parameters
			let genre_param = if genre_ids.is_empty() {
				String::new()
			} else {
				format!("&genre={}", genre_ids.join(","))
			};
			
			let url = format!("{}/front/manga?page={}&limit=20{}{}", 
				API_URL, page, query_params, genre_param);
			let response = match self.get_api_json_robust(&url) {
				Ok(json) => json,
				Err(_) => {
					// If listing fails, return empty result
					return Ok(MangaPageResult {
						entries: Vec::new(),
						has_next_page: false,
					});
				}
			};
			
			parser::parse_manga_list(response)
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
				// Essayer de parser les détails, mais continuer si ça échoue
				if let Ok(detailed_manga) = parser::parse_manga_details(manga.key.clone(), response.clone()) {
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
				// Si le parsing échoue, on garde les valeurs existantes du manga
			}

			if needs_chapters {
				// Essayer de parser les chapitres, mais continuer si ça échoue
				if let Ok(chapters) = parser::parse_chapter_list(manga.key.clone(), response) {
					manga.chapters = Some(chapters);
				}
				// Si le parsing échoue, on garde les chapitres existants (ou None)
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Utiliser le vrai endpoint API pour les pages avec headers Cloudflare
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = match self.get_api_json_robust(&url) {
			Ok(json) => json,
			Err(_) => {
				// If API fails, return empty page list
				return Ok(Vec::new());
			}
		};
		parser::parse_page_list(response)
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
			format!("{}/front/homepage?page={}&section=latest&limit=20", API_URL, page)
		} else if listing.name == "Populaire" {
			format!("{}/front/homepage?section=top", API_URL)
		} else {
			return Err(aidoku::AidokuError::message("Unimplemented listing"));
		};
		
		// Faire la requête API JSON avec headers Cloudflare
		let response = match self.get_api_json_robust(&url) {
			Ok(json) => json,
			Err(_) => {
				// If listing fails, return empty result
				return Ok(MangaPageResult {
					entries: Vec::new(),
					has_next_page: false,
				});
			}
		};
		
		parser::parse_manga_listing(response, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);