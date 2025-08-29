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

		// Utiliser les vrais endpoints API comme l'ancienne version
		if let Some(search_query) = query {
			// Endpoint de recherche
			let url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			let response = Request::get(&url)?.string()?;
			parser::parse_search_list(response)
		} else {
			// Endpoint de listing
			let url = format!("{}/front/manga?page={}&limit=20", API_URL, page);
			let response = Request::get(&url)?.string()?;
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
			// Utiliser le vrai endpoint API
			let url = format!("{}/front/manga/{}", API_URL, manga.key);
			let response = Request::get(&url)?.string()?;
			
			if needs_details {
				let detailed_manga = parser::parse_manga_details(manga.key.clone(), response.clone())?;
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
				let chapters = parser::parse_chapter_list(manga.key.clone(), response)?;
				manga.chapters = Some(chapters);
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Utiliser le vrai endpoint API pour les pages
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = Request::get(&url)?.string()?;
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
		
		// Faire la requête API JSON
		let response = Request::get(&url)?.string()?;
		parser::parse_manga_listing(response, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);