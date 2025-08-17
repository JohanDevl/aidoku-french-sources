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
	let title = html.select("#titreOeuvre").text().read();
	let cover = html.select("#coverOeuvre").attr("src").read();
	
	// Approche robuste pour extraire description et genres
	let mut description = String::new();
	let mut genre_text = String::new();
	
	// Parcourir tous les h2 dans sousBlocMiddle pour trouver Synopsis et Genres
	for h2 in html.select("#sousBlocMiddle h2").array() {
		let h2_node = match h2.as_node() {
			Ok(node) => node,
			Err(_) => continue,
		};
		
		let h2_text = h2_node.text().read();
		
		if h2_text.contains("Synopsis") {
			// Chercher le paragraphe suivant
			let mut current = h2_node;
			if let Some(next_sibling) = current.next() {
				if next_sibling.select("p").array().len() > 0 || next_sibling.tag_name().read() == "p" {
					description = next_sibling.text().read();
					break;
				}
				current = next_sibling;
			} else {
				break;
			}
		} else if h2_text.contains("Genres") {
			// Chercher l'élément suivant contenant les genres
			let mut current = h2_node;
			if let Some(next_sibling) = current.next() {
				let sibling_text = next_sibling.text().read();
				if !sibling_text.is_empty() && sibling_text.contains(",") {
					genre_text = sibling_text;
					break;
				}
				current = next_sibling;
			} else {
				break;
			}
		}
	}
	
	// Si la méthode robuste échoue, essayer les sélecteurs simples
	if description.is_empty() {
		description = html.select("#sousBlocMiddle p").first().text().read();
	}
	if genre_text.is_empty() {
		genre_text = html.select("#sousBlocMiddle a").first().text().read();
	}
	
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
			url: format!("/catalogue/{}/scan/vf/chapitre-{}", manga_id, i),
			lang: String::from("fr")
		});
	}
	
	// Inverser l'ordre pour avoir les derniers chapitres en premier
	chapters.reverse();
	
	Ok(chapters)
}


pub fn parse_page_list(_html: Node, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Extraire le numéro de chapitre depuis l'ID
	let chapter_num = chapter_id.replace("chapitre-", "");
	
	// Générer une liste de pages basique
	// Utiliser le pattern observé d'AnimeSama: /s2/scans/{manga}/{chapter}/{page}.jpg
	for i in 1..=20 {
		let image_url = format!("{}{}/{}/{}.jpg", String::from(CDN_URL), manga_id, chapter_num, i);
		pages.push(Page {
			index: i,
			url: image_url,
			base64: String::new(),
			text: String::new()
		});
	}
	
	Ok(pages)
}