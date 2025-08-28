#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, vec, string::ToString},
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

		// Construire l'URL finale
		if let Some(search_query) = query {
			let url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			let response = Request::get(&url)?.string()?;
			let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
			let json = json_value.as_object()?;
			parser::parse_search_list(json)
		} else {
			let genres_query_str = genres_query.join(",");
			let url = format!("{}/front/manga?{}&genre={}&page={}&limit=20", API_URL, query_params, genres_query_str, page);
			let response = Request::get(&url)?.string()?;
			let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
			let json = json_value.as_object()?;
			parser::parse_manga_list(json)
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
			let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
			let json = json_value.as_object()?;
			let detailed_manga = parser::parse_manga_details(manga.key.clone(), json)?;
			
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
			let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
			let json = json_value.as_object()?;
			let chapters = parser::parse_chapter_list(manga.key.clone(), json)?;
			manga.chapters = Some(chapters);
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = Request::get(&url)?.string()?;
		let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
		let json = json_value.as_object()?;
		parser::parse_page_list(json)
	}
}

impl ListingProvider for PhenixScans {
	fn get_manga_list(
		&self,
		listing: Listing,
		page: i32,
	) -> Result<MangaPageResult> {
		let url = if listing.name == "Dernières Sorties" {
			format!("{}/front/homepage?page={}&section=latest&limit=20", API_URL, page)
		} else if listing.name == "Populaire" {
			format!("{}/front/homepage?section=top", API_URL)
		} else {
			return Err(aidoku::AidokuError::message("Unimplemented listing"));
		};
		
		let response = Request::get(&url)?.string()?;
		let json_value = aidoku::prelude::ValueRef::new_string(&response)?.json()?;
		let json = json_value.as_object()?;
		parser::parse_manga_listing(json, &listing.name)
	}
}