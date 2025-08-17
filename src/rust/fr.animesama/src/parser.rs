use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::{BASE_URL, CDN_URL};

pub fn parse_manga_list(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	
	// Sélecteur pour les éléments manga dans le catalogue
	for element in html.select("#list_catalog > div").array() {
		let element = element.as_node()?;
		
		let title = element.select("h1").text().read();
		if title.is_empty() {
			continue;
		}
		
		let relative_url = element.select("a").attr("href").read();
		let cover_url = element.select("img").attr("src").read();
		
		mangas.push(Manga {
			id: relative_url.clone(),
			cover: cover_url,
			title,
			author: String::new(),
			artist: String::new(),
			description: String::new(),
			url: format!("{}{}", String::from(BASE_URL), relative_url),
			categories: Vec::new(),
			status: MangaStatus::Unknown,
			nsfw: MangaContentRating::Safe,
			viewer: MangaViewer::Scroll
		});
	}
	
	// Vérifier s'il y a une page suivante
	let has_more = html.select("#list_pagination > a.bg-sky-900 + a").html().read().len() > 0;
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more
	})
}

pub fn parse_manga_listing(html: Node, listing_type: &str) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();
	let mut has_more = false;
	
	if listing_type == "Dernières Sorties" {
		// Pour les dernières sorties, utiliser le conteneur spécial de la page d'accueil
		for element in html.select("#containerAjoutsScans > div").array() {
			let element = element.as_node()?;
			
			let title = element.select("h1").text().read();
			if title.is_empty() {
				continue;
			}
			
			let mut relative_url = element.select("a").attr("href").read();
			// Supprimer le suffixe /scan/vf/ s'il existe
			if relative_url.ends_with("/scan/vf/") {
				relative_url = relative_url.replace("/scan/vf/", "");
			}
			
			let cover_url = element.select("img").attr("src").read();
			
			mangas.push(Manga {
				id: relative_url.clone(),
				cover: cover_url,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url: format!("{}{}", String::from(BASE_URL), relative_url),
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
		}
		
		// Les dernières sorties de la page d'accueil n'ont généralement pas de pagination
		has_more = false;
	} else {
		// Pour le populaire, utiliser le même parsing que la liste générale
		return parse_manga_list(html);
	}
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more
	})
}

pub fn parse_manga_details(manga_id: String, html: Node) -> Result<Manga> {
	// Sur la page principale de détails, utiliser les bons sélecteurs
	
	// Titre : h4 dans la section principale
	let title = if !html.select("h4").text().read().is_empty() {
		html.select("h4").text().read()
	} else {
		// Fallback avec h2 si h4 n'existe pas
		if !html.select("h2").first().text().read().is_empty() {
			html.select("h2").first().text().read()
		} else {
			String::from("Manga")
		}
	};
	
	// Cover : img avec alt contenant le titre
	let cover = html.select("img[alt*='Blue Lock'], img[alt*='lock'], img[src*='blue-lock']").attr("src").read();
	
	// Description : le paragraphe le plus long (synopsis)
	let mut description = String::new();
	let paragraphs = html.select("p");
	let mut longest_text = String::new();
	
	for paragraph in paragraphs.array() {
		let paragraph = paragraph.as_node().unwrap();
		let text = paragraph.text().read();
		if text.len() > longest_text.len() && text.len() > 50 {
			longest_text = text;
		}
	}
	
	description = if longest_text.is_empty() {
		String::from("Description non disponible")
	} else {
		longest_text
	};
	
	// Genres : texte après le h2 "Genres"
	let mut categories: Vec<String> = Vec::new();
	let h2_elements = html.select("h2");
	
	for h2_element in h2_elements.array() {
		let h2_element = h2_element.as_node().unwrap();
		if h2_element.text().read() == "Genres" {
			// Récupérer l'élément suivant qui contient les genres
			let next_element = h2_element.select("+ *").first();
			let genre_text = next_element.text().read();
			if !genre_text.is_empty() {
				categories = genre_text.split(',').map(|s| String::from(s.trim())).collect();
				break;
			}
		}
	}
	
	// Fallback pour les catégories
	if categories.is_empty() {
		categories.push(String::from("Manga"));
	};
	
	Ok(Manga {
		id: manga_id.clone(),
		cover,
		title,
		author: String::new(),
		artist: String::new(),
		description,
		url: format!("{}{}", String::from(BASE_URL), manga_id),
		categories,
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

pub fn parse_chapter_list(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Parser le <select> qui contient tous les chapitres disponibles
	let select_options = html.select("select option");
	
	if select_options.array().len() == 0 {
		// Fallback : créer une liste de base si pas de select
		for i in 1..=50 {
			chapters.push(Chapter {
				id: format!("chapitre-{}", i),
				title: format!("Chapitre {}", i),
				volume: -1.0,
				chapter: i as f32,
				date_updated: current_date(),
				scanlator: String::from("AnimeSama"),
				url: format!("{}/scan/vf/chapitre-{}", manga_id, i),
				lang: String::from("fr")
			});
		}
	} else {
		// Utiliser les vraies données du select
		for option in select_options.array() {
			let option = option.as_node().unwrap();
			let option_text = option.text().read();
			
			// Extraire le numéro de chapitre depuis "Chapitre X"
			if option_text.starts_with("Chapitre ") {
				let chapter_num_str = option_text.replace("Chapitre ", "");
				
				// Convertir en nombre
				if let Ok(chapter_num) = chapter_num_str.parse::<i32>() {
					chapters.push(Chapter {
						id: format!("chapitre-{}", chapter_num),
						title: option_text,
						volume: -1.0,
						chapter: chapter_num as f32,
						date_updated: current_date(),
						scanlator: String::from("AnimeSama"),
						url: format!("{}/scan/vf/chapitre-{}", manga_id, chapter_num),
						lang: String::from("fr")
					});
				}
			}
		}
	}
	
	// Inverser l'ordre pour avoir les derniers chapitres en premier
	chapters.reverse();
	
	Ok(chapters)
}


pub fn parse_page_list(_html: Node, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
	let manga_name = manga_id.split('/').last().unwrap_or("manga");
	
	// Extraire le numéro de chapitre depuis l'ID
	let chapter_num = chapter_id.replace("chapitre-", "");
	
	// Générer une liste de pages basique
	// Utiliser le pattern observé d'AnimeSama: {CDN_URL}{manga_name}/{chapter}/{page}.jpg
	for i in 1..=20 {
		let image_url = format!("{}{}/{}/{}.jpg", String::from(CDN_URL), manga_name, chapter_num, i);
		pages.push(Page {
			index: i,
			url: image_url,
			base64: String::new(),
			text: String::new()
		});
	}
	
	Ok(pages)
}