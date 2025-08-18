#![no_std]
use aidoku::{
	prelude::*,
	error::Result,
	std::{
		net::{Request, HttpMethod},
		String, Vec
	},
	Filter, FilterType, Listing, Manga, MangaPageResult, Page, Chapter
};

mod parser;
mod helper;

pub static BASE_URL: &str = "https://anime-sama.fr";
pub static CDN_URL: &str = "https://anime-sama.fr/s2/scans";

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let _query = String::new();
	let mut genres_query: Vec<String> = Vec::new();
	let mut search_query = String::new();
	
	for filter in filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(value) = filter.value.as_string() {
					search_query = helper::urlencode(&value.read());
				}
			}
			FilterType::Genre => {
				if let Ok(filter_id) = filter.object.get("id").as_string() {
					genres_query.push(helper::urlencode(&filter_id.read()));
				}
			}
			_ => continue,
		}
	}

	let url = if search_query.is_empty() {
		let genres_param = if genres_query.is_empty() {
			String::new()
		} else {
			format!("&genre[]={}", genres_query.join("&genre[]="))
		};
		format!("{}/catalogue?type[0]=Scans&page={}{}", String::from(BASE_URL), helper::i32_to_string(page), genres_param)
	} else {
		format!("{}/catalogue?type[0]=Scans&search={}&page={}", String::from(BASE_URL), search_query, helper::i32_to_string(page))
	};

	let html = Request::new(&url, HttpMethod::Get).html()?;
	parser::parse_manga_list(html)
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let url = if listing.name == "Dernières Sorties" {
		String::from(BASE_URL)
	} else if listing.name == "Populaire" {
		format!("{}/catalogue?type[0]=Scans&page={}", String::from(BASE_URL), helper::i32_to_string(page))
	} else {
		return Err(aidoku::error::AidokuError { reason: aidoku::error::AidokuErrorKind::Unimplemented });
	};

	let html = Request::new(&url, HttpMethod::Get).html()?;
	parser::parse_manga_listing(html, &listing.name)
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let dummy_html = Request::new("https://anime-sama.fr/", HttpMethod::Get).html()?;
	parser::parse_manga_details(manga_id, dummy_html)
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	// Requête vers la vraie page /scan/vf/ pour parser le select des chapitres
	let url = format!("{}{}/scan/vf/", String::from(BASE_URL), manga_id);
	let html = Request::new(url, HttpMethod::Get).html()?;
	parser::parse_chapter_list(manga_id, html)
}

#[get_page_list]
fn get_page_list(manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	// Le chapter_id contient déjà l'URL complète vers episodes.js
	// On n'a pas besoin du HTML pour AnimeSama, on utilise directement episodes.js
	let empty_html = aidoku::std::html::Node::new_fragment("")?;
	parser::parse_page_list(empty_html, manga_id, chapter_id)
}