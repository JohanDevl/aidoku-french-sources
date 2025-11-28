#![no_std]

use aidoku::{
	Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
	Page, PageContext, Result, Source,
	alloc::{String, Vec, vec},
	imports::{net::{Request, Response}, std::send_partial_result},
	prelude::*,
	AidokuError,
};

extern crate alloc;
use alloc::format;


// Modules contenant la logique de parsing sophistiquée d'AnimeSama
pub mod parser;
pub mod helper;

pub const BASE_URL: &str = "https://anime-sama.org";
pub const CDN_URL: &str = "https://anime-sama.org/s2/scans";
pub const CDN_URL_LEGACY: &str = "https://s22.anime-sama.me/s1/scans";

// Helper function for robust HTTP requests with Cloudflare bypass and error handling
fn make_request_with_cloudflare_retry(url: &str) -> Result<Response> {
	let mut attempt = 0;
	const MAX_RETRIES: u32 = 3;
	
	loop {
		// Rate limiting: Add delays between requests to avoid triggering Cloudflare
		if attempt > 0 {
			let _backoff_ms = 1000 * (1 << (attempt - 1).min(3)); // Exponential backoff: 1s, 2s, 4s, 8s
			// Exponential backoff would be implemented here in non-WASM environment
		} else {
			// Even on first request, add small delay to avoid rapid-fire requests
			// Rate limiting would be implemented here in non-WASM environment
		}
		
		let request = Request::get(url)?
			.header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604")
			.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Accept-Encoding", "gzip, deflate, br")
			.header("DNT", "1")
			.header("Connection", "keep-alive")
			.header("Upgrade-Insecure-Requests", "1")
			.header("Cache-Control", "max-age=0")
			.header("Referer", BASE_URL);
		
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
				// Cloudflare block, retry with longer delay
				if attempt >= MAX_RETRIES {
					return Err(AidokuError::message("Cloudflare block (403)"));
				}
				attempt += 1;
				continue;
			},
			429 => {
				// Rate limited by Cloudflare, retry with exponential backoff
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

fn make_realistic_request(url: &str) -> Result<aidoku::imports::html::Document> {
	let response = make_request_with_cloudflare_retry(url)?;
	Ok(response.get_html()?)
}

struct AnimeSama;

impl Source for AnimeSama {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let mut filter_params = String::new();

		for filter in &filters {
			match filter {
				FilterValue::MultiSelect { id, included, excluded: _ } => {
					if id == "genre" && !included.is_empty() {
						for genre_value in included {
							if !genre_value.is_empty() {
								filter_params.push_str(&format!("&genre%5B%5D={}", helper::urlencode(genre_value)));
							}
						}
					}
				}
				FilterValue::Text { id, value } => {
					if id == "genre" && !value.is_empty() {
						filter_params.push_str(&format!("&genre%5B%5D={}", helper::urlencode(value)));
					}
				}
				_ => {}
			}
		}
		
		// Construire l'URL de recherche pour anime-sama.org
		// Format attendu: type%5B%5D=Scans&genre%5B%5D=Genre&search=query&page=N
		let url = if let Some(search_query) = query {
			// Avec recherche
			format!("{}/catalogue?type%5B%5D=Scans{}&search={}&page={}", 
				BASE_URL, 
				filter_params,
				helper::urlencode(&search_query),
				page
			)
		} else {
			// Sans recherche mais avec search= vide pour correspondre au format du site
			format!("{}/catalogue?type%5B%5D=Scans{}&search=&page={}", 
				BASE_URL, 
				filter_params,
				page
			)
		};
		
		// Faire la requête HTTP avec headers réalistes et propagation d'erreur
		let html = make_realistic_request(&url)?;
		
		// Parser les résultats
		parser::parse_manga_list(html)
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let clean_key = if manga.key.starts_with("http") {
			helper::clean_url(&manga.key)
		} else {
			let cleaned = helper::clean_url(&manga.key);
			format!("{}{}", BASE_URL, cleaned)
		};
		let base_manga_url = clean_key;

		if needs_details {
			// Faire une requête pour récupérer les détails du manga (URL de base)
			let html = make_realistic_request(&base_manga_url)?;
			let detailed_manga = parser::parse_manga_details(manga.key.clone(), html)?;
			
			// Mettre à jour les champs du manga avec les détails récupérés
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

			if needs_chapters {
				send_partial_result(&manga);
			}
		}

		if needs_chapters {
			// Pour les chapitres, utiliser aussi l'URL de base (le JavaScript est sur la page principale)
			let html = make_realistic_request(&base_manga_url)?;
			let chapters = parser::parse_chapter_list(manga.key.clone(), html)?;
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let chapter_url = chapter.url.unwrap_or_else(|| {
			let clean_manga_key = helper::clean_url(&manga.key);

			let is_one_piece = helper::is_one_piece_manga(&clean_manga_key);
			let scan_path = if is_one_piece {
				if let Ok(chapter_num) = chapter.key.parse::<i32>() {
					if chapter_num >= 1046 {
						helper::SCAN_VF_PATH
					} else {
						helper::SCAN_BW_PATH
					}
				} else {
					helper::SCAN_BW_PATH
				}
			} else {
				helper::SCAN_VF_PATH
			};
			
			if clean_manga_key.starts_with("http") {
				// Remove trailing slash from clean_manga_key if it exists to avoid double slash
				let clean_key = clean_manga_key.trim_end_matches('/');
				format!("{}{}?id={}", clean_key, scan_path, chapter.key)
			} else {
				// Remove trailing slash from clean_manga_key if it exists to avoid double slash
				let clean_key = clean_manga_key.trim_end_matches('/');
				format!("{}{}{}?id={}", BASE_URL, clean_key, scan_path, chapter.key)
			}
		});
		
		// Faire une requête pour récupérer la page du chapitre
		let html = make_realistic_request(&chapter_url)?;
		
		// Parser les pages depuis le HTML ou utiliser la logique CDN
		parser::parse_page_list(html, manga.key, chapter.key)
	}
}

impl ListingProvider for AnimeSama {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"dernières-sorties" => {
				// Faire une requête vers la page d'accueil pour les dernières sorties
				let html = make_realistic_request(BASE_URL)?;
				parser::parse_manga_listing(html, "Dernières Sorties")
			},
			"populaire" => {
				// Faire une requête vers le catalogue pour les mangas populaires
				let url = format!("{}/catalogue?type%5B%5D=Scans&search=&page={}", BASE_URL, page);
				let html = make_realistic_request(&url)?;
				parser::parse_manga_listing(html, "Populaire")
			},
			_ => {
				// Listing ID non reconnu, retourner résultat vide
				Ok(MangaPageResult {
					entries: vec![],
					has_next_page: false,
				})
			}
		}
	}
}

impl ImageRequestProvider for AnimeSama {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(url)?
			.header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604")
			.header("Accept", "image/webp,image/apng,image/*,*/*;q=0.8")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Referer", BASE_URL))
	}
}

register_source!(AnimeSama, ListingProvider, ImageRequestProvider);