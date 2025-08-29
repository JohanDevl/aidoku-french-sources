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
		let mut query_params = String::new();
		let mut genres_query: Vec<String> = Vec::new();
		
		// Ignorer les filtres pour l'instant
		let _ = filters;

		// Construire l'URL finale - essayer l'API JSON d'abord, puis HTML si bloqué
		if let Some(search_query) = query {
			// Essayer d'abord l'API JSON
			let api_url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			match Request::get(&api_url)?.string() {
				Ok(response) => {
					if response.contains("Just a moment") || response.contains("<!DOCTYPE html>") {
						// Fallback vers HTML parsing
						let html_url = format!("{}/catalogue?search={}", BASE_URL, helper::urlencode(&search_query));
						let html = Request::get(&html_url)?.html()?;
						parser::parse_search_html(html)
					} else {
						parser::parse_search_list(response)
					}
				}
				Err(_) => {
					// Fallback vers HTML parsing
					let html_url = format!("{}/catalogue?search={}", BASE_URL, helper::urlencode(&search_query));
					let html = Request::get(&html_url)?.html()?;
					parser::parse_search_html(html)
				}
			}
		} else {
			// Essayer d'abord l'API JSON
			let genres_query_str = genres_query.join(",");
			let api_url = format!("{}/front/manga?{}&genre={}&page={}&limit=20", API_URL, query_params, genres_query_str, page);
			match Request::get(&api_url)?.string() {
				Ok(response) => {
					if response.contains("Just a moment") || response.contains("<!DOCTYPE html>") {
						// Fallback vers HTML parsing
						let html_url = format!("{}/catalogue?page={}", BASE_URL, page);
						let html = Request::get(&html_url)?.html()?;
						parser::parse_manga_html(html)
					} else {
						parser::parse_manga_list(response)
					}
				}
				Err(_) => {
					// Fallback vers HTML parsing
					let html_url = format!("{}/catalogue?page={}", BASE_URL, page);
					let html = Request::get(&html_url)?.html()?;
					parser::parse_manga_html(html)
				}
			}
		}
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details {
			let url = format!("{}/front/manga/{}", API_URL, manga.key);
			let response = Request::get(&url)?.string()?;
			let detailed_manga = parser::parse_manga_details(manga.key.clone(), response)?;
			
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
			let url = format!("{}/front/manga/{}", API_URL, manga.key);
			let response = Request::get(&url)?.string()?;
			let chapters = parser::parse_chapter_list(manga.key.clone(), response)?;
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
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
		// Essayer d'abord l'API JSON
		let api_url = if listing.name == "Dernières Sorties" {
			format!("{}/front/homepage?page={}&section=latest&limit=20", API_URL, page)
		} else if listing.name == "Populaire" {
			format!("{}/front/homepage?section=top", API_URL)
		} else {
			return Err(aidoku::AidokuError::message("Unimplemented listing"));
		};
		
		match Request::get(&api_url)?.string() {
			Ok(response) => {
				if response.contains("Just a moment") || response.contains("<!DOCTYPE html>") {
					// Fallback vers HTML parsing
					let html_url = format!("{}/", BASE_URL);
					let html = Request::get(&html_url)?.html()?;
					parser::parse_listing_html(html, &listing.name)
				} else {
					parser::parse_manga_listing(response, &listing.name)
				}
			}
			Err(_) => {
				// Fallback vers HTML parsing
				let html_url = format!("{}/", BASE_URL);
				let html = Request::get(&html_url)?.html()?;
				parser::parse_listing_html(html, &listing.name)
			}
		}
	}
}

register_source!(PhenixScans, ListingProvider);