use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, net::{Request, HttpMethod}, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};
use core::cmp::Ordering;

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
	
	// Essayer d'extraire l'image de couverture depuis l'élément avec ID coverOeuvre
	let cover = if html.select("#coverOeuvre").attr("src").read().is_empty() {
		String::from("https://anime-sama.fr/images/default.jpg")
	} else {
		let cover_src = html.select("#coverOeuvre").attr("src").read();
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


// Nouvelle fonction pour récupérer les chapitres depuis l'API AnimeSama
fn get_chapters_from_api(manga_title: &str) -> Result<Vec<i32>> {
	// Construire l'URL de l'API
	let encoded_title = helper::urlencode(manga_title);
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", encoded_title);
	
	// Faire la requête
	let json = Request::new(&api_url, HttpMethod::Get)
		.header("User-Agent", "Mozilla/5.0")
		.header("Accept", "application/json")
		.json()?;
	let json_obj = json.as_object()?;
	
	// Parser le JSON pour extraire les clés (numéros de chapitres)
	let mut chapters: Vec<i32> = Vec::new();
	
	// Parcourir toutes les clés de l'objet JSON
	for key in json_obj.keys() {
		if let Ok(key_str) = key.as_string() {
			if let Ok(chapter_num) = key_str.read().parse::<i32>() {
				if chapter_num > 0 {
					chapters.push(chapter_num);
				}
			}
		}
	}
	
	// Trier les chapitres
	chapters.sort_unstable();
	
	if chapters.is_empty() {
		return Err(aidoku::error::AidokuError { 
			reason: aidoku::error::AidokuErrorKind::Unimplemented 
		});
	}
	
	Ok(chapters)
}

pub fn parse_chapter_list_dynamic_with_debug(manga_id: String, html: Node, _request_url: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Extraire le titre du manga depuis le HTML
	let manga_title = html.select("#titreOeuvre").text().read();
	let manga_name = if manga_title.is_empty() {
		// Transformer le manga_id en titre (blue-lock -> Blue Lock)
		manga_id.replace("-", " ")
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
		manga_title
	};
	
	// PRIORITÉ 1 : Essayer l'API get_nb_chap_et_img.php
	match get_chapters_from_api(&manga_name) {
		Ok(chapter_numbers) => {
			// Succès avec l'API
			for chapter_num in &chapter_numbers {
				chapters.push(Chapter {
					id: format!("{}", chapter_num),
					title: format!("Chapitre {}", chapter_num),
					volume: -1.0,
					chapter: *chapter_num as f32,
					date_updated: current_date(),
					scanlator: String::from(""),
					url: build_chapter_url(&manga_id),
					lang: String::from("fr")
				});
			}
			
			// Tri par numéro de chapitre (du plus récent au plus ancien)
			chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
			return Ok(chapters);
		}
		Err(_) => {
			// L'API a échoué, essayer episodes.js
		}
	}
	
	// PRIORITÉ 2 : Fallback sur episodes.js si l'API échoue
	match parse_episodes_js(&manga_id, &html) {
		Ok(js_chapters) if !js_chapters.is_empty() => {
			return Ok(js_chapters);
		}
		_ => {
			// episodes.js a aussi échoué
		}
	}
	
	// Aucune méthode n'a fonctionné - retourner liste vide
	Ok(chapters)
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

// Fonction pour parser directement le select HTML (méthode prioritaire)
fn parse_chapter_list_from_select(html: &Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// NOUVEAU: Parser directement le HTML brut au lieu d'utiliser les sélecteurs CSS
	let html_content = html.html().read();
	let regex_chapters = parse_options_from_raw_html(&html_content);
	
	if !regex_chapters.is_empty() {
		// Vérifier si ce sont des vraies valeurs ou des codes debug négatifs
		if regex_chapters.iter().all(|&x| x > 0) {
			// Calculer min/max avant de déplacer le vector
			let min_ch = *regex_chapters.iter().min().unwrap_or(&0);
			let max_ch = *regex_chapters.iter().max().unwrap_or(&0);
			let chapter_count = regex_chapters.len();
			
			// Succès avec parsing HTML brut
			for chapter_num in &regex_chapters {
				chapters.push(Chapter {
					id: format!("{}", chapter_num),
					title: format!("Chapitre {}", chapter_num),
					volume: -1.0,
					chapter: *chapter_num as f32,
					date_updated: current_date(),
					scanlator: String::from(""),
					url: String::from(""), // Sera rempli plus tard par build_chapter_url
					lang: String::from("fr")
				});
			}
			
			// Debug pour confirmer le succès
			chapters.push(Chapter {
				id: String::from("debug_regex_success"),
				title: format!("DEBUG: PARSING HTML BRUT RÉUSSI - {} chapitres (min: {}, max: {})", chapter_count, min_ch, max_ch),
				volume: -1.0,
				chapter: -1.0,
				date_updated: current_date(),
				scanlator: String::from("AnimeSama Debug"),
				url: String::from(""),
				lang: String::from("fr")
			});
			
			return Ok(chapters);
		} else {
			// Codes debug négatifs - interpréter les signaux
			let debug_code = regex_chapters[0];
			let debug_msg = match debug_code {
				-3000 => String::from("Pas de <select> du tout dans le HTML"),
				-2000 => String::from("Select existe mais aucune <option>"),
				code if code <= -1000 => format!("Select + {} <option> mais pas 'Chapitre X'", -1000 - code),
				_ => format!("Code debug inconnu: {}", debug_code)
			};
			
			chapters.push(Chapter {
				id: String::from("debug_html_analysis"),
				title: format!("DEBUG: HTML ANALYSIS - {}", debug_msg),
				volume: -1.0,
				chapter: -1.0,
				date_updated: current_date(),
				scanlator: String::from("AnimeSama Debug"),
				url: String::from(""),
				lang: String::from("fr")
			});
			
			// Continuer vers les fallbacks
		}
	}
	
	// Si le parsing HTML brut échoue, essayer les sélecteurs CSS comme fallback
	let mut select_options = html.select("select option");
	let mut options_count = select_options.array().len();
	
	if options_count == 0 {
		select_options = html.select("option");
		options_count = select_options.array().len();
	}
	
	if options_count == 0 {
		select_options = html.select("#selectChapitres option");
		options_count = select_options.array().len();
	}
	
	// Ajouter debug pour voir ce qu'on trouve exactement
	chapters.push(Chapter {
		id: String::from("debug_select_info"),
		title: format!("DEBUG: HTML brut échoué, CSS: {} options trouvées", options_count),
		volume: -1.0,
		chapter: -1.0,
		date_updated: current_date(),
		scanlator: String::from("AnimeSama Debug"),
		url: String::from(""),
		lang: String::from("fr")
	});
	
	if options_count == 0 {
		return Ok(chapters);
	}
	
	let mut max_chapter = 0;
	let mut min_chapter = i32::MAX;
	let mut debug_options: Vec<String> = Vec::new();
	
	for option in select_options.array() {
		if let Ok(option_node) = option.as_node() {
			let option_text = String::from(option_node.text().read().trim());
			
			// Garder trace des premières options pour debug
			if debug_options.len() < 5 {
				debug_options.push(option_text.clone());
			}
			
			// Parser "Chapitre X"
			if option_text.starts_with("Chapitre ") {
				let chapter_num_str = option_text.replace("Chapitre ", "");
				if let Ok(chapter_num) = chapter_num_str.parse::<i32>() {
					if chapter_num > max_chapter {
						max_chapter = chapter_num;
					}
					if chapter_num < min_chapter {
						min_chapter = chapter_num;
					}
					
					chapters.push(Chapter {
						id: format!("{}", chapter_num),
						title: format!("Chapitre {}", chapter_num),
						volume: -1.0,
						chapter: chapter_num as f32,
						date_updated: current_date(),
						scanlator: String::from(""),
						url: String::from(""), // Sera rempli plus tard par build_chapter_url
						lang: String::from("fr")
					});
				}
			}
		}
	}
	
	// Ajouter debug avec le contenu réel des options
	let debug_text = if debug_options.is_empty() {
		String::from("Aucune option trouvée")
	} else {
		format!("Options: {:?}", debug_options)
	};
	
	chapters.push(Chapter {
		id: String::from("debug_options_content"),
		title: format!("DEBUG: {}", debug_text),
		volume: -1.0,
		chapter: -2.0,
		date_updated: current_date(),
		scanlator: String::from("AnimeSama Debug"),
		url: String::from(""),
		lang: String::from("fr")
	});
	
	Ok(chapters)
}

// Nouvelle fonction pour parser les options directement depuis le HTML brut
fn parse_options_from_raw_html(html_content: &str) -> Vec<i32> {
	let mut chapters: Vec<i32> = Vec::new();
	
	// DEBUG: Ajouter un chapitre avec un extrait du HTML pour voir ce qu'on reçoit vraiment
	let html_excerpt = if html_content.len() > 500 {
		&html_content[..500]
	} else {
		html_content
	};
	
	// Chercher s'il y a un select dans le HTML
	let has_select = html_content.contains("<select");
	let has_option = html_content.contains("<option");
	let option_count = html_content.matches("<option").count();
	
	// Chercher tous les patterns <option>Chapitre X</option>
	let mut start_pos = 0;
	while let Some(pos) = html_content[start_pos..].find("<option>Chapitre ") {
		start_pos += pos + 17; // Skip "<option>Chapitre "
		let remaining = &html_content[start_pos..];
		
		// Extraire le numéro jusqu'à </option>
		if let Some(end_pos) = remaining.find("</option>") {
			let chapter_text = &remaining[..end_pos];
			
			// Parser le numéro
			let mut number_str = String::new();
			for char in chapter_text.chars() {
				if char.is_ascii_digit() {
					number_str.push(char);
				} else {
					break;
				}
			}
			
			if let Ok(chapter_num) = number_str.parse::<i32>() {
				chapters.push(chapter_num);
			}
		}
		
		start_pos += 1;
	}
	
	// Si aucun chapitre trouvé, créer un chapitre debug avec info HTML
	if chapters.is_empty() {
		// Créer un chapitre debug pour montrer ce qu'on trouve dans le HTML
		// On ne peut pas retourner Chapter ici, mais on peut utiliser les valeurs négatives pour debug
		// Retourner une valeur spéciale qui sera interceptée par la fonction parent
		if has_select && has_option {
			chapters.push(-1000 - option_count as i32); // Signal: select existe mais pas de "Chapitre X"
		} else if has_select {
			chapters.push(-2000); // Signal: select existe mais pas d'options
		} else {
			chapters.push(-3000); // Signal: pas de select du tout
		}
	}
	
	// Trier et dédupliquer (sauf les valeurs debug négatives)
	if chapters.iter().all(|&x| x > 0) {
		chapters.sort();
		chapters.dedup();
	}
	
	chapters
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
		
	}
	
	Ok(chapters)
}

// Helper pour parser les numéros de chapitre depuis les messages de la page
fn parse_chapter_from_message(html_content: &str) -> i32 {
	// Chercher des patterns dans les messages comme "chapitre 314"
	let chapter_patterns = [
		"chapitre ",
		"Chapitre ",
		"CHAPITRE ",
		"chapter ",
		"Chapter "
	];
	
	let mut max_chapter = 0;
	
	for pattern in &chapter_patterns {
		let mut start_pos = 0;
		while let Some(pos) = html_content[start_pos..].find(pattern) {
			start_pos += pos + pattern.len();
			let remaining = &html_content[start_pos..];
			
			// Extraire le numéro qui suit
			let mut number_str = String::new();
			for char in remaining.chars() {
				if char.is_ascii_digit() {
					number_str.push(char);
				} else {
					break;
				}
			}
			
			if let Ok(chapter_num) = number_str.parse::<i32>() {
				// Filtrer les numéros raisonnables (éviter les années, etc.)
				if chapter_num > max_chapter && chapter_num < 5000 {
					max_chapter = chapter_num;
				}
			}
		}
	}
	
	max_chapter
}

// Parser les commandes JavaScript dans la page HTML (méthode principale)
fn parse_javascript_commands(html_content: &str, manga_id: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Chercher resetListe() pour confirmer qu'il y a des scripts actifs
	if !html_content.contains("resetListe()") {
		return Ok(Vec::new());
	}
	
	let mut chapter_counter = 1;
	let mut finir_liste_start = 0;
	
	// 1. Parser les commandes creerListe(start, end)
	let mut start_pos = 0;
	while let Some(pos) = html_content[start_pos..].find("creerListe(") {
		start_pos += pos + 11; // Skip "creerListe("
		let remaining = &html_content[start_pos..];
		
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
							url: build_chapter_url(manga_id),
							lang: String::from("fr")
						});
					}
					chapter_counter = end_num + 1; // Préparer pour les suivants
				}
			}
		}
		
		start_pos += 1;
	}
	
	// 2. Parser les commandes newSP() pour les chapitres spéciaux
	let mut start_pos = 0;
	while let Some(pos) = html_content[start_pos..].find("newSP(") {
		start_pos += pos + 6; // Skip "newSP("
		let remaining = &html_content[start_pos..];
		
		if let Some(end_pos) = remaining.find(')') {
			let param = &remaining[..end_pos].trim();
			
			if param.starts_with('"') && param.ends_with('"') {
				let special_title = &param[1..param.len()-1]; // Enlever les quotes
				chapters.push(Chapter {
					id: format!("special_{}", chapter_counter),
					title: format!("Chapitre {}", special_title),
					volume: -1.0,
					chapter: chapter_counter as f32,
					date_updated: current_date(),
					scanlator: String::from(""),
					url: build_chapter_url(manga_id),
					lang: String::from("fr")
				});
				chapter_counter += 1;
			}
		}
		
		start_pos += 1;
	}
	
	// 3. Parser finirListe(start) et continuer jusqu'à la fin
	let mut start_pos = 0;
	if let Some(pos) = html_content[start_pos..].find("finirListe(") {
		start_pos += pos + 11; // Skip "finirListe("
		let remaining = &html_content[start_pos..];
		
		if let Some(end_pos) = remaining.find(')') {
			let param = remaining[..end_pos].trim();
			if let Ok(finir_start) = param.parse::<i32>() {
				finir_liste_start = finir_start;
			}
		}
	}
	
	// 4. Si finirListe() trouvé, essayer de déterminer la fin via episodes.js
	if finir_liste_start > 0 {
		// Essayer d'obtenir le maximum depuis episodes.js
		if let Ok(episodes_max) = get_max_episode_from_js(manga_id) {
			if episodes_max > finir_liste_start {
				// Continuer de finir_liste_start jusqu'à episodes_max
				for i in finir_liste_start..=episodes_max {
					chapters.push(Chapter {
						id: format!("{}", i),
						title: format!("Chapitre {}", i),
						volume: -1.0,
						chapter: i as f32,
						date_updated: current_date(),
						scanlator: String::from(""),
						url: build_chapter_url(manga_id),
						lang: String::from("fr")
					});
				}
			}
		}
	}
	
	Ok(chapters)
}

// Helper pour obtenir le maximum d'épisode depuis episodes.js
fn get_max_episode_from_js(manga_id: &str) -> Result<i32> {
	// Essayer de récupérer episodes.js rapidement juste pour le maximum
	let is_one_piece = manga_id.contains("one-piece") || manga_id.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	let episodes_url = if manga_id.starts_with("http") {
		format!("{}{}episodes.js", manga_id, scan_path)
	} else {
		format!("{}{}{}episodes.js", String::from(BASE_URL), manga_id, scan_path)
	};
	
	match aidoku::std::net::Request::new(&episodes_url, aidoku::std::net::HttpMethod::Get).string() {
		Ok(js_content) => {
			// Parser uniquement pour trouver le maximum
			let mut max_episode = 0;
			let mut start_pos = 0;
			
			while let Some(pos) = js_content[start_pos..].find("eps") {
				start_pos += pos + 3; // Skip "eps"
				let remaining = &js_content[start_pos..];
				
				let mut number_str = String::new();
				for char in remaining.chars() {
					if char.is_ascii_digit() {
						number_str.push(char);
					} else {
						break;
					}
				}
				
				if let Ok(episode_num) = number_str.parse::<i32>() {
					if episode_num > max_episode {
						max_episode = episode_num;
					}
				}
			}
			
			Ok(max_episode)
		}
		Err(_) => Err(aidoku::error::AidokuError { 
			reason: aidoku::error::AidokuErrorKind::Unimplemented 
		})
	}
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