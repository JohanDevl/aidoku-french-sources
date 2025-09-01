#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, vec, string::ToString},
	imports::{net::Request, std::send_partial_result},
	prelude::*,
};


// Modules contenant la logique de parsing sophistiquée d'AnimeSama
pub mod parser;
pub mod helper;

pub const BASE_URL: &str = "https://anime-sama.fr";
pub const CDN_URL: &str = "https://anime-sama.fr/s2/scans";

struct AnimeSama;

impl Source for AnimeSama {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// Construire l'URL de recherche pour anime-sama.fr
		// Essayer différents ordres de paramètres selon si on a une recherche ou pas
		let url = if let Some(search_query) = query {
			// Avec recherche : mettre search en premier
			format!("{}/catalogue?search={}&type[0]=Scans&page={}", 
				BASE_URL, 
				helper::urlencode(&search_query), 
				page
			)
		} else {
			// Sans recherche : ordre normal
			format!("{}/catalogue?type[0]=Scans&page={}", BASE_URL, page)
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

register_source!(AnimeSama, ListingProvider);