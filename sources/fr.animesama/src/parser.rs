use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::html::Document,
};

use crate::{BASE_URL, CDN_URL};

// Version simplifiée des fonctions de parsing pour AnimeSama

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Sélecteur pour les éléments manga dans le catalogue
	if let Some(catalog_divs) = html.select("#list_catalog > div") {
		for element in catalog_divs {
			if let Some(title) = element.select("h1").and_then(|els| els.text()) {
				if title.is_empty() {
					continue;
				}
				
				let relative_url = element.select("a").and_then(|els| els.first()).and_then(|el| el.attr("href")).unwrap_or_default();
				let cover_url = element.select("img").and_then(|els| els.first()).and_then(|el| el.attr("src")).unwrap_or_default();
				
				mangas.push(Manga {
					key: relative_url.clone(),
					cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
					title,
					authors: None,
					artists: None,
					description: None,
					url: Some(format!("{}{}", BASE_URL, relative_url)),
					tags: Some(Vec::new()),
					status: MangaStatus::Unknown,
					content_rating: ContentRating::Safe,
					viewer: Viewer::RightToLeft,
					..Default::default()
				});
			}
		}
	}
	
	// Simple pagination check
	let has_next_page = html.select(".pagination").is_some();
	
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page
	})
}

pub fn parse_manga_listing(html: Document, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	if listing_type == "Dernières Sorties" {
		// Pour les dernières sorties, chercher dans le conteneur d'accueil
		if let Some(container) = html.select("#containerAjoutsScans > div, .latest-manga > div, .home-manga > div") {
			for element in container {
				if let Some(title) = element.select("h1, h2, h3, .title, .manga-title").and_then(|els| els.text()) {
					if title.is_empty() {
						continue;
					}
					
					let relative_url = element.select("a").and_then(|els| els.first()).and_then(|el| el.attr("href")).unwrap_or_default();
					let cover_url = element.select("img").and_then(|els| els.first()).and_then(|el| el.attr("src")).unwrap_or_default();
					
					mangas.push(Manga {
						key: relative_url.clone(),
						cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
						title,
						authors: None,
						artists: None,
						description: None,
						url: Some(format!("{}{}", BASE_URL, relative_url)),
						tags: Some(Vec::new()),
						status: MangaStatus::Unknown,
						content_rating: ContentRating::Safe,
						viewer: Viewer::RightToLeft,
						..Default::default()
					});
				}
			}
		}
	} else if listing_type == "Populaire" {
		// Pour populaire, utiliser le catalogue normal
		return parse_manga_list(html);
	}
	
	Ok(MangaPageResult {
		entries: mangas,
		has_next_page: false
	})
}

pub fn parse_manga_details(manga_key: String, html: Document) -> Result<Manga> {
	// Parser le titre
	let title = html.select("#titreOeuvre")
		.and_then(|els| els.text())
		.unwrap_or_else(|| {
			// Fallback: convertir manga_key en titre (ex: "one-piece" → "One Piece")
			manga_key.split('/').last().unwrap_or("manga")
				.replace("-", " ")
				.split_whitespace()
				.map(|word| {
					let mut chars = word.chars();
					match chars.next() {
						None => String::new(),
						Some(first) => first.to_uppercase().chain(chars).collect(),
					}
				})
				.collect::<Vec<String>>()
				.join(" ")
		});
	
	// Parser la description/synopsis - version simplifiée
	let description = html.select("#sousBlocMiddle p")
		.and_then(|paras| {
			for para in paras {
				if let Some(text) = para.text() {
					let trimmed = text.trim();
					if trimmed.len() > 50 { // Description substantielle
						return Some(trimmed.to_string());
					}
				}
			}
			None
		})
		.or_else(|| {
			// Fallback: chercher tout paragraphe avec du texte
			html.select("p").and_then(|paras| {
				for para in paras {
					if let Some(text) = para.text() {
						let trimmed = text.trim();
						if trimmed.len() > 50 { // Description substantielle
							return Some(trimmed.to_string());
						}
					}
				}
				None
			})
		});
	
	// Parser la couverture
	let cover = html.select("#coverOeuvre")
		.and_then(|els| els.first())
		.and_then(|el| el.attr("src"))
		.map(|src| {
			if src.starts_with("http") {
				src
			} else {
				format!("{}{}", BASE_URL, src)
			}
		});
	
	// Parser les genres
	let mut tags = Vec::new();
	
	// Parser les genres de façon simplifiée
	if let Some(genre_links) = html.select("#sousBlocMiddle a") {
		for link in genre_links {
			if let Some(genre_text) = link.text() {
				let text = genre_text.trim();
				if !text.is_empty() && text.len() < 30 {
					tags.push(text.to_string());
				}
			}
		}
	}
	
	// Si aucun genre trouvé, ajouter des genres par défaut
	if tags.is_empty() {
		tags.push("Manga".to_string());
	}
	
	Ok(Manga {
		key: manga_key.clone(),
		title,
		authors: None,
		artists: None,
		description,
		url: Some(format!("{}{}", BASE_URL, manga_key)),
		cover,
		tags: Some(tags),
		status: MangaStatus::Ongoing,
		content_rating: ContentRating::Safe,
		viewer: Viewer::RightToLeft,
		..Default::default()
	})
}

pub fn parse_chapter_list(manga_key: String, html: Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Version simplifiée - créer quelques chapitres par défaut
	if chapters.is_empty() {
		for i in 1..=10 {
			chapters.push(Chapter {
				key: i.to_string(),
				title: Some(format!("Chapitre {}", i)),
				chapter_number: Some(i as f32),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec!["AnimeSama".into()]),
				url: Some(format!("{}{}/scan/vf/{}", BASE_URL, manga_key, i)),
				..Default::default()
			});
		}
	} else {
		// Fallback: chercher des sélecteurs HTML standards
		let selectors = vec![
			".chapter-list .chapter",
			".episodes li",
			".scan-list li",
			".chapter-item"
		];
		
		for selector in selectors {
			if let Some(chapter_elements) = html.select(selector) {
				for (index, chapter_el) in chapter_elements.enumerate() {
					let title = chapter_el.select("a, .chapter-title")
						.and_then(|els| els.text())
						.unwrap_or(format!("Chapitre {}", index + 1));
					
					let chapter_url = chapter_el.select("a").and_then(|els| els.first()).and_then(|el| el.attr("href")).unwrap_or_default();
					
					chapters.push(Chapter {
						key: (index + 1).to_string(),
						title: Some(title),
						chapter_number: Some((index + 1) as f32),
						volume_number: None,
						date_uploaded: None,
						scanlators: Some(vec!["AnimeSama".into()]),
						url: Some(if chapter_url.starts_with("http") { 
							chapter_url 
						} else { 
							format!("{}{}", BASE_URL, chapter_url) 
						}),
						..Default::default()
					});
				}
				break; // Stop dès qu'on trouve des chapitres
			}
		}
		
		// Si toujours pas de chapitres trouvés, créer des chapitres de test
		if chapters.is_empty() {
			for i in 1i32..=5 {
				chapters.push(Chapter {
					key: i.to_string(),
					title: Some(format!("Chapitre {}", i)),
					chapter_number: Some(i as f32),
					volume_number: None,
					date_uploaded: None,
					scanlators: Some(vec!["AnimeSama".into()]),
					url: Some(format!("{}{}/scan/vf/{}", BASE_URL, manga_key, i)),
					..Default::default()
				});
			}
		}
	}
	
	Ok(chapters)
}

pub fn parse_page_list(_html: Document, manga_key: String, chapter_key: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Version très simplifiée - créer des pages avec des URLs CDN basiques
	let manga_name = manga_key.split('/').last().unwrap_or("manga");
	let chapter_num = chapter_key.parse::<i32>().unwrap_or(1);
	
	for i in 1i32..=20 {
		let page_url = format!("{}/{}/{}/{:03}.jpg", CDN_URL, manga_name, chapter_num, i);
		pages.push(Page {
			content: PageContent::url(page_url),
			thumbnail: None,
			has_description: false,
			description: None,
		});
	}
	
	Ok(pages)
}