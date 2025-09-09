#![no_std]

use aidoku::{
	Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, PageContext, Result, Source,
	alloc::{String, Vec, vec, string::ToString},
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

// Note: filters.json est inclus pour référence mais on utilise la liste hardcodée pour la performance

// Parser les IDs de genres depuis le JSON inclus
fn get_genre_ids() -> Vec<&'static str> {
	// Liste complète basée sur le site AnimeSama
	// Utilise les valeurs exactes des checkbox du formulaire de filtrage
	vec![
		"", // Pour "Tout"
		"Action",
		"Adolescence",
		"Aliens / Extra-terrestres",
		"Amitié",
		"Amour",
		"Apocalypse",
		"Art",
		"Arts martiaux",
		"Assassinat",
		"Autre monde",
		"Aventure",
		"Combats",
		"Comédie",
		"Crime",
		"Cyberpunk",
		"Danse",
		"Démons",
		"Détective",
		"Donghua",
		"Dragon",
		"Drame",
		"Ecchi",
		"Ecole",
		"Elfe",
		"Enquête",
		"Famille",
		"Fantastique",
		"Fantasy",
		"Fantômes",
		"Futur",
		"Gastronomie",
		"Ghibli",
		"Guerre",
		"Harcèlement",
		"Harem",
		"Harem inversé",
		"Histoire",
		"Historique",
		"Horreur",
		"Isekai",
		"Jeunesse",
		"Jeux",
		"Jeux vidéo",
		"Josei",
		"Journalisme",
		"Mafia",
		"Magical girl",
		"Magie",
		"Maladie",
		"Mariage",
		"Mature",
		"Mechas",
		"Médiéval",
		"Militaire",
		"Monde virtuel",
		"Monstres",
		"Musique",
		"Mystère",
		"Nekketsu",
		"Ninjas",
		"Nostalgie",
		"Paranormal",
		"Philosophie",
		"Pirates",
		"Police",
		"Politique",
		"Post-apocalyptique",
		"Pouvoirs psychiques",
		"Préhistoire",
		"Prison",
		"Psychologique",
		"Quotidien",
		"Religion",
		"Réincarnation / Transmigration",
		"Romance",
		"Samouraïs",
		"School Life",
		"Science-Fantasy",
		"Science-fiction",
		"Scientifique",
		"Seinen",
		"Shôjo",
		"Shôjo-Ai",
		"Shônen",
		"Shônen-Ai",
		"Slice of Life",
		"Société",
		"Sport",
		"Super pouvoirs",
		"Super-héros",
		"Surnaturel",
		"Survie",
		"Survival game",
		"Technologies",
		"Thriller",
		"Tournois",
		"Travail",
		"Vampires",
		"Vengeance",
		"Voyage",
		"Voyage temporel",
		"Webcomic",
		"Yakuza",
		"Yaoi",
		"Yokai",
		"Yuri"
	]
}

// Vérifier si un ID de genre est valide
fn is_valid_genre_id(genre_id: &str) -> bool {
	get_genre_ids().contains(&genre_id)
}

// Helper function for robust HTTP requests with Cloudflare bypass and error handling
fn make_request_with_cloudflare_retry(url: &str) -> Result<Response> {
	let mut attempt = 0;
	const MAX_RETRIES: u32 = 3;
	
	loop {
		// Rate limiting: Add delays between requests to avoid triggering Cloudflare
		if attempt > 0 {
			let backoff_ms = 1000 * (1 << (attempt - 1).min(3)); // Exponential backoff: 1s, 2s, 4s, 8s
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
					return Err(e);
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
					return Err(AidokuError::HttpError(403));
				}
				attempt += 1;
				continue;
			},
			429 => {
				// Rate limited by Cloudflare, retry with exponential backoff
				if attempt >= MAX_RETRIES {
					return Err(AidokuError::HttpError(429));
				}
				attempt += 1;
				continue;
			},
			503 | 502 | 504 => {
				// Server error, might be temporary Cloudflare protection
				if attempt >= MAX_RETRIES {
					return Err(AidokuError::HttpError(response.status_code()));
				}
				attempt += 1;
				continue;
			},
			_ => return Err(AidokuError::HttpError(response.status_code())),
		}
	}
}

// Wrapper function with fallback for HTML parsing
fn make_realistic_request(url: &str) -> Result<aidoku::imports::html::Document> {
	match make_request_with_cloudflare_retry(url) {
		Ok(response) => response.get_html(),
		Err(_) => {
			// If all retries fail, return an error that can be handled gracefully
			Err(AidokuError::HttpError(503))
		}
	}
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
		// Construire les paramètres de filtre
		let mut filter_params = String::new();
		
		// Traiter chaque filtre - utiliser le pattern matching sur FilterValue
		for filter in &filters {
			match filter {
				FilterValue::MultiSelect { id, included, excluded: _ } => {
					// Vérifier que c'est le filtre de genre et qu'il y a des genres sélectionnés
					if id == "genre" && !included.is_empty() {
						let genre_ids = get_genre_ids();
						
						// Traiter chaque genre inclus
						for selected_value in included {
							if !selected_value.is_empty() {
								// Essayer de parser la valeur comme un index
								if let Ok(selected_index) = selected_value.parse::<i32>() {
									if selected_index >= 0 && (selected_index as usize) < genre_ids.len() {
										let genre_id = genre_ids[selected_index as usize];
										// Format correct pour AnimeSama : utiliser genre%5B%5D=
										if !genre_id.is_empty() {
											filter_params.push_str(&format!("&genre%5B%5D={}", helper::urlencode(genre_id)));
										}
									}
								} else if is_valid_genre_id(selected_value) {
									// Si ce n'est pas un index, peut-être que c'est directement l'ID
									filter_params.push_str(&format!("&genre%5B%5D={}", helper::urlencode(selected_value)));
								}
							}
						}
					}
				}
				FilterValue::Text { id, value } => {
					// Les filtres Text avec ID genre (pour compatibilité)
					if id == "genre" && !value.is_empty() && is_valid_genre_id(value) {
						filter_params.push_str(&format!("&genre%5B%5D={}", helper::urlencode(value)));
					}
				}
				_ => {
					// Autres types de filtres ignorés pour l'instant
				}
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
		
		// Faire la requête HTTP avec headers réalistes et gestion d'erreur
		let html = match make_realistic_request(&url) {
			Ok(doc) => doc,
			Err(_) => {
				// If request fails (Cloudflare block, rate limit, etc.), return empty result
				return Ok(MangaPageResult {
					entries: Vec::new(),
					has_next_page: false,
				});
			}
		};
		
		// Parser les résultats
		parser::parse_manga_list(html)
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		// Construire l'URL de base du manga (sans /scan/vf/) pour les détails
		let base_manga_url = if manga.key.starts_with("http") {
			// Si c'est une URL complète, s'assurer qu'elle ne contient pas /scan/vf/
			if manga.key.contains("/scan/vf/") || manga.key.contains("/scan_noir-et-blanc/vf/") {
				// Retirer la partie /scan.../vf/... pour avoir l'URL de base
				let parts: Vec<&str> = manga.key.split("/scan").collect();
				if parts.len() > 1 {
					parts[0].to_string()
				} else {
					manga.key.clone()
				}
			} else {
				manga.key.clone()
			}
		} else {
			// S'assurer que manga.key ne contient pas /scan/vf/
			let clean_key = if manga.key.contains("/scan/vf/") || manga.key.contains("/scan_noir-et-blanc/vf/") {
				let parts: Vec<&str> = manga.key.split("/scan").collect();
				if parts.len() > 1 {
					parts[0]
				} else {
					&manga.key
				}
			} else {
				&manga.key
			};
			format!("{}{}", BASE_URL, clean_key)
		};
		
		if needs_details {
			// Faire une requête pour récupérer les détails du manga (URL de base)
			let html = match make_realistic_request(&base_manga_url) {
				Ok(doc) => doc,
				Err(_) => {
					// If request fails, return the original manga without details
					return Ok(manga);
				}
			};
			let detailed_manga = match parser::parse_manga_details(manga.key.clone(), html) {
				Ok(m) => m,
				Err(_) => {
					// If parsing fails, return the original manga
					return Ok(manga);
				}
			};
			
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
			let html = match make_realistic_request(&base_manga_url) {
				Ok(doc) => doc,
				Err(_) => {
					// If request fails, keep original chapters (or None)
					return Ok(manga);
				}
			};
			let chapters = match parser::parse_chapter_list(manga.key.clone(), html) {
				Ok(c) => c,
				Err(_) => {
					// If parsing fails, keep original chapters (or None)
					return Ok(manga);
				}
			};
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Construire l'URL du chapitre si elle n'existe pas
		let chapter_url = chapter.url.unwrap_or_else(|| {
			// Nettoyer manga.key pour s'assurer qu'il ne contient pas déjà /scan/vf/
			let clean_manga_key = if manga.key.starts_with("http") {
				if manga.key.contains("/scan/vf/") || manga.key.contains("/scan_noir-et-blanc/vf/") {
					let parts: Vec<&str> = manga.key.split("/scan").collect();
					if parts.len() > 1 {
						parts[0].to_string()
					} else {
						manga.key.clone()
					}
				} else {
					manga.key.clone()
				}
			} else {
				if manga.key.contains("/scan/vf/") || manga.key.contains("/scan_noir-et-blanc/vf/") {
					let parts: Vec<&str> = manga.key.split("/scan").collect();
					if parts.len() > 1 {
						parts[0].to_string()
					} else {
						manga.key.clone()
					}
				} else {
					manga.key.clone()
				}
			};
			
			// Déterminer le scan path (One Piece special case)
			let is_one_piece = clean_manga_key.contains("one-piece") || clean_manga_key.contains("one_piece");
			let scan_path = if is_one_piece {
				// For One Piece chapters 1046+, try using normal scan path instead of noir-et-blanc
				if let Ok(chapter_num) = chapter.key.parse::<i32>() {
					if chapter_num >= 1046 {
						"/scan/vf/" // Normal scan path for recent chapters
					} else {
						"/scan_noir-et-blanc/vf/" // Noir-et-blanc for older chapters
					}
				} else {
					"/scan_noir-et-blanc/vf/" // Default for One Shot and other special chapters
				}
			} else { 
				"/scan/vf/" 
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
		let html = match make_realistic_request(&chapter_url) {
			Ok(doc) => doc,
			Err(_) => {
				// If request fails, return empty page list
				return Ok(Vec::new());
			}
		};
		
		// Parser les pages depuis le HTML ou utiliser la logique CDN
		parser::parse_page_list(html, manga.key, chapter.key)
	}
}

impl ListingProvider for AnimeSama {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"dernières-sorties" => {
				// Faire une requête vers la page d'accueil pour les dernières sorties
				let html = match make_realistic_request(BASE_URL) {
					Ok(doc) => doc,
					Err(_) => {
						// If request fails, return empty result
						return Ok(MangaPageResult {
							entries: Vec::new(),
							has_next_page: false,
						});
					}
				};
				parser::parse_manga_listing(html, "Dernières Sorties")
			},
			"populaire" => {
				// Faire une requête vers le catalogue pour les mangas populaires
				let url = format!("{}/catalogue?type%5B%5D=Scans&search=&page={}", BASE_URL, page);
				let html = match make_realistic_request(&url) {
					Ok(doc) => doc,
					Err(_) => {
						// If request fails, return empty result
						return Ok(MangaPageResult {
							entries: Vec::new(),
							has_next_page: false,
						});
					}
				};
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