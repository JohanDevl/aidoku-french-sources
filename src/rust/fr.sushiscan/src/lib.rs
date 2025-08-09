#![no_std]
use aidoku::{
	prelude::*,
	error::Result,
	std::{
		net::{Request,HttpMethod},
		String, Vec
	},
	Filter, FilterType, Manga, MangaPageResult, Page, Chapter, Listing, DeepLink
};

mod parser;
mod helper;

const BASE_URL: &str = "https://sushiscan.net";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut query = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					query.push_str(format!("&s={}", helper::urlencode(value.read())).as_str());
				}
			}
			FilterType::Select => {
				if filter.name == "Status" {
					match filter.value.as_int().unwrap_or(-1) {
						0 => query.push_str(""),
						1 => query.push_str("&status=ongoing"),
						2 => query.push_str("&status=completed"),
						3 => query.push_str("&status=hiatus"),
						4 => query.push_str("&status=cancelled"),
						_ => continue,
					}
				}
			}
			_ => continue,
		}
	}
	
	let url = format!("{}/page/{}?{}", String::from(BASE_URL), helper::i32_to_string(page), query);
	let html = Request::new(&url, HttpMethod::Get)
		.header("Referer", BASE_URL)
		.html()?;
	parser::parse_manga_list(html)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = match listing.name.as_str() {
		"Populaire" => format!("{}/catalogue/?page={}&order=popular", BASE_URL, page),
		"Dernières" => format!("{}/catalogue/?page={}&order=update", BASE_URL, page),
		"Nouveau" => format!("{}/catalogue/?page={}&order=latest", BASE_URL, page),
		_ => format!("{}/catalogue/?page={}&order=latest", BASE_URL, page),
	};
	
	let html = Request::new(&url, HttpMethod::Get)
		.header("Referer", BASE_URL)
		.html()?;
	parser::parse_manga_listing(html)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("{}/catalogue/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get)
		.header("Referer", BASE_URL)
		.html()?;
	parser::parse_manga_details(String::from(BASE_URL), manga_id, html)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("{}/catalogue/{}", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get)
		.header("Referer", BASE_URL)
		.html()?;
	parser::parse_chapter_list(String::from(BASE_URL), manga_id, html)
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("{}/{}", String::from(BASE_URL), chapter_id);
	let html = Request::new(url, HttpMethod::Get)
		.header("Referer", BASE_URL)
		.html()?;
	parser::parse_page_list(html)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	request
		.header("Referer", BASE_URL)
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36");
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let manga_id = String::from(url.split('/').last().unwrap_or(""));
	Ok(DeepLink {
		manga: get_manga_details(manga_id).ok(),
		chapter: None,
	})
}