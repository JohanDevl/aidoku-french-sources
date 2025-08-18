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
	// LelManga uses MangaThemesia structure with chapters in HTML, not AJAX
	let data = get_data();
	let url = format!("{}/{}/{}/", data.base_url, data.source_path, id);
	
	let mut req = aidoku::std::net::Request::new(&url, aidoku::std::net::HttpMethod::Get);
	if let Some(user_agent) = &data.user_agent {
		req = req.header("User-Agent", user_agent);
	}
	
	let html = req.html()?;
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Use MangaThemesia selectors for chapters
	for item in html.select("div.bxcl li, div.cl li, #chapterlist li, ul li:has(div.chbox):has(div.eph-num)").array() {
		let obj = item.as_node().expect("node array");
		
		let link = obj.select("a").first();
		let href = link.attr("href").read();
		if href.is_empty() {
			continue;
		}
		
		// Extract chapter ID from URL
		let chapter_id = href
			.replace(&data.base_url, "")
			.trim_start_matches('/')
			.trim_end_matches('/')
			.to_string();
		
		if chapter_id.is_empty() {
			continue;
		}
		
		// Get chapter title with fallbacks
		let title = if !obj.select(".lch a").text().read().is_empty() {
			obj.select(".lch a").text().read()
		} else if !obj.select(".chapternum").text().read().is_empty() {
			obj.select(".chapternum").text().read()
		} else {
			link.text().read()
		};
		
		if title.is_empty() {
			continue;
		}
		
		// Extract chapter number from URL or title
		let chapter_num = extract_chapter_number(&chapter_id, &title);
		
		// Parse chapter date
		let date_str = obj.select(".chapterdate").text().read();
		let date_updated = parse_chapter_date(&date_str);
		
		chapters.push(Chapter {
			id: chapter_id,
			title,
			volume: -1.0,
			chapter: chapter_num,
			date_updated,
			scanlator: String::new(),
			url: String::new(),
			lang: data.lang.clone(),
		});
	}
	
	Ok(chapters)
}

fn extract_chapter_number(chapter_id: &str, title: &str) -> f32 {
	// First try to extract from URL ID
	if let Some(num_str) = chapter_id.split('-').last() {
		if let Ok(num) = num_str.parse::<f32>() {
			return num;
		}
	}
	
	// Then try to extract from title
	// Look for patterns like "Chapitre 123" or "Chapter 123"
	let words: Vec<&str> = title.split_whitespace().collect();
	for (i, word) in words.iter().enumerate() {
		if word.to_lowercase().contains("chapitre") || word.to_lowercase().contains("chapter") {
			if i + 1 < words.len() {
				if let Ok(num) = words[i + 1].parse::<f32>() {
					return num;
				}
			}
		}
	}
	
	// Last resort: try to find any number in the title
	for word in words {
		if let Ok(num) = word.parse::<f32>() {
			return num;
		}
	}
	
	-1.0
}

fn parse_chapter_date(date_str: &str) -> f64 {
	// LelManga uses English date format: "MMMM d, yyyy"
	// Examples: "August 17, 2025", "May 8, 2022"
	
	if date_str.is_empty() {
		return 0.0;
	}
	
	// Simple date parsing for common English month names
	let months = [
		("January", 1), ("February", 2), ("March", 3), ("April", 4),
		("May", 5), ("June", 6), ("July", 7), ("August", 8),
		("September", 9), ("October", 10), ("November", 11), ("December", 12)
	];
	
	let parts: Vec<&str> = date_str.trim().split_whitespace().collect();
	if parts.len() >= 3 {
		// Try to parse "Month Day, Year" format
		let month_name = parts[0];
		let day_str = parts[1].trim_end_matches(',');
		let year_str = parts[2];
		
		if let Some((_, month)) = months.iter().find(|(name, _)| name.eq_ignore_ascii_case(month_name)) {
			if let (Ok(day), Ok(year)) = (day_str.parse::<i32>(), year_str.parse::<i32>()) {
				// Convert to timestamp (simplified)
				// This is a rough approximation: days since 1970-01-01
				let days_since_1970 = (year - 1970) * 365 + (month - 1) * 30 + day;
				return (days_since_1970 as f64) * 86400.0; // seconds in a day
			}
		}
	}
	
	0.0
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