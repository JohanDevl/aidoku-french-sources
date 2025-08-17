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
	// Extraire titre avec valeur par défaut
	let title = if !html.select("#titreOeuvre").text().read().is_empty() {
		html.select("#titreOeuvre").text().read()
	} else {
		String::from("Manga")
	};
	
	let cover = html.select("#coverOeuvre").attr("src").read();
	
	// Essayer plusieurs méthodes pour la description
	let mut description = String::new();
	
	// Méthode 1: Tous les paragraphes dans sousBlocMiddle
	let paragraphs = html.select("#sousBlocMiddle p");
	if paragraphs.array().len() > 0 {
		description = paragraphs.first().text().read();
	}
	
	// Méthode 2: Si vide, chercher dans tout sousBlocMiddle
	if description.is_empty() {
		let all_text = html.select("#sousBlocMiddle").text().read();
		if all_text.len() > 100 { // Si il y a du contenu substantiel
			// Extraire une partie qui semble être une description
			let words: Vec<&str> = all_text.split_whitespace().collect();
			if words.len() > 10 {
				description = words[..50.min(words.len())].join(" ");
			}
		}
	}
	
	// Valeur par défaut si toujours vide
	if description.is_empty() {
		description = String::from("Description non disponible");
	}
	
	// Essayer plusieurs méthodes pour les genres
	let mut genre_text = String::new();
	
	// Méthode 1: Tous les liens dans sousBlocMiddle
	let links = html.select("#sousBlocMiddle a");
	if links.array().len() > 0 {
		genre_text = links.first().text().read();
	}
	
	// Méthode 2: Chercher du texte contenant des virgules (probablement des genres)
	if genre_text.is_empty() {
		let all_text = html.select("#sousBlocMiddle").text().read();
		let lines: Vec<&str> = all_text.split('\n').collect();
		for line in lines {
			if line.contains(',') && line.len() < 200 && line.len() > 5 {
				genre_text = String::from(line.trim());
				break;
			}
		}
	}
	
	// Convertir les genres en Vec<String>
	let categories: Vec<String> = if genre_text.is_empty() {
		let mut default_cats = Vec::new();
		default_cats.push(String::from("Manga"));
		default_cats
	} else {
		genre_text.split(',').map(|s| String::from(s.trim())).collect()
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
	
	// Extraire le titre du manga
	let manga_title = html.select("#titreOeuvre").text().read();
	if manga_title.is_empty() {
		return Ok(chapters);
	}
	
	// Créer une liste de chapitres basique pour permettre la navigation
	// En attendant une solution pour le contenu JavaScript dynamique
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