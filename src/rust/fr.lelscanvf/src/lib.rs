#![no_std]
use aidoku::{
	prelude::*, error::Result, std::String, std::Vec, std::net::Request, std::net::HttpMethod,
	Filter, Manga, MangaPageResult, Page, Chapter, DeepLink
};

mod parser;

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	let mut result: Vec<Manga> = Vec::new();

	let mut url = String::new();
	parser::get_filtered_url(filters, page, &mut url);
	let html = Request::new(url.as_str(), HttpMethod::Get)
		.header(
			"Referer", 
			"https://lelscanvf.com/"
		).html();

	if url.contains("search") {
		parser::parse_search(html, &mut result);
	} else if url.contains("filterList") {
		parser::parse_filterlist(html, &mut result);	
	} else {
		parser::parse_recents(html, &mut result);
	}

	if result.len() >= 50 {
		Ok(MangaPageResult {
			manga: result,
			has_more: true,
		})
	} else {
		Ok(MangaPageResult {
			manga: result,
			has_more: false,
		})
	}
}

#[get_manga_details]
fn get_manga_details(manga_id: String) -> Result<Manga> {
	let url = format!("https://lelscanvf.com/manga/{}", &manga_id);
	let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
	return parser::parse_manga(html, manga_id);
}

#[get_chapter_list]
fn get_chapter_list(manga_id: String) -> Result<Vec<Chapter>> {
	let url = format!("https://lelscanvf.com/manga/{}", &manga_id);
	let html = Request::new(url.clone().as_str(), HttpMethod::Get).html();
	return parser::get_chaper_list(html);
}

#[get_page_list]
fn get_page_list(chapter_id: String) -> Result<Vec<Page>> {
	let url = format!("https://lelscanvf.com/manga/{}", &chapter_id);
	let html = Request::new(url.clone().as_str(), HttpMethod::Get)
	.header(
		"Referer", 
		"https://lelscanvf.com/"
	).html();
	return parser::get_page_list(html);
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	let parsed_manga_id = parser::parse_incoming_url(url);

	Ok(DeepLink {
        manga: Some(get_manga_details(parsed_manga_id.clone())?),
        chapter: None
	})
}