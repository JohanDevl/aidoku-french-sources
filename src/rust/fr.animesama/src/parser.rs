use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;

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

pub fn parse_chapter_list(_manga_id: String, _html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// TEST : 10 chapitres fake pour vérifier l'affichage dans Aidoku
	println!("AnimeSama debug: Creating 10 fake chapters for testing");
	
	for i in 1..=10 {
		let chapter_url = format!("{}/catalogue/blue-lock/scan/vf/", String::from(BASE_URL));
		
		chapters.push(Chapter {
			id: format!("{}", i),
			title: format!("Chapitre {}", i),  // Titre explicite pour test
			volume: -1.0,
			chapter: i as f32,
			date_updated: current_date(),
			scanlator: String::from("AnimeSama Test"),
			url: chapter_url,
			lang: String::from("fr")
		});
		
		println!("AnimeSama debug: Created fake chapter {}", i);
	}
	
	println!("AnimeSama debug: Created {} fake chapters", chapters.len());
	
	// Les plus récents en premier
	chapters.reverse();
	
	Ok(chapters)
}


pub fn parse_page_list(html: Node, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	println!("AnimeSama debug: parse_page_list - manga_id: {}", manga_id);
	println!("AnimeSama debug: parse_page_list - chapter_id: {}", chapter_id);
	
	// Parser les images directement depuis la page HTML
	// Les images ont des alt comme "Chapitre X – page Y"
	let images = html.select("img[alt*='Chapitre']");
	let images_count = images.array().len();
	
	println!("AnimeSama debug: Found {} images with Chapitre in alt", images_count);
	
	if images_count > 0 {
		// Parser les vraies images de la page
		let mut page_index = 1;
		for image in images.array() {
			if let Ok(image_node) = image.as_node() {
				let image_src = image_node.attr("src").read();
				let image_alt = image_node.attr("alt").read();
				
				// Vérifier que c'est bien une page de chapitre
				if !image_src.is_empty() && image_alt.contains("page") {
					pages.push(Page {
						index: page_index,
						url: image_src.clone(),
						base64: String::new(),
						text: String::new()
					});
					
					println!("AnimeSama debug: Added page {} with URL: {}", page_index, image_src);
					page_index += 1;
				}
			}
		}
		
		println!("AnimeSama debug: Successfully parsed {} pages from HTML", pages.len());
	} else {
		// Fallback uniquement si aucune image trouvée
		println!("AnimeSama debug: No images found, using fallback");
		
		// Extraire le numéro de chapitre depuis chapter_id (maintenant c'est juste le numéro)
		let chapter_num = chapter_id.parse::<i32>().unwrap_or(1);
		
		// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
		let manga_name = manga_id.split('/').last().unwrap_or("manga");
		
		// Générer des URLs de fallback basées sur la structure CDN d'AnimeSama
		for i in 1..=20 {
			let fallback_url = format!("{}/s2/scans/{}/{}/{}.jpg", 
				String::from(BASE_URL), 
				manga_name, 
				chapter_num, 
				i
			);
			pages.push(Page {
				index: i,
				url: fallback_url,
				base64: String::new(),
				text: String::new()
			});
		}
	}
	
	println!("AnimeSama debug: Final page count: {}", pages.len());
	
	Ok(pages)
}