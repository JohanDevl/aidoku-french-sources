#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, vec},
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
		let mut url = format!("{}/catalogue?type[0]=Scans&page={}", BASE_URL, page);
		
		// Ajouter la query de recherche si fournie
		if let Some(search_query) = query {
			url.push_str(&format!("&search={}", helper::urlencode(&search_query)));
		}
		
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
		let manga_url = format!("{}{}", BASE_URL, manga.key);
		
		if needs_details {
			// Faire une requête pour récupérer les détails du manga
			let html = Request::get(&manga_url)?.html()?;
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
			// Faire une requête pour récupérer la liste des chapitres
			let html = Request::get(&manga_url)?.html()?;
			let chapters = parser::parse_chapter_list(manga.key.clone(), html)?;
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Construire l'URL du chapitre si elle n'existe pas
		let chapter_url = chapter.url.unwrap_or_else(|| {
			format!("{}{}/scan/vf/{}", BASE_URL, manga.key, chapter.key)
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