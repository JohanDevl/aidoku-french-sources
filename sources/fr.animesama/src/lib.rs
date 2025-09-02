#![no_std]

use aidoku::{
	Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, PageContext, Result, Source,
	alloc::{String, Vec, vec, string::ToString},
	imports::{net::Request, std::send_partial_result},
	prelude::*,
};


// Modules contenant la logique de parsing sophistiquée d'AnimeSama
pub mod parser;
pub mod helper;

pub const BASE_URL: &str = "https://anime-sama.fr";
pub const CDN_URL: &str = "https://s22.anime-sama.me/s1/scans";

// Vérifier si un ID de genre est valide
fn is_valid_genre_id(genre_id: &str) -> bool {
	matches!(genre_id, 
		"action" | "aventure" | "combat" | "comedie" | "drame" | "ecchi" | "fantasy" |
		"harem" | "historique" | "horreur" | "isekai" | "josei" | "magie" | "arts-martiaux" |
		"mature" | "mystere" | "psychologique" | "romance" | "school-life" | "sci-fi" |
		"seinen" | "shoujo" | "shounen" | "slice-of-life" | "sports" | "supernatural" |
		"thriller" | "tragedie"
	)
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
							// Mapping des indices vers les IDs de genre (basé sur filters.json)
							let genre_ids = [
								"action", "aventure", "combat", "comedie", "drame", "ecchi", "fantasy", 
								"harem", "historique", "horreur", "isekai", "josei", "magie", "arts-martiaux",
								"mature", "mystere", "psychologique", "romance", "school-life", "sci-fi",
								"seinen", "shoujo", "shounen", "slice-of-life", "sports", "supernatural", 
								"thriller", "tragedie"
							];
							
							if selected_index >= 0 && (selected_index as usize) < genre_ids.len() {
								let genre_id = genre_ids[selected_index as usize];
								filter_params.push_str(&format!("&genre[]={}", helper::urlencode(genre_id)));
							}
						} else if is_valid_genre_id(value) {
							// Si ce n'est pas un index, peut-être que c'est directement l'ID
							filter_params.push_str(&format!("&genre[]={}", helper::urlencode(value)));
						}
					}
				}
				FilterValue::Text { id, value } => {
					// Les filtres Text avec ID genre
					if id == "genre" && !value.is_empty() && is_valid_genre_id(value) {
						filter_params.push_str(&format!("&genre[]={}", helper::urlencode(value)));
					}
				}
				_ => {
					// Autres types de filtres ignorés pour l'instant
				}
			}
		}
		
		// Construire l'URL de recherche pour anime-sama.fr
		// Essayer différents ordres de paramètres selon si on a une recherche ou pas
		let url = if let Some(search_query) = query {
			// Avec recherche : mettre search en premier
			format!("{}/catalogue?search={}&type[0]=Scans{}&page={}", 
				BASE_URL, 
				helper::urlencode(&search_query),
				filter_params,
				page
			)
		} else {
			// Sans recherche : ordre normal
			format!("{}/catalogue?type[0]=Scans{}&page={}", BASE_URL, filter_params, page)
		};
		
		// Faire la requête HTTP
		let html = Request::get(&url)?.html()?;
		
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
			let html = Request::get(&base_manga_url)?.html()?;
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
			let html = Request::get(&base_manga_url)?.html()?;
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
			let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
			
			if clean_manga_key.starts_with("http") {
				format!("{}{}?id={}", clean_manga_key, scan_path, chapter.key)
			} else {
				format!("{}{}{}?id={}", BASE_URL, clean_manga_key, scan_path, chapter.key)
			}
		});
		
		// Faire une requête pour récupérer la page du chapitre
		let html = Request::get(&chapter_url)?.html()?;
		
		// Parser les pages depuis le HTML ou utiliser la logique CDN
		parser::parse_page_list(html, manga.key, chapter.key)
	}
}

impl ListingProvider for AnimeSama {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"dernières-sorties" => {
				// Faire une requête vers la page d'accueil pour les dernières sorties
				let html = Request::get(BASE_URL)?.html()?;
				parser::parse_manga_listing(html, "Dernières Sorties")
			},
			"populaire" => {
				// Faire une requête vers le catalogue pour les mangas populaires
				let url = format!("{}/catalogue?type[0]=Scans&page={}", BASE_URL, page);
				let html = Request::get(&url)?.html()?;
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