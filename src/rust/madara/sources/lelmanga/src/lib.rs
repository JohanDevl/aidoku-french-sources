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
		url = format!("{}/?s={}&page={}", data.base_url, search_query, page);
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
	// LelManga uses MangaThemesia structure with different selectors than Madara
	let data = get_data();
	let url = format!("{}/{}/{}/", data.base_url, data.source_path, id);
	
	let mut req = aidoku::std::net::Request::new(&url, aidoku::std::net::HttpMethod::Get);
	if let Some(user_agent) = &data.user_agent {
		req = req.header("User-Agent", user_agent);
	}
	
	let html = req.html()?;
	
	// Use MangaThemesia selectors for manga details
	let details_container = html.select("div.bigcontent, div.animefull, div.main-info, div.postbody").first();
	
	// Extract title with MangaThemesia selectors
	let title = if !details_container.select("h1.entry-title").text().read().is_empty() {
		details_container.select("h1.entry-title").text().read()
	} else if !details_container.select(".ts-breadcrumb li:last-child span").text().read().is_empty() {
		details_container.select(".ts-breadcrumb li:last-child span").text().read()
	} else {
		// Fallback: try to find any h1 on the page
		html.select("h1").text().read()
	};
	
	if title.is_empty() {
		return Err(aidoku::error::AidokuError {
			reason: aidoku::error::AidokuErrorKind::Unimplemented,
		});
	}
	
	// Extract cover image
	let cover = if !html.select(".infomanga > div[itemprop=image] img").attr("src").read().is_empty() {
		html.select(".infomanga > div[itemprop=image] img").attr("src").read()
	} else if !html.select(".thumb img").attr("src").read().is_empty() {
		html.select(".thumb img").attr("src").read()
	} else {
		// Try common image selectors as fallback
		if !html.select(".manga-poster img").attr("src").read().is_empty() {
			html.select(".manga-poster img").attr("src").read()
		} else {
			String::new()
		}
	};
	
	// Extract author with French selector
	let author = details_container.select(".imptdt:contains(Auteur) i").text().read();
	
	// Extract artist (if different from author)
	let artist = details_container.select(".imptdt:contains(Artiste) i").text().read();
	
	// Extract description with MangaThemesia selectors
	let description = if !details_container.select(".desc").text().read().is_empty() {
		details_container.select(".desc").text().read()
	} else {
		details_container.select(".entry-content[itemprop=description]").text().read()
	};
	
	// Extract genres
	let mut categories: Vec<String> = Vec::new();
	for genre_element in details_container.select("div.gnr a, .mgen a, .seriestugenre a").array() {
		let genre = genre_element.as_node().expect("node array").text().read();
		if !genre.is_empty() {
			categories.push(genre);
		}
	}
	
	// Extract status
	let status = (data.status)(&details_container);
	
	// Extract content rating
	let nsfw = (data.nsfw)(&details_container, &categories);
	
	// Determine viewer type
	let viewer = (data.viewer)(&details_container, &categories);
	
	Ok(Manga {
		id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw,
		viewer,
	})
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
			url: href,
			lang: data.lang.clone(),
		});
	}
	
	Ok(chapters)
}

fn extract_chapter_number(chapter_id: &str, title: &str) -> f32 {
	// First try to extract from URL ID by looking for "chapitre-XXX" or "chapter-XXX" pattern
	let chapter_id_lower = chapter_id.to_lowercase();
	
	// Look for the pattern "chapitre-" or "chapter-" followed by a number
	if let Some(chapitre_pos) = chapter_id_lower.find("chapitre-") {
		let after_chapitre = &chapter_id[chapitre_pos + 9..]; // 9 is length of "chapitre-"
		if let Some(num_str) = after_chapitre.split('-').next() {
			if let Ok(num) = num_str.parse::<f32>() {
				return num;
			}
		}
	}
	
	if let Some(chapter_pos) = chapter_id_lower.find("chapter-") {
		let after_chapter = &chapter_id[chapter_pos + 8..]; // 8 is length of "chapter-"
		if let Some(num_str) = after_chapter.split('-').next() {
			if let Ok(num) = num_str.parse::<f32>() {
				return num;
			}
		}
	}
	
	// New logic for lelmanga format: manga-name-XXX-Y where XXX is chapter number and Y is version/part
	// Split by '-' and look for the pattern where we have a large number followed by a small number
	let parts: Vec<&str> = chapter_id.split('-').collect();
	if parts.len() >= 2 {
		let last_part = parts[parts.len() - 1];
		let second_last_part = parts[parts.len() - 2];
		
		// Try to parse both parts as numbers
		if let (Ok(last_num), Ok(second_last_num)) = (last_part.parse::<f32>(), second_last_part.parse::<f32>()) {
			// If last number is small (likely a version/part) and second last is larger (likely chapter number)
			if last_num <= 10.0 && second_last_num > 10.0 {
				return second_last_num;
			}
			// If only one big number, prefer the larger one
			if second_last_num > last_num && second_last_num > 10.0 {
				return second_last_num;
			}
		}
	}
	
	// Fallback: try to extract from the last segment of URL (original behavior)
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
	// LelManga uses MangaThemesia structure with JavaScript-loaded images
	let data = get_data();
	let url = format!("{}/{}/", data.base_url, chapter_id);
	
	let mut req = aidoku::std::net::Request::new(&url, aidoku::std::net::HttpMethod::Get);
	if let Some(user_agent) = &data.user_agent {
		req = req.header("User-Agent", user_agent);
	}
	
	let html = req.html()?;
	let mut pages: Vec<Page> = Vec::new();
	
	// First try: Check if images are in HTML (div#readerarea img)
	for (index, img_element) in html.select("div#readerarea img").array().enumerate() {
		let img_node = img_element.as_node().expect("node array");
		
		// Try multiple image attributes (data-lazy-src, data-src, src)
		let img_url = if !img_node.attr("data-lazy-src").read().is_empty() {
			img_node.attr("data-lazy-src").read()
		} else if !img_node.attr("data-src").read().is_empty() {
			img_node.attr("data-src").read()
		} else {
			img_node.attr("src").read()
		};
		
		if !img_url.is_empty() {
			pages.push(Page {
				index: index as i32,
				url: img_url,
				base64: String::new(),
				text: String::new(),
			});
		}
	}
	
	// If HTML parsing succeeded, return the pages
	if !pages.is_empty() {
		return Ok(pages);
	}
	
	// Second try: Parse JavaScript configuration for images
	let html_content = html.text().read();
	
	// Look for ts_reader.run configuration
	// More robust pattern matching for the configuration object
	if let Some(ts_reader_start) = html_content.find("ts_reader.run({") {
		let config_start = ts_reader_start + 15; // Skip "ts_reader.run({"
		if let Some(config_end) = html_content[config_start..].find("})") {
			let config_content = &html_content[config_start..config_start + config_end];
			
			// Look for "sources" key in the configuration
			if let Some(sources_start) = config_content.find("\"sources\":[") {
				let sources_section = &config_content[sources_start + 11..]; // Skip "sources":["
				
				// Find the images array in the first source
				if let Some(images_start) = sources_section.find("\"images\":[") {
					let images_section = &sources_section[images_start + 10..]; // Skip "images":["
					if let Some(images_end) = images_section.find("]") {
						let images_content = &images_section[..images_end];
						
						// Parse image URLs more robustly
						let mut index = 0;
						let parts: Vec<&str> = images_content.split('"').collect();
						
						for part in parts {
							// Check if this part looks like a complete image URL
							if part.starts_with("https://") && part.contains("/wp-content/uploads/") && 
							   (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp") || part.ends_with(".jpeg")) {
								pages.push(Page {
									index,
									url: part.to_string(),
									base64: String::new(),
									text: String::new(),
								});
								index += 1;
							}
						}
					}
				}
			}
		}
	}
	
	// Fallback: Look for any pattern with "images":[...] in the JavaScript
	if pages.is_empty() {
		if let Some(images_pattern_start) = html_content.find("\"images\":[") {
			let images_section = &html_content[images_pattern_start + 10..];
			if let Some(images_end) = images_section.find("]") {
				let images_content = &images_section[..images_end];
				
				// Simple extraction of URLs from the array
				let mut index = 0;
				let parts: Vec<&str> = images_content.split('"').collect();
				
				for part in parts {
					if part.starts_with("https://") && part.contains("lelmanga.com") && 
					   (part.ends_with(".jpg") || part.ends_with(".png") || part.ends_with(".webp")) {
						pages.push(Page {
							index,
							url: part.to_string(),
							base64: String::new(),
							text: String::new(),
						});
						index += 1;
					}
				}
			}
		}
	}
	
	Ok(pages)
}

#[modify_image_request]
fn modify_image_request(request: Request) {
	// LelManga needs specific headers for image loading
	let data = get_data();
	
	// Set appropriate headers for image requests - chain them directly
	let request_with_headers = request
		.header("Referer", &data.base_url)
		.header("Accept", "image/avif,image/webp,image/png,image/jpeg,*/*")
		.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
		.header("Sec-Fetch-Dest", "image")
		.header("Sec-Fetch-Mode", "no-cors")
		.header("Sec-Fetch-Site", "same-origin");
	
	// Add user agent if configured using the template helper
	if let Some(user_agent) = &data.user_agent {
		request_with_headers.header("User-Agent", user_agent);
	}
}

#[handle_url]
pub fn handle_url(url: String) -> Result<DeepLink> {
	template::handle_url(url, get_data())
}