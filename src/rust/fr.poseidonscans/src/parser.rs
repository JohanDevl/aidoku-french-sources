use aidoku::{
	error::Result, prelude::*, std::{
		html::Node, ObjectRef, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;


// Parse search results with client-side filtering on /series page
pub fn parse_search_manga(search_query: String, html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let query_lower = search_query.to_lowercase();
	
	// Parse all manga from series page and filter by search query
	for link in html.select("a[href*='/serie/']").array() {
		let link = link.as_node()?;
		let href = link.attr("href").read();
		
		// Extract slug from URL
		let slug = if let Some(slug_start) = href.rfind('/') {
			String::from(&href[slug_start + 1..])
		} else {
			continue;
		};
		
		if slug == "unknown" || slug.is_empty() {
			continue;
		}
		
		// Get title and description for filtering
		let title_element = link.select(".title, .manga-title, h3, .name").first();
		let title = if title_element.html().is_empty() {
			String::from(&slug.replace("-", " "))
		} else {
			title_element.text().read()
		};
		
		let description_element = link.select(".description, .summary, .manga-excerpt").first();
		let description = description_element.text().read();
		
		// Client-side search filtering
		let title_lower = title.to_lowercase();
		let description_lower = description.to_lowercase();
		let slug_lower = slug.to_lowercase();
		
		if !title_lower.contains(&query_lower) && 
		   !description_lower.contains(&query_lower) && 
		   !slug_lower.contains(&query_lower) {
			continue;
		}
		
		// Get cover image
		let img_element = link.select("img").first();
		let cover = if !img_element.html().is_empty() {
			let src = img_element.attr("src").read();
			let data_src = img_element.attr("data-src").read();
			let lazy_src = img_element.attr("data-wpfc-original-src").read();
			
			if !data_src.is_empty() {
				String::from(data_src)
			} else if !lazy_src.is_empty() {
				String::from(lazy_src)
			} else if !src.is_empty() {
				String::from(src)
			} else {
				format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
			}
		} else {
			format!("{}/api/covers/{}.webp", String::from(BASE_URL), slug)
		};
		
		let url = format!("{}/serie/{}", String::from(BASE_URL), slug);
		
		mangas.push(Manga {
			id: slug,
			cover,
			title,
			author: String::new(),
			artist: String::new(),
			description,
			url,
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
	}
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more: false,
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