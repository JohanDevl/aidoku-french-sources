use aidoku_stable::prelude::*;
use alloc::{string::String, vec::Vec, format};
use core::cmp::Ordering;

use crate::{BASE_URL, CDN_URL, helper};

// Structure pour stocker le mapping indice -> numéro de chapitre
struct ChapterMapping {
	index: i32,
	chapter_number: f32,
	title: String,
}

// Parser les commandes JavaScript pour créer le mapping
fn parse_chapter_mapping(html_content: &str) -> Vec<ChapterMapping> {
	let mut mappings: Vec<ChapterMapping> = Vec::new();
	let mut current_index = 1; // Indices commencent à 1
	
	// Parser ligne par ligne en respectant l'ordre des commandes
	let lines: Vec<&str> = html_content.lines().collect();
	
	for line in lines {
		let trimmed = line.trim();
		
		// Parser tous les patterns sur la même ligne dans l'ordre
		let mut line_pos = 0;
		
		while line_pos < trimmed.len() {
			// Chercher le prochain pattern sur cette ligne
			let creer_pos = trimmed[line_pos..].find("creerListe(");
			let newsp_pos = trimmed[line_pos..].find("newSP(");
			
			// Déterminer quel pattern vient en premier
			let next_pattern = match (creer_pos, newsp_pos) {
				(Some(c), Some(n)) if c < n => ("creer", line_pos + c),
				(Some(c), None) => ("creer", line_pos + c),
				(None, Some(n)) => ("newsp", line_pos + n),
				_ => break, // Aucun pattern trouvé
			};
			
			match next_pattern.0 {
				"creer" => {
					let start_pos = next_pattern.1 + "creerListe(".len();
					if let Some(params_end) = trimmed[start_pos..].find(");") {
						let params = &trimmed[start_pos..start_pos + params_end];
						let parts: Vec<&str> = params.split(",").map(|s| s.trim()).collect();
						
						if parts.len() == 2 {
							if let (Ok(start), Ok(end)) = (parts[0].parse::<i32>(), parts[1].parse::<i32>()) {
								// Créer les mappings pour la plage
								for chapter_num in start..=end {
									mappings.push(ChapterMapping {
										index: current_index,
										chapter_number: chapter_num as f32,
										title: format!("Chapitre {}", chapter_num),
									});
									current_index += 1;
								}
							}
						}
						line_pos = start_pos + params_end + 2; // +2 pour ");
					} else {
						break;
					}
				}
				"newsp" => {
					let start_pos = next_pattern.1 + "newSP(".len();
					if let Some(params_end) = trimmed[start_pos..].find(");") {
						let param = trimmed[start_pos..start_pos + params_end].trim();
						
						// Cas 1: Chapitre avec label texte comme "One Shot"
						if param.starts_with("\"") && param.ends_with("\"") && param.len() > 2 {
							let text_content = &param[1..param.len()-1];
							mappings.push(ChapterMapping {
								index: current_index,
								chapter_number: current_index as f32, // Utiliser l'index comme numéro
								title: format!("Chapitre {}", text_content),
							});
							current_index += 1;
						}
						// Cas 2: Gérer les nombres décimaux comme 19.5
						else if let Ok(special_num) = param.parse::<f32>() {
							mappings.push(ChapterMapping {
								index: current_index,
								chapter_number: special_num,
								title: format!("Chapitre {}", special_num),
							});
							current_index += 1;
						}
						line_pos = start_pos + params_end + 2; // +2 pour ");
					} else {
						break;
					}
				}
				_ => break,
			}
		}
	}
	
	mappings
}

// Fonction pour récupérer le nombre total de chapitres depuis l'API
fn get_total_chapters_from_api(manga_title: &str) -> Result<i32> {
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", 
		helper::urlencode(manga_title));
	
	// Placeholder: Le wrapper stable ne supporte pas encore les appels HTTP réels
	// Retourner une valeur par défaut basée sur les chapitres mappés
	let default_max_chapter = 100; // Valeur par défaut raisonnable
	
	// TODO: Implémenter l'appel API réel quand le wrapper supportera HTTP
	if default_max_chapter > 0 {
		Ok(default_max_chapter)
	} else {
		Err(AidokuError::new("No chapters found"))
	}
}

// Fonction pour calculer le numéro de chapitre réel en tenant compte des chapitres spéciaux
fn calculate_chapter_number_for_index(index: i32, mappings: &[ChapterMapping]) -> f32 {
	// Pour les indices non mappés après finirListe(), continuer la numérotation séquentielle
	// Ex: Dandadan finirListe(27) → indice 28 = chapitre 27, indice 29 = chapitre 28, etc.
	
	if mappings.is_empty() {
		return index as f32;
	}
	
	// Trouver le dernier indice mappé
	let last_mapped_index = mappings.iter().map(|m| m.index).max().unwrap_or(0);
	
	// Trouver le dernier chapitre numérique (ignorer "One Shot", etc.)
	let last_numeric_chapter = mappings.iter()
		.filter(|m| m.title.chars().any(|c| c.is_ascii_digit()) && !m.title.contains("One Shot"))
		.filter_map(|m| {
			let title_parts: Vec<&str> = m.title.split_whitespace().collect();
			if title_parts.len() >= 2 {
				title_parts[1].parse::<f32>().ok()
			} else {
				None
			}
		})
		.max_by(|a, b| a.partial_cmp(b).unwrap_or(Ordering::Equal))
		.unwrap_or(0.0);
	
	// Pour finirListe(27), les indices après le dernier mapping commencent au chapitre suivant entier
	// Si le dernier chapitre est 19.5, le suivant devrait être 20, pas 20.5
	let chapters_after_last_numeric = index - last_mapped_index;
	
	// Calculer le prochain numéro de chapitre entier après le dernier chapitre numérique
	let fractional_part = last_numeric_chapter - (last_numeric_chapter as i32 as f32);
	let next_chapter_base = if fractional_part > 0.0 {
		// Si c'est un décimal (ex: 19.5), le prochain entier est 20
		(last_numeric_chapter as i32 + 1) as f32
	} else {
		// Si c'est déjà un entier (ex: 19), le prochain est 20
		last_numeric_chapter + 1.0
	};
	
	// Les chapitres après reprennent une numérotation entière normale
	next_chapter_base + (chapters_after_last_numeric - 1) as f32
}

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
			cover: Some(cover_url),
			title,
			author: None,
			artist: None,
			description: None,
			url: Some(build_manga_url(&relative_url)),
			categories: Some(Vec::new()),
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
				cover: Some(cover_url),
				title,
				author: None,
				artist: None,
				description: None,
				url: Some(build_manga_url(&relative_url)),
				categories: Some(Vec::new()),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Scroll
			});
		}
		
		// Les dernières sorties de la page d'accueil n'ont généralement pas de pagination
	} else {
		// Pour le populaire, utiliser le même parsing que la liste générale
		return parse_manga_list(html);
	}
	
	Ok(MangaPageResult {
		manga: mangas,
		has_more: false
	})
}

pub fn parse_manga_details(manga_id: String, html: Node) -> Result<Manga> {
	// Extraire le titre depuis l'élément #titreOeuvre
	let manga_title = if !html.select("#titreOeuvre").text().read().trim().is_empty() {
		String::from(html.select("#titreOeuvre").text().read().trim())
	} else {
		// Fallback: extraire depuis l'ID si pas de #titreOeuvre
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
	};
	
	// Extraire la description depuis le synopsis (h2:contains(Synopsis)+p)
	let synopsis_elements = html.select("#sousBlocMiddle h2:contains(Synopsis) + p").array();
	let description = if !synopsis_elements.is_empty() {
		// Simplifier en utilisant la sélection directement
		let synopsis_text = html.select("#sousBlocMiddle h2:contains(Synopsis) + p").text().read();
		if synopsis_text.trim().is_empty() {
			String::from("Manga disponible sur AnimeSama")
		} else {
			String::from(synopsis_text.trim())
		}
	} else {
		String::from("Manga disponible sur AnimeSama")
	};
	
	// Extraire les genres depuis les liens après h2:contains(Genres)
	let mut categories: Vec<String> = Vec::new();
	
	// Simplifier la récupération des genres en utilisant le texte de la sélection
	let genre_text = html.select("#sousBlocMiddle h2:contains(Genres) + a").text().read();
	if !genre_text.is_empty() {
		// Vérifier si ce genre contient des virgules (ex: "Action, Drame, Psychologique")
		if genre_text.contains(',') {
			// Diviser par les virgules et ajouter chaque genre individuellement
			for genre in genre_text.split(',') {
				let cleaned_genre = genre.trim();
				if !cleaned_genre.is_empty() {
					categories.push(String::from(cleaned_genre));
				}
			}
		} else {
			// Genre unique, l'ajouter directement
			categories.push(String::from(genre_text.trim()));
		}
	}
	
	// Méthode 2: Si pas de genres trouvés, essayer de récupérer le texte complet des genres
	if categories.is_empty() {
		// Chercher le texte après le h2 "Genres" qui peut contenir "Action, Comédie, Horreur, Science-fiction"
		let genres_text = html.select("#sousBlocMiddle").text().read();
		
		// Chercher la section GENRES dans le texte
		if let Some(genres_start) = genres_text.find("GENRES") {
			let genres_section = &genres_text[genres_start..];
			
			// Prendre la première ligne après "GENRES" qui contient les genres séparés par des virgules
			if let Some(first_line_end) = genres_section.find('\n') {
				let genres_line = genres_section[7..first_line_end].trim(); // Skip "GENRES\n"
				
				// Diviser par les virgules et nettoyer chaque genre
				for genre in genres_line.split(',') {
					let cleaned_genre = genre.trim();
					if !cleaned_genre.is_empty() {
						categories.push(String::from(cleaned_genre));
					}
				}
			}
		}
	}
	
	// Ajouter "Manga" par défaut si aucun genre trouvé
	if categories.is_empty() {
		categories.push(String::from("Manga"));
	}
	
	// Extraire l'image de couverture depuis l'élément avec ID coverOeuvre
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
	
	Ok(Manga {
		id: manga_id.clone(),
		cover: Some(cover),
		title: manga_title,
		author: None,
		artist: None,
		description: Some(description),
		url: Some(build_manga_url(&manga_id)),
		categories: Some(categories),
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}


// Nouvelle fonction pour récupérer les chapitres depuis l'API AnimeSama
fn _get_chapters_from_api(manga_title: &str) -> Result<Vec<i32>> {
	// Construire l'URL de l'API
	let encoded_title = helper::urlencode(manga_title);
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", encoded_title);
	
	// Placeholder: Le wrapper stable ne supporte pas encore les appels HTTP réels
	// TODO: Implémenter l'appel API réel quand le wrapper supportera HTTP
	let mut chapters: Vec<i32> = (1..=50).collect(); // Générer une liste de 50 chapitres par défaut
	
	// Trier les chapitres
	chapters.sort_unstable();
	
	if chapters.is_empty() {
		return Err(AidokuError::new("No chapters found"));
	}
	
	Ok(chapters)
}

// Nouvelle fonction pour récupérer le nombre de pages depuis l'API AnimeSama
fn get_page_count_from_api(manga_name: &str, chapter_num: i32) -> Result<i32> {
	// Construire l'URL de l'API
	let encoded_title = helper::urlencode(manga_name);
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", encoded_title);
	
	// Placeholder: Le wrapper stable ne supporte pas encore les appels HTTP réels
	// TODO: Implémenter l'appel API réel quand le wrapper supportera HTTP
	let default_page_count = 20; // Nombre de pages par défaut par chapitre
	
	if default_page_count > 0 {
		Ok(default_page_count)
	} else {
		Err(AidokuError::new("No pages found for chapter"))
	}
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
	
	// Étape 1: Parser le JavaScript de mapping des chapitres
	let html_content = html.html().read();
	let chapter_mappings = parse_chapter_mapping(&html_content);
	
	// Étape 2: Utiliser l'API pour connaître le nombre TOTAL de chapitres disponibles
	let total_chapters = match get_total_chapters_from_api(&manga_name) {
		Ok(count) => count,
		Err(_) => {
			// Fallback: utiliser une valeur par défaut raisonnable
			if !chapter_mappings.is_empty() {
				// Utiliser le dernier indice des mappings + quelques chapitres
				chapter_mappings.iter().map(|m| m.index).max().unwrap_or(100) + 50
			} else {
				100 // Valeur par défaut
			}
		}
	};
	
	// Étape 3: Créer tous les chapitres de 1 au maximum total
	// En utilisant les mappings JavaScript quand ils existent
	for index in 1..=total_chapters {
		// Chercher si un mapping existe pour cet indice
		if let Some(mapping) = chapter_mappings.iter().find(|m| m.index == index) {
			// Utiliser le mapping JavaScript
			chapters.push(Chapter {
				id: format!("{}", mapping.index),
				title: Some(mapping.title.clone()),
				volume: None,
				chapter: Some(mapping.chapter_number),
				date_updated: None,
				scanlator: None,
				url: None,
				lang: String::from("fr")
			});
		} else {
			// Pas de mapping, utiliser numérotation normale
			// MAIS ajuster pour les chapitres spéciaux qui "décalent" la numérotation
			let chapter_number = calculate_chapter_number_for_index(index, &chapter_mappings);
			
			chapters.push(Chapter {
				id: format!("{}", index),
				title: Some(format!("Chapitre {}", chapter_number as i32)),
				volume: None,
				chapter: Some(chapter_number),
				date_updated: None,
				scanlator: None,
				url: None,
				lang: String::from("fr")
			});
		}
	}
	
	// Tri par numéro de chapitre (du plus récent au plus ancien)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(Ordering::Equal));
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
			title: Some(format!("Chapitre {}", i)),
			volume: None,
			chapter: Some(i as f32),
			date_updated: None,
			scanlator: None,
			url: Some(build_chapter_url(&manga_id)),
			lang: String::from("fr")
		});
	}
	
	// Inverser pour avoir les derniers en premier
	chapters.reverse();
	
	Ok(chapters)
}

// Fonction pour parser episodes.js (méthode Tachiyomi)
fn _parse_episodes_js(manga_id: &str, html: &Node) -> Result<Vec<Chapter>> {
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
	
	// Placeholder: Le wrapper stable ne supporte pas encore les appels HTTP réels
	// TODO: Faire la requête vers episodes.js quand le wrapper supportera HTTP  
	match Ok::<String, AidokuError>(String::from("")) {
		Ok(js_content) => {
			_parse_episodes_content(&js_content, manga_id)
		}
		Err(_) => Ok(Vec::new()) // Échec de la requête
	}
}

// Fonction pour parser directement le select HTML (méthode prioritaire)
fn _parse_chapter_list_from_select(html: &Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// NOUVEAU: Parser directement le HTML brut au lieu d'utiliser les sélecteurs CSS
	let html_content = html.html().read();
	let regex_chapters = _parse_options_from_raw_html(&html_content);
	
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
					title: Some(format!("Chapitre {}", chapter_num)),
					volume: None,
					chapter: Some(*chapter_num as f32),
					date_updated: None,
					scanlator: None,
					url: None, // Sera rempli plus tard par build_chapter_url
					lang: String::from("fr")
				});
			}
			
			// Debug pour confirmer le succès
			chapters.push(Chapter {
				id: String::from("debug_regex_success"),
				title: Some(format!("DEBUG: PARSING HTML BRUT RÉUSSI - {} chapitres (min: {}, max: {})", chapter_count, min_ch, max_ch)),
				volume: None,
				chapter: None,
				date_updated: None,
				scanlator: Some(String::from("AnimeSama Debug")),
				url: None,
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
				title: Some(format!("DEBUG: HTML ANALYSIS - {}", debug_msg)),
				volume: None,
				chapter: None,
				date_updated: None,
				scanlator: Some(String::from("AnimeSama Debug")),
				url: None,
				lang: String::from("fr")
			});
			
			// Continuer vers les fallbacks
		}
	}
	
	// Si le parsing HTML brut échoue, essayer les sélecteurs CSS comme fallback
	let mut select_options = html.select("select option");
	let mut options_array = select_options.array();
	let mut options_count = options_array.len();
	
	if options_count == 0 {
		select_options = html.select("option");
		options_array = select_options.array();
		options_count = options_array.len();
	}
	
	if options_count == 0 {
		select_options = html.select("#selectChapitres option");
		options_array = select_options.array();
		options_count = options_array.len();
	}
	
	// Ajouter debug pour voir ce qu'on trouve exactement
	chapters.push(Chapter {
		id: String::from("debug_select_info"),
		title: Some(format!("DEBUG: HTML brut échoué, CSS: {} options trouvées", options_count)),
		volume: None,
		chapter: None,
		date_updated: None,
		scanlator: Some(String::from("AnimeSama Debug")),
		url: None,
		lang: String::from("fr")
	});
	
	if options_count == 0 {
		return Ok(chapters);
	}
	
	let mut max_chapter = 0;
	let mut min_chapter = i32::MAX;
	let mut debug_options: Vec<String> = Vec::new();
	
	for option in options_array {
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
						title: Some(format!("Chapitre {}", chapter_num)),
						volume: None,
						chapter: Some(chapter_num as f32),
						date_updated: None,
						scanlator: None,
						url: None, // Sera rempli plus tard par build_chapter_url
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
		title: Some(format!("DEBUG: {}", debug_text)),
		volume: None,
		chapter: None,
		date_updated: None,
		scanlator: Some(String::from("AnimeSama Debug")),
		url: None,
		lang: String::from("fr")
	});
	
	Ok(chapters)
}

// Nouvelle fonction pour parser les options directement depuis le HTML brut
fn _parse_options_from_raw_html(html_content: &str) -> Vec<i32> {
	let mut chapters: Vec<i32> = Vec::new();
	
	// DEBUG: Ajouter un chapitre avec un extrait du HTML pour voir ce qu'on reçoit vraiment
	let _html_excerpt = if html_content.len() > 500 {
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
fn _parse_episodes_content(js_content: &str, manga_id: &str) -> Result<Vec<Chapter>> {
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
		let _total_chapters = max_episode - min_episode + 1;
		
		for episode_num in min_episode..=max_episode {
			chapters.push(Chapter {
				id: format!("{}", episode_num),
				title: Some(format!("Chapitre {}", episode_num)),
				volume: None,
				chapter: Some(episode_num as f32),
				date_updated: None,
				scanlator: None,
				url: Some(build_chapter_url(manga_id)),
				lang: String::from("fr")
			});
		}
		
	}
	
	Ok(chapters)
}

// Helper pour parser les numéros de chapitre depuis les messages de la page
fn _parse_chapter_from_message(html_content: &str) -> i32 {
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
fn _parse_javascript_commands(html_content: &str, manga_id: &str) -> Result<Vec<Chapter>> {
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
							title: Some(format!("Chapitre {}", i)),
							volume: None,
							chapter: Some(i as f32),
							date_updated: None,
							scanlator: None,
							url: Some(build_chapter_url(manga_id)),
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
					title: Some(format!("Chapitre {}", special_title)),
					volume: None,
					chapter: Some(chapter_counter as f32),
					date_updated: None,
					scanlator: None,
					url: Some(build_chapter_url(manga_id)),
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
		if let Ok(episodes_max) = _get_max_episode_from_js(manga_id) {
			if episodes_max > finir_liste_start {
				// Continuer de finir_liste_start jusqu'à episodes_max
				for i in finir_liste_start..=episodes_max {
					chapters.push(Chapter {
						id: format!("{}", i),
						title: Some(format!("Chapitre {}", i)),
						volume: None,
						chapter: Some(i as f32),
						date_updated: None,
						scanlator: None,
						url: Some(build_chapter_url(manga_id)),
						lang: String::from("fr")
					});
				}
			}
		}
	}
	
	Ok(chapters)
}

// Helper pour obtenir le maximum d'épisode depuis episodes.js
fn _get_max_episode_from_js(manga_id: &str) -> Result<i32> {
	// Essayer de récupérer episodes.js rapidement juste pour le maximum
	let is_one_piece = manga_id.contains("one-piece") || manga_id.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	let episodes_url = if manga_id.starts_with("http") {
		format!("{}{}episodes.js", manga_id, scan_path)
	} else {
		format!("{}{}{}episodes.js", String::from(BASE_URL), manga_id, scan_path)
	};
	
	// Placeholder: Le wrapper stable ne supporte pas encore les appels HTTP réels
	// TODO: Faire la requête vers episodes.js quand le wrapper supportera HTTP
	match Ok::<String, AidokuError>(String::from("")) {
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
		Err(_) => Err(AidokuError::new("Failed to get max episode"))
	}
}

pub fn _parse_chapter_list_simple(manga_id: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Défaut simple : 50 chapitres
	for i in 1..=50 {
		chapters.push(Chapter {
			id: format!("{}", i),
			title: Some(format!("Chapitre {}", i)),
			volume: None,
			chapter: Some(i as f32),
			date_updated: None,
			scanlator: None,
			url: Some(build_chapter_url(&manga_id)),
			lang: String::from("fr")
		});
	}
	
	// Inverser pour avoir les derniers en premier
	chapters.reverse();
	
	Ok(chapters)
}


pub fn parse_page_list(html: Node, manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// chapter_id est l'indice API à utiliser directement dans les URLs
	let chapter_index = chapter_id.parse::<i32>().unwrap_or(1);
	
	// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
	let manga_slug = manga_id.split('/').last().unwrap_or("manga");
	
	// Extraire le titre du manga depuis le HTML pour construire les URLs CDN
	let mut manga_title = String::new();
	
	// Méthode 1: Chercher #titreOeuvre (page principale manga)
	let title_from_element = html.select("#titreOeuvre").text().read();
	if !title_from_element.is_empty() {
		manga_title = title_from_element;
	}
	
	// Méthode 2: Extraire depuis le <title> de la page (ex: "Kaiju N°8 - Scans")
	if manga_title.is_empty() {
		let page_title = html.select("title").text().read();
		if !page_title.is_empty() && page_title.contains(" - ") {
			// Extraire la partie avant " - Scans" ou " - "
			manga_title = String::from(page_title.split(" - ").next().unwrap_or("").trim());
		}
	}
	
	// Méthode 3: Chercher dans les éléments h1 qui peuvent contenir le titre
	if manga_title.is_empty() {
		let h1_text = html.select("h1").text().read();
		if !h1_text.is_empty() {
			manga_title = h1_text;
		}
	}
	
	// Fallback final: convertir le slug en titre avec gestion des cas spéciaux
	if manga_title.is_empty() {
		manga_title = match manga_slug {
			"kaiju-n8" => String::from("Kaiju N°8"), // Cas spécial avec symbole degré
			_ => {
				// Conversion générique: slug -> Title Case
				manga_slug.replace('-', " ")
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
			}
		};
	}
	
	// PRIORITÉ 1 : Parser le JavaScript dans le HTML pour trouver les patterns eps{number}
	let html_content = html.html().read();
	let page_count = parse_episodes_js_from_html(&html_content, chapter_index);
	
	if page_count > 0 {
		// Succès avec le parsing JavaScript - utiliser l'indice API dans l'URL
		for i in 1..=page_count {
			let page_url = format!("{}/{}/{}/{}.jpg", 
				String::from(CDN_URL), 
				helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
				chapter_index, 
				i
			);
			pages.push(Page {
				content: PageContent::url(page_url),
				has_description: false,
				description: None
			});
		}
		return Ok(pages);
	}
	
	// PRIORITÉ 2 : Fallback avec l'API AnimeSama
	match get_page_count_from_api(&manga_title, chapter_index) {
		Ok(api_page_count) => {
			for i in 1..=api_page_count {
				let page_url = format!("{}/{}/{}/{}.jpg", 
					String::from(CDN_URL), 
					helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
					chapter_index, 
					i
				);
				pages.push(Page {
					content: PageContent::url(page_url),
					has_description: false,
					description: None
				});
			}
			return Ok(pages);
		}
		Err(_) => {
			// L'API a aussi échoué
		}
	}
	
	// PRIORITÉ 3 : Fallback ultime - 20 pages par défaut
	for i in 1..=20 {
		let page_url = format!("{}/{}/{}/{}.jpg", 
			String::from(CDN_URL), 
			helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
			chapter_index, 
			i
		);
		pages.push(Page {
			content: PageContent::url(page_url),
			has_description: false,
			description: None
		});
	}
	
	Ok(pages)
}

// Fonction pour parser les patterns eps{number} dans le JavaScript (inspirée de Tachiyomi)
fn parse_episodes_js_from_html(html_content: &str, chapter_num: i32) -> i32 {
	// Regex inspirée de Tachiyomi : eps(\d+)\s*(?:=\s*\[(.*?)]|\.length\s*=\s*(\d+))
	let mut start_pos = 0;
	let eps_pattern = format!("eps{}", chapter_num);
	
	// Chercher le pattern eps{chapter_num} dans le HTML
	while let Some(pos) = html_content[start_pos..].find(&eps_pattern) {
		start_pos += pos + eps_pattern.len();
		let remaining = &html_content[start_pos..];
		
		// Cas 1: eps123.length = 15
		if remaining.starts_with(".length") {
			if let Some(eq_pos) = remaining[7..].find('=') {
				let after_eq = &remaining[7 + eq_pos + 1..];
				let mut number_str = String::new();
				
				for ch in after_eq.trim().chars() {
					if ch.is_ascii_digit() {
						number_str.push(ch);
					} else {
						break;
					}
				}
				
				if let Ok(page_count) = number_str.parse::<i32>() {
					if page_count > 0 && page_count <= 100 {
						return page_count;
					}
				}
			}
		}
		
		// Cas 2: eps123 = ["page1", "page2", ...]
		if remaining.trim_start().starts_with('=') {
			let after_eq = &remaining[remaining.find('=').unwrap() + 1..];
			if let Some(bracket_start) = after_eq.find('[') {
				if let Some(bracket_end) = after_eq[bracket_start..].find(']') {
					let array_content = &after_eq[bracket_start + 1..bracket_start + bracket_end];
					// Compter les éléments (approximatif)
					let item_count = array_content.matches(',').count() + 1;
					if item_count > 0 && item_count <= 100 {
						return item_count as i32;
					}
				}
			}
		}
		
		start_pos += 1;
	}
	
	0 // Aucun pattern trouvé
}