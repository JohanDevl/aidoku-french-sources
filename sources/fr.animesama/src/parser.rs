use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::html::Document,
	imports::net::Request,
};

use crate::{BASE_URL, CDN_URL, helper};

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
	
	// Récupérer le contenu JavaScript complet pour chercher les variables eps
	let mut html_content = String::new();
	if let Some(scripts) = html.select("script") {
		for script in scripts {
			if let Some(script_text) = script.text() {
				html_content.push_str(&script_text);
				html_content.push('\n');
			}
		}
	}
	
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
	
	// Si toujours vide, utiliser les méthodes de l'ancienne version
	if pages.is_empty() {
		let chapter_index = chapter_key.parse::<i32>().unwrap_or(1);
		
		// Extraire le nom du manga depuis l'ID (ex: /catalogue/blue-lock -> blue-lock)
		let manga_slug = manga_key.split('/').last().unwrap_or("manga");
		
		// Extraire le titre du manga depuis le HTML pour construire les URLs CDN
		let mut manga_title = String::new();
		
		// Méthode 1: Chercher #titreOeuvre (page principale manga)
		if let Some(title_elem) = html.select("#titreOeuvre").and_then(|els| els.first()) {
			if let Some(title_text) = title_elem.text() {
				if !title_text.trim().is_empty() {
					manga_title = title_text.trim().to_string();
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
						manga_title = page_title.split(" - ").next().unwrap_or("").trim().to_string();
					}
				}
			}
		}
		
		// Méthode 3: Chercher dans les éléments h1 qui peuvent contenir le titre
		if manga_title.is_empty() {
			if let Some(h1_elem) = html.select("h1").and_then(|els| els.first()) {
				if let Some(h1_text) = h1_elem.text() {
					if !h1_text.trim().is_empty() {
						manga_title = h1_text.trim().to_string();
					}
				}
			}
		}
		
		// Fallback final: convertir le slug en titre avec gestion des cas spéciaux
		if manga_title.is_empty() {
			manga_title = match manga_slug {
				"kaiju-n8" => "Kaiju N°8".to_string(), // Cas spécial avec symbole degré
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
		let page_count = parse_episodes_js_from_html(&html_content, chapter_index);
		
		if page_count > 0 {
			// Succès avec le parsing JavaScript - utiliser l'indice API dans l'URL
			for i in 1..=page_count {
				let page_url = format!("{}/{}/{}/{}.jpg", 
					CDN_URL, 
					helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
					chapter_index, 
					i
				);
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
					let page_url = format!("{}/{}/{}/{}.jpg", 
						CDN_URL, 
						helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
						chapter_index, 
						i
					);
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
				// L'API a aussi échoué
			}
		}
		
		// PRIORITÉ 3 : Fallback ultime - 20 pages par défaut
		for i in 1..=20 {
			let page_url = format!("{}/{}/{}/{}.jpg", 
				CDN_URL, 
				helper::urlencode(&manga_title), // URL encode complètement (espaces + caractères spéciaux)
				chapter_index, 
				i
			);
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
	
	// Si le chapitre n'est pas trouvé dans l'API, retourner une erreur simple
	// Utiliser une façon différente de créer l'erreur
	let utf8_err: core::str::Utf8Error = match core::str::from_utf8(&[0xFF]) {
		Err(e) => e,
		Ok(_) => unreachable!(),
	};
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
		format!("{}{}?id={}", manga_key, scan_path, chapter_index)
	} else {
		format!("{}{}{}?id={}", BASE_URL, manga_key, scan_path, chapter_index)
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


// Estimer le nombre de pages d'un chapitre avec API call
fn estimate_page_count(manga_name: &str, chapter_index: i32) -> i32 {
	match get_page_count_from_api(manga_name, chapter_index) {
		Ok(count) => count,
		Err(_) => 20 // Fallback default
	}
}