#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, string::ToString},
	imports::net::Request,
	prelude::*,
};

mod parser;
mod helper;

pub const BASE_URL: &str = "https://phenix-scans.com";
pub const API_URL: &str = "https://phenix-scans.com/api";

struct PhenixScans;

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
		// Ignorer les filtres pour l'instant  
		let _ = filters;

		// Construire l'URL HTML comme AnimeSama - approche directe
		let mut url = format!("{}/manga?page={}", BASE_URL, page);
		
		// Ajouter la recherche si fournie
		if let Some(search_query) = query {
			url.push_str(&format!("&search={}", helper::urlencode(&search_query)));
		}
		
		// Faire la requête HTML directement (comme AnimeSama)
		let html = Request::get(&url)?.html()?;
		
		// Parser les résultats
		parser::parse_manga_list_html(html)
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		// Construire l'URL HTML pour les détails manga (comme AnimeSama)
		let manga_url = if manga.key.starts_with("http") {
			manga.key.clone()
		} else {
			format!("{}/manga/{}", BASE_URL, manga.key)
		};

		if needs_details {
			let html = Request::get(&manga_url)?.html()?;
			let detailed_manga = parser::parse_manga_details_html(manga.key.clone(), html)?;
			
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

		if needs_chapters {
			let html = Request::get(&manga_url)?.html()?;
			let chapters = parser::parse_chapter_list_html(manga.key.clone(), html)?;
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Construire l'URL du chapitre (comme AnimeSama)
		let chapter_url = if chapter.url.is_some() {
			chapter.url.unwrap()
		} else {
			format!("{}/manga/{}/chapitre/{}", BASE_URL, manga.key, chapter.key)
		};
		
		let html = Request::get(&chapter_url)?.html()?;
		parser::parse_page_list_html(html)
	}
}

impl ListingProvider for PhenixScans {
	fn get_manga_list(
		&self,
		listing: Listing,
		page: i32,
	) -> Result<MangaPageResult> {
		// Construire l'URL HTML selon le type de listing (comme AnimeSama)
		let url = match listing.id.as_str() {
			"dernières-sorties" => {
				// Page d'accueil ou section récente
				format!("{}/", BASE_URL)
			},
			"populaire" => {
				// Catalogue avec tri par popularité
				format!("{}/manga?page={}&sort=popular", BASE_URL, page)
			},
			_ => {
				return Err(aidoku::AidokuError::message("Unimplemented listing"));
			}
		};
		
		// Faire la requête HTML directement
		let html = Request::get(&url)?.html()?;
		
		// Parser selon le type de listing
		parser::parse_manga_listing_html(html, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);