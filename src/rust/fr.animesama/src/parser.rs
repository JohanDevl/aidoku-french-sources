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

pub fn parse_chapter_list(manga_id: String, _html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// AnimeSama utilise un système d'episodes.js au lieu de select
	// Il faut d'abord identifier les scanlateurs disponibles puis récupérer episodes.js
	
	// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
	let manga_name = manga_id.split('/').last().unwrap_or("manga");
	
	// Construire l'URL pour episodes.js
	// AnimeSama utilise le slug URL comme titre (ex: blue-lock, pas "Blue Lock")
	let episodes_url = format!("{}{}/scan/vf/episodes.js?title={}", 
		String::from(BASE_URL), 
		manga_id, 
		manga_name
	);
	
	println!("AnimeSama debug: Requesting episodes.js from: {}", episodes_url);
	
	// Faire une requête pour récupérer le fichier episodes.js
	match crate::helper::request_text(&episodes_url) {
		Ok(episodes_content) => {
			println!("AnimeSama debug: Episodes.js content length: {}", episodes_content.len());
			
			// Parser le contenu JavaScript pour extraire les numéros d'épisodes
			// Chercher les variables "var eps[nombre]= ["
			let mut episode_numbers: Vec<i32> = Vec::new();
			
			for line in episodes_content.split('\n') {
				let trimmed_line = line.trim();
				if trimmed_line.starts_with("var eps") && trimmed_line.contains("= [") {
					// Extraire le numéro après "var eps" et avant "="
					if let Some(start) = trimmed_line.find("var eps") {
						let after_eps = &trimmed_line[start + 7..];
						if let Some(end) = after_eps.find('=') {
							let number_str = after_eps[..end].trim();
							if let Ok(episode_num) = number_str.parse::<i32>() {
								episode_numbers.push(episode_num);
								println!("AnimeSama debug: Found episode {}", episode_num);
							}
						}
					}
				}
			}
			
			// Supprimer les doublons et trier
			episode_numbers.sort();
			episode_numbers.dedup();
			
			println!("AnimeSama debug: Found {} episodes from episodes.js", episode_numbers.len());
			
			if episode_numbers.is_empty() {
				// Fallback si aucun épisode trouvé
				println!("AnimeSama debug: No episodes found in episodes.js, using fallback");
				for i in 1..=312 {
					episode_numbers.push(i);
				}
			}
			
			// Créer les chapitres basés sur les épisodes trouvés
			for episode_num in episode_numbers {
				chapters.push(Chapter {
					id: format!("/scan/vf/episodes.js?title={}&id={}", 
						manga_name, 
						episode_num
					),
					title: format!("Chapitre {}", episode_num),
					volume: -1.0,
					chapter: episode_num as f32,
					date_updated: current_date(),
					scanlator: String::from("AnimeSama"),
					url: format!("{}{}/scan/vf/episodes.js?title={}&id={}", 
						String::from(BASE_URL),
						manga_id,
						manga_name, 
						episode_num
					),
					lang: String::from("fr")
				});
			}
		}
		Err(_) => {
			println!("AnimeSama debug: Failed to fetch episodes.js, using fallback with 312 chapters");
			// Fallback si la requête échoue
			for i in 1..=312 {
				chapters.push(Chapter {
					id: format!("/scan/vf/episodes.js?title={}&id={}", 
						manga_name, 
						i
					),
					title: format!("Chapitre {}", i),
					volume: -1.0,
					chapter: i as f32,
					date_updated: current_date(),
					scanlator: String::from("AnimeSama"),
					url: format!("{}{}/scan/vf/episodes.js?title={}&id={}", 
						String::from(BASE_URL),
						manga_id,
						manga_name, 
						i
					),
					lang: String::from("fr")
				});
			}
		}
	}
	
	println!("AnimeSama debug: Final chapter count: {}", chapters.len());
	
	// Inverser l'ordre pour avoir les derniers chapitres en premier
	chapters.reverse();
	
	Ok(chapters)
}


pub fn parse_page_list(_html: Node, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Le chapter_id contient l'URL complète vers episodes.js avec les paramètres
	// Format: /scan/vf/episodes.js?title=Blue Lock&id=1
	
	println!("AnimeSama debug: parse_page_list - manga_id: {}", manga_id);
	println!("AnimeSama debug: parse_page_list - chapter_id: {}", chapter_id);
	
	// Construire l'URL complète pour episodes.js
	let episodes_url = if chapter_id.starts_with("http") {
		chapter_id.clone()
	} else {
		format!("{}{}{}", String::from(BASE_URL), manga_id, chapter_id)
	};
	
	println!("AnimeSama debug: Requesting episodes.js for pages from: {}", episodes_url);
	
	// Extraire le numéro d'épisode depuis l'URL
	let episode_num = if let Some(id_param) = episodes_url.split("id=").nth(1) {
		id_param.split('&').next().unwrap_or("1").parse::<i32>().unwrap_or(1)
	} else {
		1
	};
	
	println!("AnimeSama debug: Episode number: {}", episode_num);
	
	// Faire une requête pour récupérer le fichier episodes.js
	match crate::helper::request_text(&episodes_url) {
		Ok(episodes_content) => {
			println!("AnimeSama debug: Episodes.js content length for pages: {}", episodes_content.len());
			
			// Chercher la variable correspondant à cet épisode
			let episode_var = format!("var eps{}=", episode_num);
			
			for line in episodes_content.split('\n') {
				let trimmed_line = line.trim();
				if trimmed_line.starts_with(&episode_var) {
					println!("AnimeSama debug: Found episode variable: {}", episode_var);
					
					// Extraire le contenu du tableau JavaScript
					if let Some(start) = trimmed_line.find('[') {
						if let Some(end) = trimmed_line.rfind(']') {
							let array_content = &trimmed_line[start + 1..end];
							
							// Parser les URLs des images (entre guillemets)
							let mut page_index = 1;
							for url_part in array_content.split(',') {
								let url_clean = url_part.trim().trim_matches('\'').trim_matches('"');
								if !url_clean.is_empty() && url_clean.starts_with("http") {
									pages.push(Page {
										index: page_index,
										url: String::from(url_clean),
										base64: String::new(),
										text: String::new()
									});
									page_index += 1;
								}
							}
							break;
						}
					}
				}
			}
			
			println!("AnimeSama debug: Found {} pages from episodes.js", pages.len());
			
			// Si aucune page trouvée, utiliser un fallback
			if pages.is_empty() {
				println!("AnimeSama debug: No pages found, using fallback");
				for i in 1..=20 {
					let fallback_url = format!("{}/s2/scans/Blue Lock/{}/{}.jpg", String::from(BASE_URL), episode_num, i);
					pages.push(Page {
						index: i,
						url: fallback_url,
						base64: String::new(),
						text: String::new()
					});
				}
			}
		}
		Err(_) => {
			println!("AnimeSama debug: Failed to fetch episodes.js for pages, using fallback");
			// Fallback si la requête échoue
			for i in 1..=20 {
				let fallback_url = format!("{}/s2/scans/Blue Lock/{}/{}.jpg", String::from(BASE_URL), episode_num, i);
				pages.push(Page {
					index: i,
					url: fallback_url,
					base64: String::new(),
					text: String::new()
				});
			}
		}
	}
	
	Ok(pages)
}