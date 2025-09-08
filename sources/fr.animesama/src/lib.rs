#![no_std]

use aidoku::{
	Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, PageContext, Result, Source,
	alloc::{String, Vec, vec, string::ToString},
	imports::{net::Request, std::send_partial_result},
	prelude::*,
};

extern crate alloc;
use alloc::format;


// Modules contenant la logique de parsing sophistiquée d'AnimeSama
pub mod parser;
pub mod helper;

pub const BASE_URL: &str = "https://anime-sama.org";
pub const CDN_URL: &str = "https://anime-sama.org/s2/scans";
pub const CDN_URL_LEGACY: &str = "https://s22.anime-sama.me/s1/scans";

// Inclure le contenu de filters.json au moment de la compilation
static FILTERS_JSON: &str = include_str!("../res/filters.json");

// Parser les IDs de genres depuis le JSON inclus
fn get_genre_ids() -> Vec<&'static str> {
	// Parsing simple du JSON pour extraire les IDs
	// Chercher "ids": [ et extraire les valeurs entre guillemets
	
	// Cette approche simple évite d'ajouter une dépendance JSON
	// On cherche la section "ids": [
	if let Some(ids_start) = FILTERS_JSON.find("\"ids\": [") {
		let ids_section = &FILTERS_JSON[ids_start + 8..];
		if let Some(ids_end) = ids_section.find(']') {
			let ids_content = &ids_section[..ids_end];
			
			// Extraire chaque ID entre guillemets
			let mut ids = Vec::new();
			let mut current_pos = 0;
			
			while let Some(quote_start) = ids_content[current_pos..].find('"') {
				let quote_start = current_pos + quote_start + 1;
				if let Some(quote_end) = ids_content[quote_start..].find('"') {
					let quote_end = quote_start + quote_end;
					let id = &ids_content[quote_start..quote_end];
					
					// Convertir en &'static str en utilisant match sur les valeurs connues
					let static_id = match id {
						"action" => "action",
						"aventure" => "aventure", 
						"combat" => "combat",
						"comedie" => "comedie",
						"drame" => "drame",
						"ecchi" => "ecchi",
						"fantasy" => "fantasy",
						"harem" => "harem",
						"historique" => "historique",
						"horreur" => "horreur",
						"isekai" => "isekai",
						"josei" => "josei",
						"magie" => "magie",
						"arts-martiaux" => "arts-martiaux",
						"mature" => "mature",
						"mystere" => "mystere",
						"psychologique" => "psychologique",
						"romance" => "romance",
						"school-life" => "school-life",
						"sci-fi" => "sci-fi",
						"seinen" => "seinen",
						"shoujo" => "shoujo",
						"shounen" => "shounen",
						"slice-of-life" => "slice-of-life",
						"sports" => "sports",
						"supernatural" => "supernatural",
						"thriller" => "thriller",
						"tragedie" => "tragedie",
						_ => continue, // Ignorer les IDs inconnus
					};
					
					ids.push(static_id);
					current_pos = quote_end + 1;
				} else {
					break;
				}
			}
			
			return ids;
		}
	}
	
	// Fallback si le parsing échoue
	vec![]
}

// Vérifier si un ID de genre est valide
fn is_valid_genre_id(genre_id: &str) -> bool {
	get_genre_ids().contains(&genre_id)
}

// Ajouter les en-têtes Cloudflare-friendly à une requête avec retry logic
fn add_cloudflare_headers(request: Request) -> Request {
	request
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8,application/signed-exchange;v=b3;q=0.7")
		.header("Accept-Language", "fr-FR,fr;q=0.9,en-US;q=0.8,en;q=0.7")
		.header("Accept-Encoding", "gzip, deflate, br")
		.header("Connection", "keep-alive")
		.header("Upgrade-Insecure-Requests", "1")
		.header("Sec-Fetch-Dest", "document")
		.header("Sec-Fetch-Mode", "navigate")
		.header("Sec-Fetch-Site", "cross-site")
		.header("Sec-Fetch-User", "?1")
		.header("Pragma", "no-cache")
		.header("Cache-Control", "no-cache")
}

// Fonction pour faire une requête avec retry en cas de Cloudflare  
fn make_cloudflare_request(url: &str) -> Result<aidoku::imports::html::Document> {
	// Première tentative
	let response = add_cloudflare_headers(Request::get(url)?).html();
	
	match response {
		Ok(doc) => {
			// Vérifier si c'est une page Cloudflare en regardant le titre ou contenu
			if let Some(title) = doc.select("title").and_then(|els| els.first()).and_then(|el| el.text()) {
				if title.contains("anime-sama.org") && 
				   (title.contains("vérifier") || title.contains("security") || title.contains("Cloudflare")) {
					// Page de vérification détectée, essayer une approche différente
					return make_simple_request(url);
				}
			}
			Ok(doc)
		}
		Err(_) => {
			// Si échec, essayer approche simplifiée
			make_simple_request(url)
		}
	}
}

// Requête simplifiée sans tous les en-têtes Cloudflare
fn make_simple_request(url: &str) -> Result<aidoku::imports::html::Document> {
	Request::get(url)?
		.header("User-Agent", "Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1")
		.html()
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
				FilterValue::Select { id, value } => {
					// Vérifier que c'est le filtre de genre et qu'il a une valeur
					if id == "genre" && !value.is_empty() {
						// Essayer de parser la valeur comme un index
						if let Ok(selected_index) = value.parse::<i32>() {
							// Utiliser les IDs de genre depuis filters.json
							let genre_ids = get_genre_ids();
							
							if selected_index >= 0 && (selected_index as usize) < genre_ids.len() {
								let genre_id = genre_ids[selected_index as usize];
								// Format correct pour AnimeSama
								if !genre_id.is_empty() {
									filter_params.push_str(&format!("&genre[]={}", genre_id));
								}
							}
						} else if is_valid_genre_id(value) {
							// Si ce n'est pas un index, peut-être que c'est directement l'ID
							filter_params.push_str(&format!("&genre[]={}", value));
						}
					}
				}
				FilterValue::Text { id, value } => {
					// Les filtres Text avec ID genre
					if id == "genre" && !value.is_empty() && is_valid_genre_id(value) {
						filter_params.push_str(&format!("&genre[]={}", value));
					}
				}
				_ => {
					// Autres types de filtres ignorés pour l'instant
				}
			}
		}
		
		// Construire l'URL de recherche pour anime-sama.org
		// Essayer différents ordres de paramètres selon si on a une recherche ou pas
		let url = if let Some(search_query) = query {
			// Avec recherche : mettre search en premier
			format!("{}/catalogue?search={}{}&type[0]=Scans&page={}", 
				BASE_URL, 
				helper::urlencode(&search_query),
				filter_params,
				page
			)
		} else {
			// Sans recherche : mettre les filtres genre avant le type
			if filter_params.is_empty() {
				format!("{}/catalogue?type[0]=Scans&page={}", BASE_URL, page)
			} else {
				// Enlever le & au début de filter_params
				let clean_params = if filter_params.starts_with('&') {
					&filter_params[1..]
				} else {
					&filter_params
				};
				format!("{}/catalogue?{}&type[0]=Scans&page={}", BASE_URL, clean_params, page)
			}
		};
		
		// Faire la requête HTTP avec protection Cloudflare
		let html = make_cloudflare_request(&url)?;
		
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
			let html = make_cloudflare_request(&base_manga_url)?;
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
			let html = make_cloudflare_request(&base_manga_url)?;
			let chapters = parser::parse_chapter_list(manga.key.clone(), html)?;
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
		let html = make_cloudflare_request(&chapter_url)?;
		
		// Parser les pages depuis le HTML ou utiliser la logique CDN
		parser::parse_page_list(html, manga.key, chapter.key)
	}
}

impl ListingProvider for AnimeSama {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"dernières-sorties" => {
				// Faire une requête vers la page d'accueil pour les dernières sorties
				let html = make_cloudflare_request(BASE_URL)?;
				parser::parse_manga_listing(html, "Dernières Sorties")
			},
			"populaire" => {
				// Faire une requête vers le catalogue pour les mangas populaires
				let url = format!("{}/catalogue?type[0]=Scans&page={}", BASE_URL, page);
				let html = make_cloudflare_request(&url)?;
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
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36")
			.header("Referer", BASE_URL))
	}
}

register_source!(AnimeSama, ListingProvider, ImageRequestProvider);