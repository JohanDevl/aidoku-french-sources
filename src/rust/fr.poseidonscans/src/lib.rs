#![no_std]
use aidoku::{
	prelude::*,
	error::Result,
	std::{
		net::{Request,HttpMethod},
		String, Vec
	},
	Filter, FilterType, Listing, Manga, MangaPageResult, Page, Chapter
};

mod parser;
mod helper;

pub static BASE_URL: &str = "https://poseidonscans.com";
pub static API_URL: &str = "https://poseidonscans.com/api";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, _page: i32) -> Result<MangaPageResult> {
	let mut search_query = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					search_query = value.read();
				}
			}
			_ => continue,
		}
	}

	if search_query.is_empty() {
		// Return empty results for now - PoseidonScans search needs special handling
		Ok(MangaPageResult {
			manga: Vec::new(),
			has_more: false,
		})
	} else {
		// Basic search implementation - can be enhanced later
		parser::parse_search_manga(search_query)
	}
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	if listing.name == "Dernières Sorties" {
		// Use API endpoint for latest chapters
		let url = format!("{}/manga/lastchapters?page={}&limit=20", String::from(API_URL), helper::i32_to_string(page));
		let json = Request::new(&url, HttpMethod::Get).json()?.as_object()?;
		parser::parse_latest_manga(json)
	} else if listing.name == "Populaire" {
		// Popular manga from homepage
		let url = String::from(BASE_URL);
		let html = Request::new(&url, HttpMethod::Get).html()?;
		parser::parse_popular_manga(html)
	} else {
		Ok(MangaPageResult {
			manga: Vec::new(),
			has_more: false,
		})
	}
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/serie/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::parse_manga_details(manga_id, html)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/serie/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::parse_chapter_list(manga_id, html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/serie/{}/chapter/{}", String::from(BASE_URL), manga_id, chapter_id);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::parse_page_list(html)
}
