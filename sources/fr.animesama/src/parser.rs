use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::html::Document,
	imports::net::Request,
};

use crate::{BASE_URL, CDN_URL, CDN_URL_LEGACY, helper};

// Fonction pour déterminer quel CDN utiliser selon le manga
fn select_cdn_url(manga_title: &str) -> &'static str {
	// Mangas qui utilisent l'ancien CDN
	match manga_title {
		"One Piece" | "Dragon Ball" => CDN_URL_LEGACY,
		_ => CDN_URL, // Nouveau CDN par défaut
	}
}

// Structure pour stocker les mappings de chapitres depuis JavaScript
#[derive(Debug, Clone)]
struct ChapterMapping {
	index: i32,
	chapter_number: f32,
	title: String,
}

// Structure pour stocker les informations de finirListe
#[derive(Debug, Clone)]
struct FinirListeInfo {
	start_index: i32, // Index où commencer la numérotation séquentielle
}

// Parser les commandes JavaScript pour créer le mapping
fn parse_chapter_mapping(html_content: &str) -> (Vec<ChapterMapping>, Option<FinirListeInfo>) {
	let mut mappings: Vec<ChapterMapping> = Vec::new();
	let mut current_index = 1; // Indices commencent à 1
	let mut finir_liste_info: Option<FinirListeInfo> = None;
	
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
			let finir_pos = trimmed[line_pos..].find("finirListe(");
			
			// Déterminer quel pattern vient en premier
			let next_pattern = match (creer_pos, newsp_pos, finir_pos) {
				(Some(c), Some(n), Some(f)) => {
					if c < n && c < f { ("creer", line_pos + c) }
					else if n < f { ("newsp", line_pos + n) }
					else { ("finir", line_pos + f) }
				}
				(Some(c), Some(_n), None) if c < _n => ("creer", line_pos + c),
				(Some(_c), Some(n), None) => ("newsp", line_pos + n),
				(Some(c), None, Some(f)) if c < f => ("creer", line_pos + c),
				(Some(_c), None, Some(f)) => ("finir", line_pos + f),
				(None, Some(n), Some(f)) if n < f => ("newsp", line_pos + n),
				(None, Some(_n), Some(f)) => ("finir", line_pos + f),
				(Some(c), None, None) => ("creer", line_pos + c),
				(None, Some(n), None) => ("newsp", line_pos + n),
				(None, None, Some(f)) => ("finir", line_pos + f),
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
				"finir" => {
					let start_pos = next_pattern.1 + "finirListe(".len();
					if let Some(params_end) = trimmed[start_pos..].find(");") {
						let param = trimmed[start_pos..start_pos + params_end].trim();
						
						// Parser le paramètre de finirListe (devrait être un nombre)
						if let Ok(finir_start) = param.parse::<i32>() {
							finir_liste_info = Some(FinirListeInfo {
								start_index: finir_start,
							});
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
	
	(mappings, finir_liste_info)
}

fn calculate_chapter_number_for_index(index: i32, mappings: &[ChapterMapping], finir_liste_info: &Option<FinirListeInfo>) -> f32 {
	// Pour les indices non mappés après finirListe(), continuer la numérotation séquentielle
	// Ex: Dandadan finirListe(27) → indice 28 = chapitre 27, indice 29 = chapitre 28, etc.
	
	if mappings.is_empty() && finir_liste_info.is_none() {
		return index as f32;
	}
	
	// Si on a des mappings mais pas de finir_liste_info (ex: One Piece avec One Shot)
	if !mappings.is_empty() && finir_liste_info.is_none() {
		// Compter combien de mappings spéciaux sont avant cet index
		let special_before = mappings.iter()
			.filter(|m| m.index <= index && (
				m.title.contains("One Shot") ||
				m.title.contains("Prologue") ||
				m.title.contains("Epilogue") ||
				m.title.contains("Extra") ||
				m.title.contains("Special") ||
				m.chapter_number != (m.chapter_number as i32 as f32) // Chapitres décimaux
			))
			.count() as i32;
		
		// Pour One Piece: après le One Shot, décaler les numéros
		// Index 1047 → Chapter 1046, Index 1048 → Chapter 1047, etc.
		return (index - special_before) as f32;
	}
	
	// Si on a une info finirListe, utiliser cette logique
	if let Some(finir_info) = finir_liste_info {
		if index >= finir_info.start_index {
			// Pour les indices après finirListe, calculer le numéro de chapitre correct
			// Trouver le dernier chapitre numérique dans les mappings
			let mut last_numeric_chapter = 0.0;
			for mapping in mappings {
				if mapping.title.chars().any(|c| c.is_ascii_digit()) && !mapping.title.contains("One Shot") {
					// Extraire le numéro du titre "Chapitre 19.5"
					let title_parts: Vec<&str> = mapping.title.split_whitespace().collect();
					if title_parts.len() >= 2 {
						if let Ok(chapter_num) = title_parts[1].parse::<f32>() {
							if chapter_num > last_numeric_chapter {
								last_numeric_chapter = chapter_num;
							}
						}
					}
				}
			}
			
			// Calculer le prochain numéro de chapitre entier après le dernier chapitre numérique
			let fractional_part = last_numeric_chapter - (last_numeric_chapter as i32 as f32);
			let next_chapter_base = if fractional_part > 0.0 {
				// Si c'est un décimal (ex: 19.5), le prochain entier est 20
				(last_numeric_chapter as i32 + 1) as f32
			} else {
				// Si c'est déjà un entier (ex: 19), le prochain est 20
				last_numeric_chapter + 1.0
			};
			
			// Les chapitres après finirListe reprennent une numérotation entière normale
			let chapters_after_finir = index - finir_info.start_index;
			return next_chapter_base + chapters_after_finir as f32;
		}
	}
	
	// Fallback pour les cas sans finirListe : utiliser l'index comme numéro de chapitre
	index as f32
}

// Chercher des chapitres décimaux dans une ligne HTML/JavaScript
fn find_decimal_chapter_in_line(line: &str) -> Option<f32> {
	// Chercher des patterns comme "19.5", "20.5" dans du JavaScript ou HTML
	let mut chars = line.chars().peekable();
	let mut current_number = String::new();
	
	while let Some(ch) = chars.next() {
		if ch.is_ascii_digit() {
			current_number.push(ch);
			
			// Chercher le point décimal
			if chars.peek() == Some(&'.') {
				current_number.push(chars.next().unwrap()); // Consommer le '.'
				
				// Chercher le chiffre après le point
				if let Some(next_ch) = chars.peek() {
					if next_ch.is_ascii_digit() {
						current_number.push(chars.next().unwrap());
						
						// Parser le nombre décimal
						if let Ok(decimal_num) = current_number.parse::<f32>() {
							// Vérifier que c'est un chapitre plausible (entre 0.1 et 999.9)
							// Et que ce n'est pas un ".0" (comme "19.0")
							if decimal_num >= 0.1 && decimal_num <= 999.9 && !current_number.ends_with(".0") {
								return Some(decimal_num);
							}
						}
					}
				}
			}
			
			// Si pas de décimal, continuer
			current_number.clear();
		} else {
			current_number.clear();
		}
	}
	
	None
}

// Chercher des chapitres spéciaux dans une ligne HTML/JavaScript
fn find_special_chapter_in_line(line: &str) -> Option<&str> {
	let special_patterns = [
		"One Shot", "one shot", "One shot", "ONE SHOT",
		"Prologue", "prologue", "PROLOGUE", 
		"Epilogue", "epilogue", "EPILOGUE",
		"Extra", "extra", "EXTRA",
		"Special", "special", "SPECIAL"
	];
	
	for pattern in &special_patterns {
		if line.contains(pattern) {
			return Some(pattern);
		}
	}
	
	None
}

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
				
				// Nettoyer l'URL pour supprimer /scan/vf/ et obtenir l'URL de base du manga
				// Exactement comme dans l'ancienne version
				let clean_url = if relative_url.contains("/scan/vf/") {
					relative_url.replace("/scan/vf/", "")
				} else if relative_url.contains("/scan_noir-et-blanc/vf/") {
					relative_url.replace("/scan_noir-et-blanc/vf/", "")
				} else {
					relative_url.clone()
				};
				
				mangas.push(Manga {
					key: clean_url.clone(),
					cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
					title,
					authors: None,
					artists: None,
					description: None,
					url: Some(if clean_url.starts_with("http") {
						clean_url.clone()
					} else {
						format!("{}{}", BASE_URL, clean_url)
					}),
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
					
					// Nettoyer l'URL pour supprimer /scan/vf/ et obtenir l'URL de base du manga
					// Exactement comme dans l'ancienne version
					let clean_url = if relative_url.contains("/scan/vf/") {
						relative_url.replace("/scan/vf/", "")
					} else if relative_url.contains("/scan_noir-et-blanc/vf/") {
						relative_url.replace("/scan_noir-et-blanc/vf/", "")
					} else {
						relative_url.clone()
					};
					
					mangas.push(Manga {
						key: clean_url.clone(),
						cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
						title,
						authors: None,
						artists: None,
						description: None,
						url: Some(if clean_url.starts_with("http") {
							clean_url.clone()
						} else {
							format!("{}{}", BASE_URL, clean_url)
						}),
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
	// Parser le titre avec plusieurs méthodes
	let title = html.select("#titreOeuvre")
		.and_then(|els| els.text())
		.or_else(|| {
			// Fallback 1: titre de la page
			html.select("title").and_then(|els| els.text()).and_then(|title_text| {
				if title_text.contains(" - ") {
					Some(title_text.split(" - ").next()?.trim().to_string())
				} else {
					Some(title_text.trim().to_string())
				}
			})
		})
		.or_else(|| {
			// Fallback 2: h1 sur la page
			html.select("h1").and_then(|els| els.text())
		})
		.unwrap_or_else(|| manga_key_to_title(&manga_key));
	
	// Extraire la description - utiliser le sélecteur exact de l'ancienne version
	let description = {
		// Méthode 1: Sélecteur direct comme l'ancienne version
		if let Some(synopsis_p) = html.select("#sousBlocMiddle h2:contains(Synopsis) + p") {
			if let Some(first_p) = synopsis_p.first() {
				if let Some(desc_text) = first_p.text() {
					let trimmed = desc_text.trim();
					if !trimmed.is_empty() && trimmed.len() > 10 {
						trimmed.to_string()
					} else {
						"Manga disponible sur AnimeSama".to_string()
					}
				} else {
					"Manga disponible sur AnimeSama".to_string()
				}
			} else {
				"Manga disponible sur AnimeSama".to_string()
			}
		} else {
			// Fallback: chercher manuellement dans tout le sousBlocMiddle
			if let Some(full_text) = html.select("#sousBlocMiddle").and_then(|els| els.text()) {
				// Chercher après le mot "Synopsis" dans le texte complet
				if let Some(synopsis_pos) = full_text.to_uppercase().find("SYNOPSIS") {
					let after_synopsis = &full_text[synopsis_pos + 8..]; // Skip "SYNOPSIS"
					// Prendre les prochaines lignes jusqu'à GENRES ou autre section
					if let Some(end_pos) = after_synopsis.to_uppercase().find("GENRES") {
						let desc_section = &after_synopsis[..end_pos];
						let cleaned = desc_section.trim();
						if !cleaned.is_empty() && cleaned.len() > 20 {
							cleaned.to_string()
						} else {
							"Manga disponible sur AnimeSama".to_string()
						}
					} else {
						// Prendre les premiers 500 caractères après Synopsis
						let desc_section = if after_synopsis.len() > 500 {
							&after_synopsis[..500]
						} else {
							after_synopsis
						};
						let cleaned = desc_section.trim();
						if !cleaned.is_empty() && cleaned.len() > 20 {
							cleaned.to_string()
						} else {
							"Manga disponible sur AnimeSama".to_string()
						}
					}
				} else {
					"Manga disponible sur AnimeSama".to_string()
				}
			} else {
				"Manga disponible sur AnimeSama".to_string()
			}
		}
	};
	
	// Parser la couverture avec fallback
	let cover = html.select("#coverOeuvre")
		.and_then(|els| els.first())
		.and_then(|el| el.attr("src"))
		.or_else(|| {
			// Fallback: chercher d'autres sélecteurs d'image
			let selectors = vec!["img.cover", ".manga-cover img", ".poster img", "img[src*='cover']"];
			for selector in selectors {
				if let Some(src) = html.select(selector).and_then(|els| els.first()).and_then(|el| el.attr("src")) {
					return Some(src);
				}
			}
			None
		})
		.map(|src| {
			if src.starts_with("http") {
				src
			} else {
				format!("{}{}", BASE_URL, src)
			}
		});
	
	// Extraire les genres - logique exacte de l'ancienne version adaptée
	let mut tags: Vec<String> = Vec::new();
	
	// Méthode 1: Essayer de récupérer les liens individuellement après h2:contains(Genres) - exactement comme l'ancienne version
	if let Some(genre_elements) = html.select("#sousBlocMiddle h2:contains(Genres) + a") {
		for genre_elem in genre_elements {
			if let Some(genre_text) = genre_elem.text() {
				let genre_raw = genre_text.trim();
				if !genre_raw.is_empty() {
					// Vérifier si ce genre contient des séparateurs (virgules ou tirets)
					if genre_raw.contains(',') || genre_raw.contains(" - ") {
						// Diviser par les virgules ou tirets et ajouter chaque genre individuellement
						let separator = if genre_raw.contains(',') { "," } else { " - " };
						for genre in genre_raw.split(separator) {
							let cleaned_genre = genre.trim();
							if !cleaned_genre.is_empty() {
								tags.push(cleaned_genre.to_string());
							}
						}
					} else {
						// Genre unique, l'ajouter directement
						tags.push(genre_raw.to_string());
					}
				}
			}
		}
	}
	
	// Méthode 2: Si pas de genres trouvés, essayer de récupérer le texte complet des genres - exactement comme l'ancienne version
	if tags.is_empty() {
		// Chercher le texte après le h2 "Genres" qui peut contenir "Action, Comédie, Horreur, Science-fiction"
		if let Some(genres_text) = html.select("#sousBlocMiddle").and_then(|el| el.text()) {
			// Chercher la section GENRES dans le texte
			if let Some(genres_start) = genres_text.find("GENRES") {
				let genres_section = &genres_text[genres_start..];
				
				// Prendre la première ligne après "GENRES" qui contient les genres séparés par des virgules
				if let Some(first_line_end) = genres_section.find('\n') {
					let genres_line = &genres_section[7..first_line_end].trim(); // Skip "GENRES\n"
					
					// Diviser par les virgules ou tirets et nettoyer chaque genre
					let separator = if genres_line.contains(',') { "," } else { " - " };
					for genre in genres_line.split(separator) {
						let cleaned_genre = genre.trim();
						if !cleaned_genre.is_empty() {
							tags.push(cleaned_genre.to_string());
						}
					}
				}
			}
		}
	}
	
	// Ajouter "Manga" par défaut si aucun genre trouvé
	if tags.is_empty() {
		tags.push("Manga".to_string());
	}
	
	// Parser le statut - exactement comme dans l'ancienne version
	let status = MangaStatus::Unknown;
	
	Ok(Manga {
		key: manga_key.clone(),
		title,
		authors: None,
		artists: None,
		description: Some(description),
		url: Some(if manga_key.starts_with("http") {
			manga_key.clone()
		} else {
			format!("{}{}", BASE_URL, manga_key)
		}),
		cover,
		tags: Some(tags),
		status,
		content_rating: ContentRating::Safe,
		viewer: Viewer::RightToLeft,
		..Default::default()
	})
}

pub fn parse_chapter_list(manga_key: String, html: Document) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Extract manga title from HTML for API call
	let manga_name = html.select("#titreOeuvre")
		.and_then(|els| els.text())
		.or_else(|| {
			html.select("title").and_then(|els| els.text()).and_then(|title_text| {
				if title_text.contains(" - ") {
					Some(title_text.split(" - ").next()?.trim().to_string())
				} else {
					Some(title_text.trim().to_string())
				}
			})
		})
		.unwrap_or_else(|| manga_key_to_title(&manga_key));
	
	// Parse JavaScript content for chapter mappings
	// IMPORTANT: Récupérer TOUT le contenu de la page, y compris les scripts inline
	let mut html_content = String::new();
	
	// Récupérer le contenu des scripts
	if let Some(scripts) = html.select("script") {
		for script in scripts {
			if let Some(script_text) = script.text() {
				html_content.push_str(&script_text);
				html_content.push('\n');
			}
		}
	}
	
	// Récupérer aussi le HTML brut complet pour chercher dans les commentaires et autres endroits
	if let Some(body) = html.select("body") {
		for element in body {
			// Récupérer le HTML complet de l'élément (peut contenir du JS dans les commentaires)
			if let Some(full_html) = element.html() {
				html_content.push_str(&full_html);
				html_content.push('\n');
			}
			
			// Chercher dans tous les attributs qui pourraient contenir du JavaScript
			for attr_name in ["onclick", "onload", "data-script", "data-js", "data-chapters"] {
				if let Some(attr_value) = element.attr(attr_name) {
					html_content.push_str(&attr_value);
					html_content.push('\n');
				}
			}
		}
	}
	
	// Chercher aussi dans les commentaires HTML de toute la page
	if let Some(head) = html.select("head") {
		for element in head {
			if let Some(head_html) = element.html() {
				html_content.push_str(&head_html);
				html_content.push('\n');
			}
		}
	}
	
	// Parse JavaScript commands to create chapter mappings
	let (mut chapter_mappings, finir_liste_info) = parse_chapter_mapping(&html_content);
	
	// Si aucun mapping JavaScript trouvé, essayer une détection basique de chapitres spéciaux
	if chapter_mappings.is_empty() && finir_liste_info.is_none() {
		// Cas spécial : One Piece sans JavaScript détecté - ajouter le One Shot manuellement
		if manga_name.to_lowercase().contains("one piece") || manga_key.contains("one-piece") {
			// Vérifier si on a des chapitres > 1045 avant d'ajouter le One Shot
			let max_cdn_chapter = get_max_available_chapter_on_cdn(&manga_name);
			
			if max_cdn_chapter > 1045 {
				// Ajouter le One Shot uniquement si le CDN a des chapitres après 1045
				chapter_mappings.push(ChapterMapping {
					index: 1046, // Position entre 1045 et 1046
					chapter_number: 1045.5, // Numéro pour tri correct
					title: "Chapitre One Shot".to_string(),
				});
			}
		}
		// Pour les autres mangas, ne pas chercher de chapitres décimaux car cela génère des faux positifs
		// Les chapitres décimaux seront gérés par les mappings JavaScript uniquement
	}
	
	// Get total chapters from API (this is the highest chapter NUMBER, not index count)
	let api_max_chapter = match get_total_chapters_from_api(&manga_name) {
		Ok(count) => {
			// Limiter pour certains mangas où l'API retourne des nombres incorrects
			if manga_name.to_lowercase().contains("versatile mage") || manga_key.contains("versatile-mage") {
				// Pour Versatile Mage, utiliser un maximum raisonnable car l'API semble incorrecte
				count.min(50) // Limiter à 50 chapitres max pour éviter les erreurs
			} else {
				count
			}
		},
		Err(_) => {
			// Fallback: reasonable default
			if !chapter_mappings.is_empty() {
				chapter_mappings.iter().map(|m| m.index).max().unwrap_or(100) + 50
			} else {
				50 // Plus conservateur pour éviter les problèmes
			}
		}
	};
	
	// Count special chapters that take indices but don't match sequential numbering
	let special_chapter_count = chapter_mappings.iter().filter(|mapping| {
		// A mapping is special if it's:
		// - "One Shot", "Prologue", etc. (text-based)
		// - Decimal chapter (.5) that doesn't match its index
		// - Any chapter whose number doesn't match its expected sequential position
		let is_text_chapter = mapping.title.contains("One Shot") 
			|| mapping.title.contains("Prologue") 
			|| mapping.title.contains("Epilogue")
			|| mapping.title.contains("Extra")
			|| mapping.title.contains("Special");
		
		// Check if it's a decimal chapter
		let is_decimal = mapping.chapter_number != (mapping.chapter_number as i32 as f32);
		
		is_text_chapter || is_decimal
	}).count() as i32;
	
	// Calculate total indices needed: API max chapter + special chapters
	// This ensures we have enough indices to create all regular chapters (1 to api_max_chapter)
	// even when special chapters occupy intermediate indices
	let total_chapters = if manga_name.to_lowercase().contains("one piece") || manga_key.contains("one-piece") {
		// Pour One Piece, utiliser la détection dynamique du CDN au lieu du hardcode
		let max_cdn_chapter = get_max_available_chapter_on_cdn(&manga_name);
		
		// Si le CDN a moins de chapitres que l'API, utiliser le CDN (plus fiable)
		// Ajouter 1 pour le One Shot uniquement s'il existe des chapitres > 1045
		if max_cdn_chapter > 1045 {
			max_cdn_chapter + 1 // +1 pour le One Shot à l'index 1046
		} else {
			max_cdn_chapter // Pas de One Shot si on n'a que 1045 chapitres ou moins
		}
	} else if !chapter_mappings.is_empty() {
		let max_mapped_index = chapter_mappings.iter().map(|m| m.index).max().unwrap_or(0);
		// Use the maximum between calculated total and actual mappings (no unnecessary buffer)
		(api_max_chapter + special_chapter_count).max(max_mapped_index)
	} else {
		// Sans mappings spéciaux, utiliser exactement le nombre de l'API (pas plus)
		api_max_chapter
	};
	
	// Create chapters from 1 to total, using JavaScript mappings when available
	for index in 1..=total_chapters {
		if let Some(mapping) = chapter_mappings.iter().find(|m| m.index == index) {
			// Use JavaScript mapping
			// For special chapters, use a safe key that doesn't conflict
			let chapter_key = if mapping.title.contains("One Shot") {
				// One Shot: use a high number that won't conflict with regular chapters
				"9999".to_string() // Safe key that won't conflict
			} else {
				// For other special chapters, use their chapter number
				(mapping.chapter_number as i32).to_string()
			};
			
			chapters.push(Chapter {
				key: chapter_key,
				title: Some(mapping.title.clone()),
				chapter_number: Some(mapping.chapter_number),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec![]), // Vide comme dans l'ancienne version
				url: Some(build_chapter_url(&manga_key)),
				..Default::default()
			});
		} else {
			// No mapping, use normal numbering
			let chapter_number = calculate_chapter_number_for_index(index, &chapter_mappings, &finir_liste_info);
			
			chapters.push(Chapter {
				key: index.to_string(), // Use API index for accessing images, not calculated chapter_number
				title: Some(format!("Chapitre {}", chapter_number as i32)),
				chapter_number: Some(chapter_number),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec![]), // Vide comme dans l'ancienne version
				url: Some(build_chapter_url(&manga_key)),
				..Default::default()
			});
		}
	}
	
	// Tri par numéro de chapitre (du plus récent au plus ancien)
	// Exactement comme dans l'ancienne version: b.chapter > a.chapter
	chapters.sort_by(|a, b| {
		let a_num = a.chapter_number.unwrap_or(0.0);
		let b_num = b.chapter_number.unwrap_or(0.0);
		// Plus simple: utiliser la comparaison directe comme l'ancienne version
		b_num.partial_cmp(&a_num).unwrap_or(core::cmp::Ordering::Equal)
	});
	
	Ok(chapters)
}

pub fn parse_page_list(html: Document, manga_key: String, chapter_key: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Récupérer TOUT le contenu HTML pour chercher les variables eps
	// Les variables JavaScript peuvent être inline ou dans différents endroits
	let mut html_content = String::new();
	
	// Récupérer le contenu des scripts
	if let Some(scripts) = html.select("script") {
		for script in scripts {
			if let Some(script_text) = script.text() {
				html_content.push_str(&script_text);
				html_content.push('\n');
			}
		}
	}
	
	// Récupérer aussi le JavaScript inline dans les attributs
	if let Some(body) = html.select("body") {
		for element in body {
			for attr_name in ["onclick", "onload", "data-script", "data-js"] {
				if let Some(attr_value) = element.attr(attr_name) {
					html_content.push_str(&attr_value);
					html_content.push('\n');
				}
			}
		}
	}
	
	// Méthode 1: Chercher les variables d'épisode JavaScript (ex: eps1, eps2)
	// Special handling for One Shot chapter
	let mut episode_patterns = Vec::new();
	let fallback_chapter = if chapter_key == "9999" {
		// One Shot: essayer plusieurs patterns possibles
		episode_patterns.push("eps1045".to_string()); // Peut être stocké comme 1045
		episode_patterns.push("epsOneShot".to_string()); // Peut être nommé OneShot
		episode_patterns.push("epsOS".to_string()); // Abbreviation possible
		episode_patterns.push("eps0".to_string()); // Peut être à l'index 0
		1045 // Fallback vers 1045
	} else {
		episode_patterns.push(format!("eps{}", chapter_key));
		chapter_key.parse::<i32>().unwrap_or(1)
	};
	
	// Essayer tous les patterns jusqu'à en trouver un qui fonctionne
	let mut found_episode_section = None;
	for pattern in &episode_patterns {
		if let Some(episode_start) = html_content.find(pattern) {
			found_episode_section = Some((episode_start, pattern.clone()));
			break;
		}
	}
	
	// Stocker les informations JavaScript pour fallback plus tard
	let mut google_drive_pages: Vec<Page> = Vec::new();
	if let Some((episode_start, _pattern)) = found_episode_section {
		// Trouver la fin de la déclaration de variable
		if let Some(episode_section) = html_content[episode_start..].find('[') {
			let array_start = episode_start + episode_section;
			if let Some(array_end) = html_content[array_start..].find("];") {
				let array_content = &html_content[array_start + 1..array_start + array_end];
				
				// Parser les URLs dans le tableau JavaScript (pour fallback)
				for line in array_content.lines() {
					let trimmed = line.trim();
					if trimmed.starts_with("'https://drive.google.com") || trimmed.starts_with("\"https://drive.google.com") {
						let url_start = trimmed.find("https://").unwrap_or(0);
						let url_end = if trimmed[url_start..].contains('\'') {
							url_start + trimmed[url_start..].find('\'').unwrap_or(trimmed.len())
						} else {
							url_start + trimmed[url_start..].find('"').unwrap_or(trimmed.len())
						};
						
						if url_end > url_start {
							let drive_url = &trimmed[url_start..url_end];
							google_drive_pages.push(Page {
								content: PageContent::url(drive_url.to_string()),
								thumbnail: None,
								has_description: false,
								description: None,
							});
						}
					}
				}
			}
		}
	}
	
	// PRIORITÉ 1 : Utiliser le CDN par défaut (plus fiable)
	let chapter_index = fallback_chapter;
	
	// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
	let manga_slug = manga_key.split('/').last().unwrap_or("manga");
		
		// Extraire le titre du manga depuis le HTML pour construire les URLs CDN
		let mut manga_title = String::new();
		
		// Méthode 1: Chercher #titreOeuvre (page principale manga)
		if let Some(title_elem) = html.select("#titreOeuvre").and_then(|els| els.first()) {
			if let Some(title_text) = title_elem.text() {
				let cleaned_title = clean_extracted_title(title_text.trim());
				if !cleaned_title.is_empty() {
					manga_title = cleaned_title;
				}
			}
		}
		
		// Méthode 2: Extraire depuis le <title> de la page (ex: "Kaiju N°8 - Scans")
		if manga_title.is_empty() {
			if let Some(title_elem) = html.select("title").and_then(|els| els.first()) {
				if let Some(page_title) = title_elem.text() {
					let page_title = page_title.trim();
					if !page_title.is_empty() && page_title.contains(" - ") {
						// Extraire la partie avant " - Scans" ou " - "
						let extracted = page_title.split(" - ").next().unwrap_or("").trim();
						let cleaned_title = clean_extracted_title(extracted);
						if !cleaned_title.is_empty() {
							manga_title = cleaned_title;
						}
					}
				}
			}
		}
		
		// Méthode 3: Chercher dans les éléments h1 qui peuvent contenir le titre
		if manga_title.is_empty() {
			if let Some(h1_elem) = html.select("h1").and_then(|els| els.first()) {
				if let Some(h1_text) = h1_elem.text() {
					let cleaned_title = clean_extracted_title(h1_text.trim());
					if !cleaned_title.is_empty() {
						manga_title = cleaned_title;
					}
				}
			}
		}
		
		// Fallback final: utiliser manga_key_to_title avec gestion des cas spéciaux
		if manga_title.is_empty() {
			manga_title = manga_key_to_title(manga_slug);
		}
		
		// PRIORITÉ 1 : Parser le JavaScript dans le HTML pour trouver les patterns eps{number}
		let mut page_count = parse_episodes_js_from_html(&html_content, chapter_index);
		
		// Si c'est le One Shot et qu'on n'a pas trouvé de pages, essayer d'autres indices
		if page_count == 0 && chapter_key == "9999" {
			// Essayer différents indices pour le One Shot
			for test_index in [1045, 1046, 0, 9999] {
				page_count = parse_episodes_js_from_html(&html_content, test_index);
				if page_count > 0 {
					break;
				}
			}
		}
		
		if page_count > 0 {
			// Succès avec le parsing JavaScript - utiliser l'indice API dans l'URL
			for i in 1..=page_count {
				// Essayer différents formats d'images selon le manga
				let page_url = generate_image_url(&manga_title, chapter_index, i);
				pages.push(Page {
					content: PageContent::url(page_url),
					thumbnail: None,
					has_description: false,
					description: None,
				});
			}
			return Ok(pages);
		}
		
		// PRIORITÉ 2 : Fallback avec l'API AnimeSama  
		match get_page_count_from_api(&manga_title, chapter_index) {
			Ok(api_page_count) => {
				for i in 1..=api_page_count {
					let page_url = generate_image_url(&manga_title, chapter_index, i);
					pages.push(Page {
						content: PageContent::url(page_url),
						thumbnail: None,
						has_description: false,
						description: None,
					});
				}
				return Ok(pages);
			}
			Err(_) => {
				// L'API a aussi échoué - pour Versatile Mage, essayer une approche différente
				if manga_title == "Versatile Mage" && chapter_index >= 817 {
					// Pour les chapitres récents de Versatile Mage, utiliser CDN directement
					// avec une estimation conservative du nombre de pages
					for i in 1..=30 {
						let page_url = generate_image_url(&manga_title, chapter_index, i);
						pages.push(Page {
							content: PageContent::url(page_url),
							thumbnail: None,
							has_description: false,
							description: None,
						});
					}
					return Ok(pages);
				}
			}
		}
		
		// PRIORITÉ 3 : Fallback CDN - nombre de pages par défaut selon le manga
		let default_page_count = match manga_title.as_str() {
			"Versatile Mage" => 25, // Versatile Mage a souvent plus de pages
			_ => 20, // Fallback standard
		};
		
		for i in 1..=default_page_count {
			let page_url = generate_image_url(&manga_title, chapter_index, i);
			pages.push(Page {
				content: PageContent::url(page_url),
				thumbnail: None,
				has_description: false,
				description: None,
			});
		}
		
		// PRIORITÉ 4 : Google Drive en dernier recours absolu (si CDN échoue totalement)
		if pages.is_empty() && !google_drive_pages.is_empty() {
			return Ok(google_drive_pages);
		}
	
	Ok(pages)
}

// Détecter le dernier chapitre disponible sur le CDN de manière conservatrice
fn get_max_available_chapter_on_cdn(manga_title: &str) -> i32 {
	// Pour One Piece, utiliser une limite conservatrice basée sur nos tests
	if manga_title == "One Piece" {
		// D'après nos tests, le CDN s'arrête à 1045
		// Utiliser cette valeur fixe plutôt que de tester en temps réel
		1045
	} else {
		// Pour les autres mangas, utiliser l'API avec une limite raisonnable
		match get_total_chapters_from_api(manga_title) {
			Ok(count) => count.min(500), // Limiter à 500 pour éviter les erreurs
			Err(_) => 100, // Fallback conservateur pour les autres mangas
		}
	}
}

// Fonction pour obtenir le nombre de pages depuis l'API AnimeSama
fn get_page_count_from_api(manga_name: &str, chapter_num: i32) -> Result<i32> {
	// Construire l'URL de l'API
	let encoded_title = helper::urlencode(manga_name);
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", encoded_title);
	
	// Faire la requête
	let json_string = Request::get(&api_url)?.string()?;
	
	// Parser le JSON manuellement pour trouver le nombre de pages pour ce chapitre
	let chapter_key = format!("\"{}\":", chapter_num);
	if let Some(pos) = json_string.find(&chapter_key) {
		let after_key = &json_string[pos + chapter_key.len()..];
		// Chercher le nombre après les espaces/guillemets
		let mut num_str = String::new();
		for ch in after_key.trim().chars() {
			if ch.is_ascii_digit() {
				num_str.push(ch);
			} else if !num_str.is_empty() {
				break;
			}
		}
		
		if let Ok(page_count) = num_str.parse::<i32>() {
			if page_count > 0 {
				return Ok(page_count);
			}
		}
	}
	
	// Si le chapitre n'est pas trouvé dans l'API, créer une erreur UTF-8
	// Créer dynamiquement un byte array invalide pour éviter le warning sur les littéraux
	let mut invalid_bytes = Vec::new();
	invalid_bytes.push(0xFF); // Byte invalide en UTF-8
	let utf8_err = core::str::from_utf8(&invalid_bytes).unwrap_err();
	Err(utf8_err.into())
}

// Parser le JavaScript pour trouver eps{number}.length ou eps{number} = [...]
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
				
				if let Ok(length) = number_str.parse::<i32>() {
					if length > 0 {
						return length;
					}
				}
			}
		}
		
		// Cas 2: eps123 = [...]
		if let Some(eq_pos) = remaining.find('=') {
			let after_eq = &remaining[eq_pos + 1..].trim_start();
			if after_eq.starts_with('[') {
				// Compter les éléments du tableau
				if let Some(bracket_end) = after_eq.find(']') {
					let array_content = &after_eq[1..bracket_end];
					
					// Compter les virgules + 1 (approximation simple)
					let comma_count = array_content.matches(',').count();
					if comma_count > 0 {
						return (comma_count + 1) as i32;
					}
				}
			}
		}
	}
	
	0 // Aucun pattern trouvé
}

// Construire l'URL d'un chapitre avec gestion du cas spécial One Piece (sans paramètre id)
fn build_chapter_url(manga_key: &str) -> String {
	let is_one_piece = manga_key.contains("one-piece") || manga_key.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	if manga_key.starts_with("http") {
		format!("{}{}", manga_key, scan_path)
	} else {
		format!("{}{}{}", BASE_URL, manga_key, scan_path)
	}
}

// Get total chapters count from AnimeSama API
fn get_total_chapters_from_api(manga_title: &str) -> Result<i32> {
	use crate::helper::urlencode;
	
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", 
		urlencode(manga_title));
	
	match Request::get(&api_url)?.string() {
		Ok(response_text) => {
			let mut max_chapter = 0;
			
			// Parse JSON-like response to find chapter numbers
			let mut start_pos = 0;
			while let Some(quote_pos) = response_text[start_pos..].find("\"") {
				start_pos += quote_pos + 1;
				if let Some(end_quote) = response_text[start_pos..].find("\"") {
					let key = &response_text[start_pos..start_pos + end_quote];
					if let Ok(chapter_num) = key.parse::<i32>() {
						if chapter_num > max_chapter {
							max_chapter = chapter_num;
						}
					}
					start_pos += end_quote + 1;
				} else {
					break;
				}
			}
			
			if max_chapter > 0 {
				Ok(max_chapter)
			} else {
				// No chapters found, use a simple fallback
				Ok(100) // Return default count
			}
		},
		Err(e) => Err(e)
	}
}

// Convertir manga_key en titre formaté avec gestion des cas spéciaux
// Nettoyer un titre extrait du HTML
fn clean_extracted_title(title: &str) -> String {
	title
		.trim()
		.replace("\n", " ")
		.replace("\t", " ")
		.replace("  ", " ") // Réduire les espaces multiples
		.replace("&#039;", "'") // Nettoyer les entités HTML
		.replace("&quot;", "\"")
		.replace("&amp;", "&")
		.replace("&lt;", "<")
		.replace("&gt;", ">")
		.replace("(Scan)", "") // Enlever les mentions parasites
		.replace("- Scan", "")
		.replace("- Scans", "")
		.replace("Scan -", "")
		.replace("Scans -", "")
		.trim()
		.to_string()
}

// Générer l'URL d'image selon le manga spécifique (logique déterministe)
fn generate_image_url(manga_title: &str, chapter_index: i32, page: i32) -> String {
	let encoded_title = helper::urlencode_path(manga_title);
	let cdn_url = select_cdn_url(manga_title);
	
	// Détection basée sur le titre du manga pour éviter les requêtes réseau
	match manga_title {
		"One Piece" => {
			if chapter_index <= 952 {
				// Chapitres 1-952 : format normal {chapter}_{page}.jpg
				format!("{}/{}/{}/{}_{}.jpg", cdn_url, encoded_title, chapter_index, chapter_index, page)
			} else {
				// Chapitres 953+ : prefix avec décalage -952
				let prefix = chapter_index - 952;
				format!("{}/{}/{}/{}_{}.jpg", cdn_url, encoded_title, chapter_index, prefix, page)
			}
		}
		"Dragon Ball" => {
			// Dragon Ball utilise un format webp spécial avec tome fixe
			format!("{}/{}/1/DragonBallFixTome1-{:03}.webp", cdn_url, encoded_title, page)
		}
		_ => {
			// Format par défaut: simple {page}.jpg (fonctionne pour 20th Century Boys, etc.)
			format!("{}/{}/{}/{}.jpg", cdn_url, encoded_title, chapter_index, page)
		}
	}
}

fn manga_key_to_title(manga_key: &str) -> String {
	let manga_slug = manga_key.split('/').last().unwrap_or("Manga");
	
	// Cas spéciaux avec caractères spéciaux et titres exacts du CDN
	match manga_slug {
		"kaiju-n8" => String::from("Kaiju N°8"),
		"one-piece" | "one_piece" => String::from("One Piece"),
		"20th-century-boys" => String::from("20th Century Boys"),
		"21st-century-boys" => String::from("21st Century Boys"),
		"dragon-ball" => String::from("Dragon Ball"),
		"naruto" => String::from("Naruto"),
		"bleach" => String::from("Bleach"),
		"hunter-x-hunter" | "hunter_x_hunter" => String::from("Hunter x Hunter"),
		"a-couple-of-cuckoos" => String::from("A Couple of Cuckoos"),
		"a-sign-of-affection" => String::from("A Sign of Affection"),
		"blue-lock" => String::from("Blue Lock"),
		"chainsaw-man" => String::from("Chainsaw Man"),
		"demon-slayer" => String::from("Demon Slayer"),
		"my-hero-academia" => String::from("My Hero Academia"),
		"attack-on-titan" => String::from("Attack on Titan"),
		"tokyo-ghoul" => String::from("Tokyo Ghoul"),
		"death-note" => String::from("Death Note"),
		"spy-family" => String::from("Spy x Family"),
		"jujutsu-kaisen" => String::from("Jujutsu Kaisen"),
		"boruto" => String::from("Boruto"),
		"black-clover" => String::from("Black Clover"),
		"fire-force" => String::from("Fire Force"),
		"dr-stone" => String::from("Dr. Stone"),
		"versatile-mage" => String::from("Versatile Mage"),
		_ => {
			// Conversion générique améliorée: slug -> Title Case
			manga_slug
				.replace("-", " ")
				.replace("_", " ")
				.split_whitespace()
				.map(|word| {
					// Cas spéciaux pour certains mots
					match word.to_lowercase().as_str() {
						"n" => String::from("N°"), // Pour les numéros
						"dr" => String::from("Dr."),
						"mr" => String::from("Mr."),
						"ms" => String::from("Ms."),
						"x" => String::from("x"), // Pour "Spy x Family"
						"vs" => String::from("vs"),
						"of" | "the" | "and" | "in" | "on" | "at" | "to" | "for" => {
							// Articles et prépositions en minuscules (sauf début)
							word.to_lowercase()
						},
						_ => {
							// Title Case normal
							let mut chars = word.chars();
							match chars.next() {
								None => String::new(),
								Some(first) => first.to_uppercase().chain(chars).collect(),
							}
						}
					}
				})
				.collect::<Vec<String>>()
				.join(" ")
				.replace(" N ", " N°") // Remplacer "N " par "N°"
		}
	}
}


