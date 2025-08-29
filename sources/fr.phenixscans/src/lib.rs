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
			// Endpoint de recherche avec headers Cloudflare
			let url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Accept-Encoding", "gzip, deflate, br")
				.header("Referer", "https://phenix-scans.com/")
				.header("Origin", "https://phenix-scans.com")
				.string()?;
			
			// Debug: afficher les premiers 500 caractères de la réponse
			let debug_msg = format!("SEARCH DEBUG - URL: {} | Response (first 500 chars): {}", url, 
				if response.len() > 500 { &response[..500] } else { &response });
			return Err(aidoku::AidokuError::message(&debug_msg));
			
			// parser::parse_search_list(response)
		} else {
			// Endpoint de listing avec headers Cloudflare
			let url = format!("{}/front/manga?page={}&limit=20", API_URL, page);
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Accept-Encoding", "gzip, deflate, br")
				.header("Referer", "https://phenix-scans.com/")
				.header("Origin", "https://phenix-scans.com")
				.string()?;
			
			// Debug: afficher les premiers 500 caractères de la réponse
			let debug_msg = format!("LIST DEBUG - URL: {} | Response (first 500 chars): {}", url, 
				if response.len() > 500 { &response[..500] } else { &response });
			return Err(aidoku::AidokuError::message(&debug_msg));
			
			// parser::parse_manga_list(response)
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
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Referer", "https://phenix-scans.com/")
				.string()?;
			
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
		// Utiliser le vrai endpoint API pour les pages avec headers Cloudflare
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
			.header("Accept", "application/json, text/plain, */*")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Referer", "https://phenix-scans.com/")
			.string()?;
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
		let response = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
			.header("Accept", "application/json, text/plain, */*")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Referer", "https://phenix-scans.com/")
			.string()?;
		
		// Debug: afficher les premiers 500 caractères de la réponse
		let debug_msg = format!("LISTING DEBUG - Type: {} | URL: {} | Response (first 500 chars): {}", 
			listing.name, url, if response.len() > 500 { &response[..500] } else { &response });
		return Err(aidoku::AidokuError::message(&debug_msg));
		
		// parser::parse_manga_listing(response, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);