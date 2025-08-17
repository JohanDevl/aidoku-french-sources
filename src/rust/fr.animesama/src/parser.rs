use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, net::{Request, HttpMethod}, String, StringRef, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::{BASE_URL, CDN_URL};
use crate::helper;

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
	let title = html.select("#titreOeuvre").text().read();
	let cover = html.select("#coverOeuvre").attr("src").read();
	let description = html.select("#sousBlocMiddle > div h2:contains(Synopsis) + p").text().read();
	let genre_text = html.select("#sousBlocMiddle > div h2:contains(Genres) + a").text().read();
	
	// Convertir les genres en Vec<String>
	let categories: Vec<String> = if genre_text.is_empty() {
		Vec::new()
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

pub fn parse_chapter_list(_manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let chapters: Vec<Chapter> = Vec::new();
	
	// Cette fonction sera plus complexe et nécessitera le parsing du JavaScript
	// Pour l'instant, implémentation basique pour compiler
	
	// Extraire le titre du manga pour les requêtes episodes.js
	let manga_title = html.select("#titreOeuvre").text().read();
	
	if manga_title.is_empty() {
		return Ok(chapters);
	}
	
	// Chercher les scripts qui contiennent panneauScan
	let script_content = html.select("script:contains(panneauScan)").text().read();
	
	if script_content.is_empty() {
		return Ok(chapters);
	}
	
	// Pour l'instant, retourner une liste vide
	// L'implémentation complète sera faite dans une phase ultérieure
	
	Ok(chapters)
}

pub fn parse_page_list(_html: Node, _manga_id: String, _chapter_id: String) -> Result<Vec<Page>> {
	let pages: Vec<Page> = Vec::new();
	
	// Cette fonction nécessitera l'extraction des informations depuis episodes.js
	// Pour l'instant, implémentation basique pour compiler
	
	Ok(pages)
}