use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::{html::{Document, Element}},
	prelude::*,
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
	// Parser les détails d'un manga depuis sa page
	let title = html.select("#titreOeuvre, h1, .manga-title")
		.and_then(|els| els.text())
		.unwrap_or("Titre inconnu".into());
	
	let description = html.select(".description, .synopsis, .manga-description")
		.and_then(|els| els.text())
		.map(|text| text.trim().to_string());
	
	let cover = html.select("img.manga-cover, .cover img, .poster img")
		.and_then(|els| els.first())
		.and_then(|el| el.attr("src"));
	
	// Tags/genres - version simplifiée
	let mut tags = Vec::new();
	if let Some(tag_elements) = html.select(".genres a, .tags a, .genre") {
		for tag_el in tag_elements {
			if let Some(tag_text) = tag_el.text() {
				tags.push(tag_text);
			}
		}
	}
	
	Ok(Manga {
		key: manga_key.clone(),
		title,
		authors: None,
		artists: None,
		description,
		url: Some(format!("{}{}", BASE_URL, manga_key)),
		cover,
		tags: if !tags.is_empty() { Some(tags) } else { None },
		status: MangaStatus::Ongoing,
		content_rating: ContentRating::Safe,
		viewer: Viewer::RightToLeft,
		..Default::default()
	})
}

pub fn parse_chapter_list(manga_key: String, html: Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Chercher les chapitres dans différents sélecteurs possibles
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
					url: Some(format!("{}{}", BASE_URL, chapter_url)),
					..Default::default()
				});
			}
			break; // Stop dès qu'on trouve des chapitres
		}
	}
	
	// Si pas de chapitres trouvés, créer des chapitres de test
	if chapters.is_empty() {
		for i in 1i32..=5 {
			chapters.push(Chapter {
				key: i.to_string(),
				title: Some(format!("Chapitre {}", i)),
				chapter_number: Some(i as f32),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec!["AnimeSama".into()]),
				url: Some(format!("{}/catalogue/{}/scan/vf/{}", BASE_URL, manga_key.trim_start_matches("/catalogue/"), i)),
				..Default::default()
			});
		}
	}
	
	Ok(chapters)
}

pub fn parse_page_list(_html: Document, manga_key: String, chapter_key: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Extraire le nom du manga depuis la clé
	let manga_name = manga_key.split('/').last().unwrap_or("manga");
	let chapter_num = chapter_key.parse::<i32>().unwrap_or(1);
	
	// Générer des URLs de pages basées sur la structure CDN d'AnimeSama
	// Les images sont généralement dans: https://anime-sama.fr/s2/scans/{manga_name}/{chapter_num}/{page_num}.jpg
	for i in 1i32..=20 { // Maximum 20 pages par chapitre
		let page_url = format!("{}/{}/{}/{:03}.jpg", CDN_URL, manga_name, chapter_num, i);
		pages.push(Page {
			content: PageContent::url(page_url),
			has_description: false,
			description: None,
			thumbnail: None,
		});
	}
	
	Ok(pages)
}