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

pub static BASE_URL: &str = "https://fmteam.fr";
pub static API_URL: &str = "https://fmteam.fr/api";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	let mut search_query = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					search_query = helper::urlencode(value.read());
				}
			}
			FilterType::Select => {
				if filter.name == "Status" {
					let index = filter.value.as_int().unwrap_or(-1);
					match index {
						1 => query.push_str("&status=ongoing"),
						2 => query.push_str("&status=completed"), 
						3 => query.push_str("&status=hiatus"),
						4 => query.push_str("&status=cancelled"),
						_ => continue,
					}
				}
				if filter.name == "Type" {
					let index = filter.value.as_int().unwrap_or(-1);
					match index {
						1 => query.push_str("&type=manga"),
						2 => query.push_str("&type=manhwa"),
						3 => query.push_str("&type=manhua"),
						4 => query.push_str("&type=novel"),
						_ => continue,
					}
				}
			}
			_ => continue,
		}
	}

	if !search_query.is_empty() {
		let url = format!("{}/search/{}", String::from(API_URL), search_query);
		let json = Request::new(&url, HttpMethod::Get).json()?.as_object()?;
		parser::parse_search_list(json)
	} else {
		let url = format!("{}/comics?page={}&limit=20{}", String::from(API_URL), helper::i32_to_string(page), query);
		let json = Request::new(&url, HttpMethod::Get).json()?.as_object()?;
		parser::parse_manga_list(json)
	}
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = if listing.name == "Dernières Sorties" {
		// Get all comics and filter for those with latest chapters
		format!("{}/comics", String::from(API_URL))
	} else if listing.name == "Populaire" {
		// Get all comics (popular will be handled in parser)
		format!("{}/comics", String::from(API_URL))
	} else {
		format!("{}/comics", String::from(API_URL))
	};
	
	let json = Request::new(&url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_manga_listing(json, &listing.name, page)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/comics/{}", String::from(API_URL), manga_id);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_manga_details(manga_id, json)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/comics/{}", String::from(API_URL), manga_id);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_chapter_list(manga_id, json)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/comics/{}/chapters/{}", String::from(API_URL), manga_id, chapter_id);
	let json = Request::new(url, HttpMethod::Get).json()?.as_object()?;
	parser::parse_page_list(json)
}