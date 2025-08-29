use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer, UpdateStrategy,
	alloc::{String, Vec, format, string::ToString},
	imports::{std::current_date, html::Document},
	prelude::*,
};

use crate::BASE_URL;

// Parse manga list from HTML (comme AnimeSama)
pub fn parse_manga_list_html(html: Document) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Sélecteurs adaptés pour PhenixScans - à ajuster selon la structure réelle
	if let Some(manga_items) = html.select(".manga-item, .card, .media, .manga-entry") {
		for item in manga_items {
			// Extraire le titre
			let title = item.select("h3, .title, .manga-title, h2")
				.and_then(|els| els.first())
				.and_then(|el| el.text())
				.unwrap_or_default();
			
			// Extraire le lien/key 
			let key = item.select("a")
				.and_then(|els| els.first())
				.and_then(|el| el.attr("href"))
				.unwrap_or_default()
				.replace("/manga/", ""); // Nettoyer pour garder juste l'ID
			
			// Extraire l'image de couverture
			let cover = item.select("img")
				.and_then(|els| els.first())
				.and_then(|el| el.attr("src").or_else(|| el.attr("data-src")))
				.map(|src| {
					if src.starts_with("http") {
						src.to_string()
					} else {
						format!("{}{}", BASE_URL, src)
					}
				});
			
			if !key.is_empty() && !title.is_empty() {
				mangas.push(Manga {
					key,
					title,
					cover,
					authors: None,
					artists: None,
					description: None,
					url: None,
					tags: None,
					status: MangaStatus::Unknown,
					content_rating: ContentRating::Safe,
					viewer: Viewer::default(),
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
	}
	
	// Détecter s'il y a une page suivante
	let has_next_page = html.select(".pagination .next, .next-page").is_some();
	
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page,
	})
}

// Parse les listings spéciaux (Populaire, Dernières Sorties) 
pub fn parse_manga_listing_html(html: Document, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	let selector = if listing_type == "Populaire" {
		".popular-manga .manga-item, .trending .card, .top-manga .item"
	} else {
		".latest-releases .manga-item, .recent .card, .newest .item"
	};
	
	if let Some(items) = html.select(selector) {
		for item in items.take(20) { // Limiter à 20 résultats
			let title = item.select("h3, .title, .manga-title")
				.and_then(|els| els.first())
				.and_then(|el| el.text())
				.unwrap_or_default();
			
			let key = item.select("a")
				.and_then(|els| els.first())
				.and_then(|el| el.attr("href"))
				.unwrap_or_default()
				.replace("/manga/", "");
			
			let cover = item.select("img")
				.and_then(|els| els.first())
				.and_then(|el| el.attr("src").or_else(|| el.attr("data-src")))
				.map(|src| {
					if src.starts_with("http") {
						src.to_string()
					} else {
						format!("{}{}", BASE_URL, src)
					}
				});
			
			if !key.is_empty() && !title.is_empty() {
				mangas.push(Manga {
					key,
					title,
					cover,
					authors: None,
					artists: None,
					description: None,
					url: None,
					tags: None,
					status: MangaStatus::Unknown,
					content_rating: ContentRating::Safe,
					viewer: Viewer::default(),
					chapters: None,
					next_update_time: None,
					update_strategy: UpdateStrategy::Never,
				});
			}
		}
	}
	
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: false, // Les listings spéciaux ont généralement une seule page
	})
}

// Parse les détails d'un manga 
pub fn parse_manga_details_html(manga_id: String, html: Document) -> Result<Manga> {
	let title = html.select(".manga-title, h1, .title")
		.and_then(|els| els.first())
		.and_then(|el| el.text())
		.unwrap_or_else(|| "Unknown Title".to_string());
	
	let cover = html.select(".manga-cover img, .cover img, .thumbnail img")
		.and_then(|els| els.first())
		.and_then(|el| el.attr("src").or_else(|| el.attr("data-src")))
		.map(|src| {
			if src.starts_with("http") {
				src.to_string()
			} else {
				format!("{}{}", BASE_URL, src)
			}
		});
	
	let description = html.select(".description, .summary, .synopsis")
		.and_then(|els| els.first())
		.and_then(|el| el.text())
		.or_else(|| Some("Aucune description disponible.".to_string()));
	
	let url = Some(format!("{}/manga/{}", BASE_URL, manga_id));
	
	// Extraire les tags/genres si disponibles
	let mut tags = Vec::new();
	if let Some(tag_elements) = html.select(".genres .genre, .tags .tag") {
		for tag in tag_elements {
			if let Some(tag_text) = tag.text() {
				tags.push(tag_text);
			}
		}
	}
	
	Ok(Manga {
		key: manga_id,
		title,
		cover,
		authors: None, // Pourrait être extrait si présent dans le HTML
		artists: None,
		description,
		url,
		tags: if tags.is_empty() { None } else { Some(tags) },
		status: MangaStatus::Unknown, // Pourrait être parsé depuis le HTML
		content_rating: ContentRating::Safe,
		viewer: Viewer::default(),
		chapters: None,
		next_update_time: None,
		update_strategy: UpdateStrategy::Never,
	})
}

// Parse la liste des chapitres 
pub fn parse_chapter_list_html(manga_id: String, html: Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	if let Some(chapter_elements) = html.select(".chapter-list .chapter, .chapters .item") {
		for (index, chapter) in chapter_elements.enumerate() {
			let title = chapter.select("a, .chapter-title")
				.and_then(|els| els.first())
				.and_then(|el| el.text())
				.unwrap_or_else(|| format!("Chapter {}", index + 1));
			
			let chapter_url = chapter.select("a")
				.and_then(|els| els.first())
				.and_then(|el| el.attr("href"))
				.map(|href| {
					if href.starts_with("http") {
						href.to_string()
					} else {
						format!("{}{}", BASE_URL, href)
					}
				});
			
			// Extraire le numéro de chapitre du titre ou de l'URL
			let chapter_number = (index + 1) as f32;
			let key = format!("{}", chapter_number);
			
			chapters.push(Chapter {
				key,
				title: Some(title),
				volume_number: Some(-1.0),
				chapter_number: Some(chapter_number),
				date_uploaded: Some(current_date()),
				scanlators: None,
				url: chapter_url,
				language: Some("fr".to_string()),
				thumbnail: None,
				locked: false,
			});
		}
	}
	
	Ok(chapters)
}

// Parse la liste des pages d'un chapitre
pub fn parse_page_list_html(html: Document) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Chercher les images dans le lecteur
	if let Some(page_images) = html.select(".reader-page img, .pages img, .chapter-content img") {
		for img in page_images {
			if let Some(src) = img.attr("src").or_else(|| img.attr("data-src")) {
				let image_url = if src.starts_with("http") {
					src.to_string()
				} else {
					format!("{}{}", BASE_URL, src)
				};
				
				pages.push(Page {
					content: PageContent::url(image_url),
					thumbnail: None,
					has_description: false,
					description: None,
				});
			}
		}
	}
	
	Ok(pages)
}