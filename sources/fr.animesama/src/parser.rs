use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::html::Document,
	imports::net::Request,
};

use crate::{BASE_URL, CDN_URL};

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
	
	// Extraire les genres - avec filtrage strict pour éliminer les faux genres
	let mut tags: Vec<String> = Vec::new();
	
	// Fonction pour vérifier si un texte est un vrai genre
	let is_valid_genre = |text: &str| -> bool {
		let lower = text.to_lowercase();
		// Exclure les textes qui ne sont pas des genres
		if lower.contains("episode") || lower.contains("chapitre") || lower.contains("->") 
			|| lower.contains("saison") || lower.contains("scan") || lower.contains("vf")
			|| text.len() > 30 || text.chars().any(|c| c.is_ascii_digit())
			|| lower.contains("lire") || lower.contains("manga") || lower.contains("anime") {
			false
		} else {
			// Doit être un mot simple ou deux mots maximum
			text.split_whitespace().count() <= 2 && text.len() >= 3 && text.len() <= 20
		}
	};
	
	// Chercher spécifiquement après h2 contenant "Genres"
	if let Some(h2_elements) = html.select("#sousBlocMiddle h2") {
		for h2 in h2_elements {
			if let Some(h2_text) = h2.text() {
				if h2_text.to_lowercase().contains("genres") {
					// Chercher les liens suivant ce h2 spécifique
					if let Some(parent) = h2.parent() {
						if let Some(genre_links) = parent.select("a") {
							for link in genre_links {
								if let Some(genre_text) = link.text() {
									let genre_raw = genre_text.trim();
									if !genre_raw.is_empty() {
										// Vérifier si c'est un vrai genre
										if is_valid_genre(genre_raw) {
											// Vérifier si ce genre contient des virgules
											if genre_raw.contains(',') {
												// Diviser par les virgules
												for genre in genre_raw.split(',') {
													let cleaned_genre = genre.trim();
													if !cleaned_genre.is_empty() && is_valid_genre(cleaned_genre) {
														tags.push(cleaned_genre.to_string());
													}
												}
											} else {
												// Genre unique valide
												tags.push(genre_raw.to_string());
											}
										}
									}
								}
							}
						}
					}
					break; // Sortir une fois qu'on a trouvé la section Genres
				}
			}
		}
	}
	
	// Fallback: chercher dans le texte complet si pas de liens trouvés
	if tags.is_empty() {
		if let Some(full_text) = html.select("#sousBlocMiddle").and_then(|els| els.text()) {
			if let Some(genres_start) = full_text.find("GENRES") {
				let genres_section = &full_text[genres_start..];
				if let Some(first_line_end) = genres_section.find('\n') {
					let genres_line = &genres_section[7..first_line_end].trim();
					
					for genre in genres_line.split(',') {
						let cleaned_genre = genre.trim();
						if !cleaned_genre.is_empty() && is_valid_genre(cleaned_genre) {
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
	let html_content = html.select("script").and_then(|scripts| {
		for script in scripts {
			if let Some(script_text) = script.text() {
				if script_text.contains("creerListe") || script_text.contains("newSP") {
					return Some(script_text);
				}
			}
		}
		None
	}).unwrap_or_default();
	
	// Parse JavaScript commands to create chapter mappings
	let chapter_mappings = parse_chapter_mapping(&html_content);
	
	// Get total chapters from API
	let total_chapters = match get_total_chapters_from_api(&manga_name) {
		Ok(count) => count,
		Err(_) => {
			// Fallback: use mappings or reasonable default
			if !chapter_mappings.is_empty() {
				chapter_mappings.iter().map(|m| m.index).max().unwrap_or(100) + 50
			} else {
				200 // Reasonable default for most manga
			}
		}
	};
	
	// Create chapters from 1 to total, using JavaScript mappings when available
	for index in 1..=total_chapters {
		if let Some(mapping) = chapter_mappings.iter().find(|m| m.index == index) {
			// Use JavaScript mapping
			chapters.push(Chapter {
				key: mapping.index.to_string(),
				title: Some(mapping.title.clone()),
				chapter_number: Some(mapping.chapter_number),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec![]), // Vide comme dans l'ancienne version
				url: Some(build_chapter_url(&manga_key, mapping.index)),
				..Default::default()
			});
		} else {
			// No mapping, use normal numbering
			let chapter_number = calculate_chapter_number_for_index(index, &chapter_mappings);
			
			chapters.push(Chapter {
				key: index.to_string(),
				title: Some(format!("Chapitre {}", chapter_number as i32)),
				chapter_number: Some(chapter_number),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec![]), // Vide comme dans l'ancienne version
				url: Some(build_chapter_url(&manga_key, index)),
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
	
	// Récupérer le contenu JavaScript pour chercher les variables eps
	let html_content = html.select("script").and_then(|scripts| {
		for script in scripts {
			if let Some(script_text) = script.text() {
				if script_text.contains("eps") && script_text.contains(&chapter_key) {
					return Some(script_text);
				}
			}
		}
		None
	}).unwrap_or_default();
	
	// Méthode 1: Chercher les variables d'épisode JavaScript (ex: eps1, eps2)
	let episode_pattern = format!("eps{}", chapter_key);
	if let Some(episode_start) = html_content.find(&episode_pattern) {
		// Trouver la fin de la déclaration de variable
		if let Some(episode_section) = html_content[episode_start..].find('[') {
			let array_start = episode_start + episode_section;
			if let Some(array_end) = html_content[array_start..].find("];") {
				let array_content = &html_content[array_start + 1..array_start + array_end];
				
				// Parser les URLs dans le tableau JavaScript
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
							pages.push(Page {
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
	
	// Méthode 2: Utiliser le CDN AnimeSama si pas d'épisodes JavaScript trouvés
	if pages.is_empty() {
		// Extraire et encoder le nom du manga
		let manga_title = extract_manga_title_for_cdn(&manga_key);
		let chapter_index = chapter_key.parse::<i32>().unwrap_or(1);
		
		// Essayer d'obtenir le nombre de pages depuis une API ou estimer
		let page_count = estimate_page_count(&manga_title, chapter_index);
		
		// Générer les URLs des pages
		for i in 1..=page_count {
			let page_url = format!("{}/{}/{}/{:03}.jpg", CDN_URL, manga_title, chapter_index, i);
			pages.push(Page {
				content: PageContent::url(page_url),
				thumbnail: None,
				has_description: false,
				description: None,
			});
		}
	}
	
	// Fallback: au moins quelques pages par défaut
	if pages.is_empty() {
		let manga_title = extract_manga_title_for_cdn(&manga_key);
		let chapter_num = chapter_key.parse::<i32>().unwrap_or(1);
		
		for i in 1i32..=20 {
			let page_url = format!("{}/{}/{}/{:03}.jpg", CDN_URL, manga_title, chapter_num, i);
			pages.push(Page {
				content: PageContent::url(page_url),
				thumbnail: None,
				has_description: false,
				description: None,
			});
		}
	}
	
	Ok(pages)
}

// Structure pour stocker le mapping indice -> numéro de chapitre
#[derive(Debug, Clone)]
struct ChapterMapping {
	index: i32,
	chapter_number: f32,
	title: String,
}

// Parser les commandes JavaScript pour créer le mapping des chapitres
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

// Construire l'URL d'un chapitre avec gestion du cas spécial One Piece
fn build_chapter_url(manga_key: &str, chapter_index: i32) -> String {
	let is_one_piece = manga_key.contains("one-piece") || manga_key.contains("one_piece");
	let scan_path = if is_one_piece { "/scan_noir-et-blanc/vf/" } else { "/scan/vf/" };
	
	if manga_key.starts_with("http") {
		format!("{}{}{}", manga_key, scan_path, chapter_index)
	} else {
		format!("{}{}{}{}", BASE_URL, manga_key, scan_path, chapter_index)
	}
}

// Extraire le titre du manga depuis le HTML pour construire les URLs CDN
fn extract_manga_title_for_cdn(manga_key: &str) -> String {
	// Version simplifiée : utiliser le dernier segment du manga_key
	let title = manga_key.split('/').last().unwrap_or("manga");
	
	// Nettoyer et formatter le titre pour les URLs CDN
	title.replace("-", "_").to_lowercase()
}

// Convertir manga_key en titre formaté
fn manga_key_to_title(manga_key: &str) -> String {
	manga_key.split('/').last().unwrap_or("Manga")
		.replace("-", " ")
		.split_whitespace()
		.map(|word| {
			let mut chars = word.chars();
			match chars.next() {
				None => String::new(),
				Some(first) => first.to_uppercase().chain(chars).collect(),
			}
		})
		.collect::<Vec<String>>()
		.join(" ")
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

// Calculate chapter number for an index, considering special chapters
fn calculate_chapter_number_for_index(index: i32, chapter_mappings: &[ChapterMapping]) -> f32 {
	// If there's a mapping for this index, use it
	if let Some(mapping) = chapter_mappings.iter().find(|m| m.index == index) {
		return mapping.chapter_number;
	}
	
	// Otherwise, calculate based on previous mappings
	let mut chapter_number = index as f32;
	
	// Find the last mapping before this index
	let mut last_mapping_index = 0;
	let mut last_chapter_number = 0.0;
	
	for mapping in chapter_mappings {
		if mapping.index < index && mapping.index > last_mapping_index {
			last_mapping_index = mapping.index;
			last_chapter_number = mapping.chapter_number;
		}
	}
	
	if last_mapping_index > 0 {
		// Calculate offset from last known mapping
		let offset = index - last_mapping_index;
		chapter_number = last_chapter_number + offset as f32;
	}
	
	chapter_number
}

// Get page count from AnimeSama API
fn get_page_count_from_api(manga_name: &str, chapter_index: i32) -> Result<i32> {
	use crate::helper::urlencode;
	
	let api_url = format!("https://anime-sama.fr/s2/scans/get_nb_chap_et_img.php?oeuvre={}", 
		urlencode(manga_name));
	
	match Request::get(&api_url)?.string() {
		Ok(response_text) => {
			// Parse JSON response to find page count for specific chapter
			let chapter_key = format!("\"{}\"", chapter_index);
			
			if let Some(chapter_start) = response_text.find(&chapter_key) {
				// Find the value after the chapter key
				if let Some(colon_pos) = response_text[chapter_start..].find(":") {
					let after_colon = chapter_start + colon_pos + 1;
					
					// Skip whitespace and quotes
					let mut value_start = after_colon;
					while value_start < response_text.len() {
						let ch = response_text.chars().nth(value_start).unwrap_or(' ');
						if ch != ' ' && ch != '\"' && ch != '\t' {
							break;
						}
						value_start += 1;
					}
					
					// Find end of value
					let mut value_end = value_start;
					while value_end < response_text.len() {
						let ch = response_text.chars().nth(value_end).unwrap_or(',');
						if ch == ',' || ch == '}' || ch == '\"' {
							break;
						}
						value_end += 1;
					}
					
					if let Ok(page_count) = response_text[value_start..value_end].trim().parse::<i32>() {
						if page_count > 0 {
							return Ok(page_count);
						}
					}
				}
			}
			
			// Fallback: return reasonable default
			Ok(20)
		},
		Err(e) => Err(e)
	}
}

// Estimer le nombre de pages d'un chapitre avec API call
fn estimate_page_count(manga_name: &str, chapter_index: i32) -> i32 {
	match get_page_count_from_api(manga_name, chapter_index) {
		Ok(count) => count,
		Err(_) => 20 // Fallback default
	}
}