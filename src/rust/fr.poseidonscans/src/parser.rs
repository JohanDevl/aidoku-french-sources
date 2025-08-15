use aidoku::{
	error::Result, prelude::*, std::{
		html::Node, ObjectRef, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;

// Parse search results with client-side filtering from /series page
pub fn parse_search_manga(search_query: String, html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let query = search_query.to_lowercase();
	
	// If search query is empty, return first few manga
	if query.trim().is_empty() {
		return parse_popular_manga(html);
	}

	// Select all manga links from the series page
	for link in html.select("a[href*='/serie/']").array() {
		let link = link.as_node()?;
		let href = link.attr("href").read();
		
		// Extract slug from URL
		let slug = if let Some(slug_start) = href.rfind('/') {
			String::from(&href[slug_start + 1..])
		} else {
			continue;
		};
		
		if slug.is_empty() {
			continue;
		}
		
		// Extract title from h2 element
		let title_node = link.select("h2").first();
		let title = if title_node.html().read().len() > 0 {
			title_node.text().read()
		} else {
			// Fallback: get title from image alt attribute
			let img_node = link.select("img").first();
			if img_node.html().read().len() > 0 {
				img_node.attr("alt").read()
			} else {
				continue;
			}
		};
		
		if title.is_empty() {
			continue;
		}
		
		// Extract description for additional search context
		let desc_node = link.select("p").first();
		let description = if desc_node.html().read().len() > 0 {
			desc_node.text().read()
		} else {
			String::new()
		};
		
		// Client-side filtering: check if query matches title, slug, or description
		let title_lower = title.to_lowercase();
		let slug_lower = slug.to_lowercase();
		let desc_lower = description.to_lowercase();
		
		let matches = title_lower.contains(&query) ||
					  slug_lower.contains(&query) ||
					  desc_lower.contains(&query);

		if !matches {
			continue;
		}
		
		// Extract cover image URL (same logic as popular manga)
		let img_node = link.select("img").first();
		let cover = if img_node.html().read().len() > 0 {
			let src = img_node.attr("src").read();
			if src.starts_with("http") {
				src
			} else if src.starts_with("/") {
				format!("{}{}", String::from(BASE_URL), src)
			} else {
				format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
			}
		} else {
			format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
		};
		
		// Truncate description for display
		let display_description = if description.len() > 200 {
			format!("{}...", &description[..200])
		} else {
			description
		};
		
		// Parse status (simplified)
		let status = MangaStatus::Unknown;
		
		// Extract categories
		let mut categories: Vec<String> = Vec::new();
		for element in link.select("div").array() {
			let element = element.as_node()?;
			let text_content = element.text().read();
			let text = text_content.trim();
			if text.len() > 2 && text.len() < 25 && !text.contains(" chapitres") && 
			   !text.contains("En cours") && !text.contains("Terminé") &&
			   !text.chars().any(|c| c.is_numeric()) {
				categories.push(String::from(text));
				if categories.len() >= 5 {
					break;
				}
			}
		}
		
		let manga_url = format!("{}/serie/{}", String::from(BASE_URL), slug);
		
		mangas.push(Manga {
			id: slug,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description: display_description,
			url: manga_url,
			categories,
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
		
		// Limit search results
		if mangas.len() >= 30 {
			break;
		}
	}
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more: false, // No pagination for search
	})
}

// Parse latest manga from API response  
pub fn parse_latest_manga(json: ObjectRef) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;

	if let Ok(data_array) = json.get("data").as_array() {
		for item in data_array {
			let manga = item.as_object()?;
			
			let slug = manga.get("slug").as_string()?.read();
			if slug == "unknown" || slug.is_empty() {
				continue;
			}
			
			let title = manga.get("title").as_string()?.read();
			let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug);

			mangas.push(Manga {
				id: slug,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: String::new(),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
		}
		
		has_more = mangas.len() == 20;
	}

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// Parse popular manga from /series HTML page
pub fn parse_popular_manga(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	// Select all manga links from the series page
	for link in html.select("a[href*='/serie/']").array() {
		let link = link.as_node()?;
		let href = link.attr("href").read();
		
		// Extract slug from URL (e.g., "/serie/sand-mage-of-the-burnt-desert" -> "sand-mage-of-the-burnt-desert")
		let slug = if let Some(slug_start) = href.rfind('/') {
			String::from(&href[slug_start + 1..])
		} else {
			continue;
		};
		
		if slug.is_empty() {
			continue;
		}
		
		// Extract title from h2 element
		let title_node = link.select("h2").first();
		let title = if title_node.html().read().len() > 0 {
			title_node.text().read()
		} else {
			// Fallback: get title from image alt attribute
			let img_node = link.select("img").first();
			if img_node.html().read().len() > 0 {
				img_node.attr("alt").read()
			} else {
				continue;
			}
		};
		
		if title.is_empty() {
			continue;
		}
		
		// Extract cover image URL
		let img_node = link.select("img").first();
		let cover = if img_node.html().read().len() > 0 {
			let src = img_node.attr("src").read();
			if src.starts_with("http") {
				src
			} else if src.starts_with("/") {
				format!("{}{}", String::from(BASE_URL), src)
			} else {
				// Fallback: use API cover pattern
				format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
			}
		} else {
			format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
		};
		
		// Extract description from paragraph
		let desc_node = link.select("p").first();
		let description = if desc_node.html().read().len() > 0 {
			let desc = desc_node.text().read();
			if desc.len() > 200 {
				format!("{}...", &desc[..200])
			} else {
				desc
			}
		} else {
			String::new()
		};
		
		// Parse status from text content (simplified)
		let status = MangaStatus::Unknown;
		
		// Extract genres/categories
		let mut categories: Vec<String> = Vec::new();
		// Look for genre containers (elements with single-word text that look like genres)
		for element in link.select("div").array() {
			let element = element.as_node()?;
			let text_content = element.text().read();
			let text = text_content.trim();
			if text.len() > 2 && text.len() < 25 && !text.contains(" chapitres") && 
			   !text.contains("En cours") && !text.contains("Terminé") &&
			   !text.chars().any(|c| c.is_numeric()) {
				categories.push(String::from(text));
				if categories.len() >= 5 { // Limit to 5 categories
					break;
				}
			}
		}
		
		let manga_url = format!("{}/serie/{}", String::from(BASE_URL), slug);
		
		mangas.push(Manga {
			id: slug,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description,
			url: manga_url,
			categories,
			status,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
		
		// Limit to avoid too many results on first load
		if mangas.len() >= 20 {
			break;
		}
	}
	
	let has_more = mangas.len() >= 20;
	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

// Parse manga details (simplified)  
pub fn parse_manga_details(manga_id: String, _html: Node) -> Result<Manga> {
	// Simplified implementation - would need real Next.js data extraction
	let title = format!("Manga {}", manga_id);
	let cover = format!("{}/api/covers/{}.webp", String::from(BASE_URL), manga_id);
	let url = format!("{}/serie/{}", String::from(BASE_URL), manga_id);
	
	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description: String::from("Aucune description disponible."),
		url,
		categories: Vec::new(),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

// Parse chapter list (simplified)
pub fn parse_chapter_list(_manga_id: String, _html: Node) -> Result<Vec<Chapter>> {
	let chapters: Vec<Chapter> = Vec::new();
	
	// This would need real Next.js data extraction  
	// For now, return empty list to avoid compilation errors
	
	Ok(chapters)
}

// Parse page list (simplified)
pub fn parse_page_list(_html: Node) -> Result<Vec<Page>> {
	let pages: Vec<Page> = Vec::new();
	
	// This would need real Next.js data extraction for image URLs
	// For now, return empty list
	
	Ok(pages)
}