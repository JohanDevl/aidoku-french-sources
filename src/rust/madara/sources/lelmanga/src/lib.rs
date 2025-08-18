#![no_std]
use aidoku::{
	error::Result, prelude::*, std::net::Request, std::String, std::Vec, Chapter, DeepLink, Filter, FilterType, Listing, Manga,
	MangaPageResult, MangaStatus, MangaContentRating, Page,
};
use madara_template::template;

extern crate alloc;
use alloc::string::ToString;

fn get_data() -> template::MadaraSiteData {
	let data: template::MadaraSiteData = template::MadaraSiteData {
		base_url: String::from("https://www.lelmanga.com"),
		lang: String::from("fr"),
		source_path: String::from("manga"),
		search_path: String::from(""),
		search_selector: String::from(".utao .uta .imgu, .listupd .bs .bsx"),
		base_id_selector: String::from("a"),
		description_selector: String::from(".desc, .entry-content[itemprop=description]"),
		author_selector: String::from(".imptdt:contains(Auteur) i"),
		date_format: String::from("MMMM d, yyyy"),
		status_filter_ongoing: String::from("En cours"),
		status_filter_completed: String::from("Terminé"),
		status_filter_cancelled: String::from("Annulé"),
		status_filter_on_hold: String::from("En pause"),
		popular: String::from("Populaire"),
		trending: String::from("Tendance"),
		status: |html| {
			let status_str = html
				.select("div.post-content_item:contains(Statut) div.summary-content")
				.text()
				.read()
				.trim()
				.to_lowercase();
			match status_str.as_str() {
				"en cours" | "ongoing" => MangaStatus::Ongoing,
				"terminé" | "completed" => MangaStatus::Completed,
				"annulé" | "cancelled" => MangaStatus::Cancelled,
				"en pause" | "hiatus" => MangaStatus::Hiatus,
				_ => MangaStatus::Unknown,
			}
		},
		nsfw: |_html, categories| {
			let suggestive_tags = ["ecchi", "mature", "adult"];
			
			for tag in suggestive_tags {
				if categories.iter().any(|v| v.to_lowercase() == tag) {
					return MangaContentRating::Suggestive;
				}
			}

			MangaContentRating::Safe
		},
		alt_ajax: true,
		user_agent: Some(String::from("Mozilla/5.0 (iPhone; CPU iPhone OS 14_6 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/14.0.3 Mobile/15E148 Safari/604.1")),
		..Default::default()
	};
	data
}

#[get_manga_list]
fn get_manga_list(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	// LelManga uses MangaThemesia structure, not standard Madara AJAX
	// Use direct page parsing instead of AJAX
	let data = get_data();
	let mut url = format!("{}/manga/", data.base_url);
	
	// Add page parameter for pagination
	if page > 1 {
		url.push_str(&format!("?page={}", page));
	}
	
	// For search, check if there are filters
	let mut search_query = String::new();
	let mut has_search = false;
	
	for filter in &filters {
		match filter.kind {
			FilterType::Title => {
				if let Ok(filter_value) = filter.value.clone().as_string() {
					let title = filter_value.read();
					if !title.is_empty() {
						search_query = title.to_string();
						has_search = true;
						break;
					}
				}
			}
			_ => {}
		}
	}
	
	if has_search {
		// Use search URL format
		url = format!("{}/?title={}&page={}", data.base_url, search_query, page);
	}
	
	get_manga_from_page(url, data)
}

fn get_manga_from_page(url: String, data: template::MadaraSiteData) -> Result<MangaPageResult> {
	let mut req = aidoku::std::net::Request::new(&url, aidoku::std::net::HttpMethod::Get);
	if let Some(user_agent) = &data.user_agent {
		req = req.header("User-Agent", user_agent);
	}
	
	let html = req.html()?;
	let mut manga: Vec<Manga> = Vec::new();
	
	// Use MangaThemesia selectors
	for item in html.select(".utao .uta .imgu, .listupd .bs .bsx, .page-listing-item").array() {
		let obj = item.as_node().expect("node array");
		
		let link = obj.select("a").first();
		let href = link.attr("href").read();
		if href.is_empty() {
			continue;
		}
		
		// Extract manga ID from URL
		let id = href
			.replace(&data.base_url, "")
			.replace(&format!("/{}/", data.source_path), "")
			.trim_start_matches('/')
			.trim_end_matches('/')
			.to_string();
		
		if id.is_empty() {
			continue;
		}
		
		let title = link.attr("title").read();
		let title = if title.is_empty() {
			obj.select("a .slide-caption h3, .bsx h3, .post-title h3").text().read()
		} else {
			title
		};
		
		if title.is_empty() {
			continue;
		}
		
		// Get cover image with multiple fallbacks
		let cover_img = obj.select("img").first();
		let cover = if !cover_img.attr("data-lazy-src").read().is_empty() {
			cover_img.attr("data-lazy-src").read()
		} else if !cover_img.attr("data-src").read().is_empty() {
			cover_img.attr("data-src").read()
		} else {
			cover_img.attr("src").read()
		};
		
		manga.push(Manga {
			id,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: String::new(),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: aidoku::MangaViewer::Scroll,
		});
	}
	
	// Check for pagination
	let has_more = !html.select(".pagination .next, .hpage .r").array().is_empty();
	
	Ok(MangaPageResult { manga, has_more })
}

#[get_manga_listing]
fn get_manga_listing(listing: Listing, page: i32) -> Result<MangaPageResult> {
	let data = get_data();
	let mut url = format!("{}/manga/", data.base_url);
	
	// Add sorting parameter based on listing type
	if listing.name == data.popular {
		url.push_str("?order=popular");
	} else if listing.name == data.trending {
		url.push_str("?order=update");
	} else {
		url.push_str("?order=latest");
	}
	
	// Add page parameter
	if page > 1 {
		if url.contains('?') {
			url.push_str(&format!("&page={}", page));
		} else {
			url.push_str(&format!("?page={}", page));
		}
	}
	
	get_manga_from_page(url, data)
}

#[get_manga_details]
fn get_manga_details(id: String) -> Result<Manga> {
	template::get_manga_details(id, get_data())
}

#[get_chapter_list]
fn get_chapter_list(id: String) -> Result<Vec<Chapter>> {
	template::get_chapter_list(id, get_data())
}

#[get_page_list]
fn get_page_list(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	template::get_page_list(chapter_id, get_data())
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	template::modify_image_request(String::from("lelmanga.com"), request, get_data());
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}