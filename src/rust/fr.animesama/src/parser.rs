use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::{BASE_URL, helper};

// Helper pour construire l'URL correctement
fn build_chapter_url(manga_id: &str) -> String {
	// Cas spécial pour One Piece qui utilise scan_noir-et-blanc
	let is_one_piece = manga_id.contains("one-piece") || manga_id.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	if manga_id.starts_with("http") {
		// manga_id contient déjà l'URL complète
		format!("{}{}", manga_id, scan_path)
	} else {
		// manga_id est relatif, ajouter BASE_URL
		format!("{}{}{}", String::from(BASE_URL), manga_id, scan_path)
	}
}

// Helper pour construire l'URL du manga correctement
fn build_manga_url(manga_id_or_url: &str) -> String {
	if manga_id_or_url.starts_with("http") {
		// Contient déjà l'URL complète, la retourner telle quelle
		String::from(manga_id_or_url)
	} else {
		// ID relatif, ajouter BASE_URL
		format!("{}{}", String::from(BASE_URL), manga_id_or_url)
	}
}

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
			url: build_manga_url(&relative_url),
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
				url: build_manga_url(&relative_url),
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
	// Essayer d'extraire le vrai titre depuis l'élément h4
	let manga_title = if html.select("h4").text().read().trim().is_empty() {
		// Fallback: extraire depuis l'ID si pas de h4
		manga_id.split('/').last().unwrap_or("Manga")
			.replace('-', " ")
			.split_whitespace()
			.map(|word| {
				let mut chars = word.chars();
				match chars.next() {
					None => String::new(),
					Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
				}
			})
			.collect::<Vec<String>>()
			.join(" ")
	} else {
		// Utiliser le vrai titre depuis le HTML
		String::from(html.select("h4").text().read().trim())
	};
	
	// Essayer d'extraire la description (si disponible)
	let description = if html.select("p").text().read().trim().is_empty() {
		String::from("Manga disponible sur AnimeSama")
	} else {
		let desc = String::from(html.select("p").text().read().trim());
		if desc.len() > 500 {
			format!("{}...", &desc[..500])
		} else {
			desc
		}
	};
	
	// Essayer d'extraire l'image de couverture
	let cover = if html.select("img").attr("src").read().is_empty() {
		String::from("https://anime-sama.fr/images/default.jpg")
	} else {
		let cover_src = html.select("img").attr("src").read();
		if cover_src.starts_with("http") {
			cover_src
		} else {
			format!("{}{}", String::from(BASE_URL), cover_src)
		}
	};
	
	let mut categories: Vec<String> = Vec::new();
	categories.push(String::from("Manga"));
	
	Ok(Manga {
		id: manga_id.clone(),
		cover,
		title: manga_title,
		author: String::from(""),
		artist: String::from(""),
		description,
		url: build_manga_url(&manga_id),
		categories,
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}


pub fn parse_chapter_list_dynamic_with_debug(manga_id: String, html: Node, _request_url: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// 1. Essayer la méthode episodes.js (comme Tachiyomi)
	let episodes_result = parse_episodes_js(&manga_id, &html);
	
	match episodes_result {
		Ok(js_chapters) if !js_chapters.is_empty() => {
			// Succès avec episodes.js
			chapters.extend(js_chapters);
			chapters.reverse(); // Derniers en premier
			return Ok(chapters);
		}
		_ => {
			// episodes.js a échoué, continuer avec autres méthodes
		}
	}
	
	// 2. Chercher les scripts JavaScript dans la page (fallback Tachiyomi)
	let html_content = html.html().read();
	let script_result = parse_javascript_commands(&html_content);
	
	match script_result {
		Ok(script_chapters) if !script_chapters.is_empty() => {
			chapters.extend(script_chapters);
			chapters.reverse();
			return Ok(chapters);
		}
		_ => {
			// Scripts ont échoué aussi
		}
	}
	
	// 3. Méthode originale: chercher les patterns de chapitres
	let mut max_chapter = 0;
	let chapter_regex_patterns = [
		"chapitre ",
		"Chapitre ",
		"CHAPITRE ",
		"chapter ",
		"Chapter "
	];
	
	for pattern in &chapter_regex_patterns {
		let mut start_pos = 0;
		while let Some(pos) = html_content[start_pos..].find(pattern) {
			start_pos += pos + pattern.len();
			let remaining = &html_content[start_pos..];
			
			let mut number_str = String::new();
			for char in remaining.chars() {
				if char.is_ascii_digit() {
					number_str.push(char);
				} else {
					break;
				}
			}
			
			if let Ok(chapter_num) = number_str.parse::<i32>() {
				if chapter_num > max_chapter && chapter_num < 10000 {
					max_chapter = chapter_num;
				}
			}
		}
	}
	
	if max_chapter > 0 {
		for i in 1..=max_chapter {
			chapters.push(Chapter {
				id: format!("{}", i),
				title: format!("Chapitre {}", i),
				volume: -1.0,
				chapter: i as f32,
				date_updated: current_date(),
				scanlator: String::from(""),
				url: build_chapter_url(&manga_id),
				lang: String::from("fr")
			});
		}
		
		chapters.reverse();
		return Ok(chapters);
	}
	
	// 4. Fallback final
	parse_chapter_list_simple(manga_id)
}

pub fn parse_chapter_list_with_debug(manga_id: String, _dummy_html: Node, _request_url: String, _error_info: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Défaut simple : 50 chapitres
	let chapter_count = 50;
	
	// Générer les chapitres avec le count adapté
	for i in 1..=chapter_count {
		chapters.push(Chapter {
			id: format!("{}", i),
			title: format!("Chapitre {}", i),
			volume: -1.0,
			chapter: i as f32,
			date_updated: current_date(),
			scanlator: String::from(""),
			url: build_chapter_url(&manga_id),
			lang: String::from("fr")
		});
	}
	
	// Inverser pour avoir les derniers en premier
	chapters.reverse();
	
	Ok(chapters)
}

// Fonction pour parser episodes.js (méthode Tachiyomi)
fn parse_episodes_js(manga_id: &str, html: &Node) -> Result<Vec<Chapter>> {
	// Extraire le titre du manga depuis l'élément titreOeuvre
	let manga_title = html.select("#titreOeuvre").text().read();
	if manga_title.is_empty() {
		return Ok(Vec::new()); // Pas de titre trouvé
	}
	
	// Cas spécial pour One Piece qui utilise scan_noir-et-blanc
	let is_one_piece = manga_id.contains("one-piece") || manga_id.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	// Construire l'URL vers episodes.js
	let episodes_url = if manga_id.starts_with("http") {
		format!("{}{}episodes.js?title={}", manga_id, scan_path, helper::urlencode(&manga_title))
	} else {
		format!("{}{}{}episodes.js?title={}", String::from(BASE_URL), manga_id, scan_path, helper::urlencode(&manga_title))
	};
	
	// Faire la requête vers episodes.js
	match aidoku::std::net::Request::new(&episodes_url, aidoku::std::net::HttpMethod::Get).string() {
		Ok(js_content) => {
			parse_episodes_content(&js_content, manga_id)
		}
		Err(_) => Ok(Vec::new()) // Échec de la requête
	}
}

// Parser le contenu JavaScript d'episodes.js
fn parse_episodes_content(js_content: &str, manga_id: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Extraire tous les numéros d'épisodes avec regex eps(\d+)
	let mut max_episode = 0;
	let mut min_episode = i32::MAX;
	let mut episodes_found = Vec::new();
	let mut start_pos = 0;
	
	while let Some(pos) = js_content[start_pos..].find("eps") {
		start_pos += pos + 3; // Skip "eps"
		let remaining = &js_content[start_pos..];
		
		// Extraire le numéro d'épisode
		let mut number_str = String::new();
		for char in remaining.chars() {
			if char.is_ascii_digit() {
				number_str.push(char);
			} else {
				break;
			}
		}
		
		if let Ok(episode_num) = number_str.parse::<i32>() {
			episodes_found.push(episode_num);
			if episode_num > max_episode {
				max_episode = episode_num;
			}
			if episode_num < min_episode {
				min_episode = episode_num;
			}
		}
	}
	
	// Nettoyer min_episode si aucun épisode trouvé
	if min_episode == i32::MAX {
		min_episode = 1;
	}
	
	// Créer une série continue de chapitres de min_episode à max_episode
	if max_episode > 0 {
		// Calculer le nombre total de chapitres
		let total_chapters = max_episode - min_episode + 1;
		
		for episode_num in min_episode..=max_episode {
			chapters.push(Chapter {
				id: format!("{}", episode_num),
				title: format!("Chapitre {}", episode_num),
				volume: -1.0,
				chapter: episode_num as f32,
				date_updated: current_date(),
				scanlator: String::from(""),
				url: build_chapter_url(manga_id),
				lang: String::from("fr")
			});
		}
		
		// Debug temporaire : ajouter un chapitre debug pour vérifier le count
		episodes_found.sort();
		episodes_found.dedup(); // Enlever les doublons
		chapters.push(Chapter {
			id: String::from("debug_count"),
			title: format!("DEBUG: Range {}..{} ({} total), {} eps found", min_episode, max_episode, total_chapters, episodes_found.len()),
			volume: -1.0,
			chapter: 999.0,
			date_updated: current_date(),
			scanlator: format!("First few eps: {:?}", episodes_found.iter().take(5).collect::<Vec<_>>()),
			url: build_chapter_url(manga_id),
			lang: String::from("fr")
		});
	}
	
	Ok(chapters)
}

// Parser les commandes JavaScript dans la page HTML (fallback)
fn parse_javascript_commands(html_content: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Chercher resetListe() pour confirmer qu'il y a des scripts
	if !html_content.contains("resetListe()") {
		return Ok(Vec::new());
	}
	
	// Parser les commandes creerListe(start, end)
	let mut start_pos = 0;
	while let Some(pos) = html_content[start_pos..].find("creerListe(") {
		start_pos += pos + 11; // Skip "creerListe("
		let remaining = &html_content[start_pos..];
		
		// Extraire les paramètres jusqu'à la parenthèse fermante
		if let Some(end_pos) = remaining.find(')') {
			let params = &remaining[..end_pos];
			let parts: Vec<&str> = params.split(',').collect();
			
			if parts.len() == 2 {
				if let (Ok(start_num), Ok(end_num)) = (
					parts[0].trim().parse::<i32>(),
					parts[1].trim().parse::<i32>()
				) {
					// Ajouter les chapitres de start à end
					for i in start_num..=end_num {
						chapters.push(Chapter {
							id: format!("{}", i),
							title: format!("Chapitre {}", i),
							volume: -1.0,
							chapter: i as f32,
							date_updated: current_date(),
							scanlator: String::from(""),
							url: String::new(), // sera rempli plus tard
							lang: String::from("fr")
						});
					}
				}
			}
		}
		
		start_pos += 1; // Avancer pour la prochaine recherche
	}
	
	// Parser les commandes newSP() pour les chapitres spéciaux
	let mut start_pos = 0;
	while let Some(pos) = html_content[start_pos..].find("newSP(") {
		start_pos += pos + 6; // Skip "newSP("
		let remaining = &html_content[start_pos..];
		
		if let Some(end_pos) = remaining.find(')') {
			let param = &remaining[..end_pos].trim();
			
			// Essayer de parser comme nombre ou garder comme string
			if let Ok(special_num) = param.parse::<i32>() {
				chapters.push(Chapter {
					id: format!("{}", special_num),
					title: format!("Chapitre {}", special_num),
					volume: -1.0,
					chapter: special_num as f32,
					date_updated: current_date(),
					scanlator: String::from(""),
					url: String::new(), // sera rempli plus tard
					lang: String::from("fr")
				});
			} else if param.starts_with('"') && param.ends_with('"') {
				let special_title = &param[1..param.len()-1]; // Enlever les quotes
				chapters.push(Chapter {
					id: String::from(special_title),
					title: format!("Chapitre {}", special_title),
					volume: -1.0,
					chapter: chapters.len() as f32 + 1.0,
					date_updated: current_date(),
					scanlator: String::from(""),
					url: String::new(), // sera rempli plus tard
					lang: String::from("fr")
				});
			}
		}
		
		start_pos += 1;
	}
	
	Ok(chapters)
}

pub fn parse_chapter_list_simple(manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Défaut simple : 50 chapitres
	for i in 1..=50 {
		chapters.push(Chapter {
			id: format!("{}", i),
			title: format!("Chapitre {}", i),
			volume: -1.0,
			chapter: i as f32,
			date_updated: current_date(),
			scanlator: String::from(""),
			url: build_chapter_url(&manga_id),
			lang: String::from("fr")
		});
	}
	
	// Inverser pour avoir les derniers en premier
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