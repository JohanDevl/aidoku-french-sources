use aidoku::{
	Chapter, ContentRating, Manga, MangaPageResult, MangaStatus, Page, PageContent, Result, 
	Viewer,
	alloc::{String, Vec, format, vec, string::ToString},
	imports::html::Document,
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
				
				mangas.push(Manga {
					key: relative_url.clone(),
					cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
					title,
					authors: None,
					artists: None,
					description: None,
					url: Some(format!("{}{}", BASE_URL, relative_url)),
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
					
					mangas.push(Manga {
						key: relative_url.clone(),
						cover: if !cover_url.is_empty() { Some(cover_url) } else { None },
						title,
						authors: None,
						artists: None,
						description: None,
						url: Some(format!("{}{}", BASE_URL, relative_url)),
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
	
	// Parser la description/synopsis avec plusieurs sélecteurs
	let description = html.select("#sousBlocMiddle p")
		.and_then(|paras| {
			for para in paras {
				if let Some(text) = para.text() {
					let trimmed = text.trim();
					if trimmed.len() > 50 && !trimmed.contains("GENRES") && !trimmed.contains("TYPE") {
						return Some(trimmed.to_string());
					}
				}
			}
			None
		})
		.or_else(|| {
			// Fallback: chercher dans .synopsis, .description
			let selectors = vec![".synopsis", ".description", ".manga-description", "#synopsis p"];
			for selector in selectors {
				if let Some(desc) = html.select(selector).and_then(|els| els.text()) {
					let trimmed = desc.trim();
					if trimmed.len() > 50 {
						return Some(trimmed.to_string());
					}
				}
			}
			None
		})
		.or_else(|| {
			// Dernier fallback: tout paragraphe substantiel
			html.select("p").and_then(|paras| {
				for para in paras {
					if let Some(text) = para.text() {
						let trimmed = text.trim();
						if trimmed.len() > 100 && !trimmed.contains("http") {
							return Some(trimmed.to_string());
						}
					}
				}
				None
			})
		});
	
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
	
	// Parser les genres avec méthodes multiples
	let mut tags = Vec::new();
	
	// Méthode 1: liens dans sousBlocMiddle
	if let Some(genre_links) = html.select("#sousBlocMiddle a") {
		for link in genre_links {
			if let Some(genre_text) = link.text() {
				let text = genre_text.trim();
				if !text.is_empty() && text.len() < 30 && !text.contains("http") {
					tags.push(text.to_string());
				}
			}
		}
	}
	
	// Méthode 2: chercher dans le texte de la page
	if tags.is_empty() {
		if let Some(page_text) = html.select("body").and_then(|els| els.text()) {
			if let Some(genres_pos) = page_text.find("GENRES") {
				let after_genres = &page_text[genres_pos..];
				if let Some(colon_pos) = after_genres.find(':') {
					let genres_section = &after_genres[colon_pos + 1..];
					if let Some(end_pos) = genres_section.find('\n') {
						let genres_line = &genres_section[..end_pos];
						for genre in genres_line.split(',') {
							let trimmed = genre.trim().replace("\"", "").replace("'", "");
							if !trimmed.is_empty() && trimmed.len() < 30 {
								tags.push(trimmed);
							}
						}
					}
				}
			}
		}
	}
	
	// Méthode 3: sélecteurs standards pour genres
	if tags.is_empty() {
		let selectors = vec![".genres a", ".tags a", ".genre", ".tag"];
		for selector in selectors {
			if let Some(genre_elements) = html.select(selector) {
				for element in genre_elements {
					if let Some(text) = element.text() {
						let trimmed = text.trim();
						if !trimmed.is_empty() && trimmed.len() < 30 {
							tags.push(trimmed.to_string());
						}
					}
				}
				if !tags.is_empty() {
					break;
				}
			}
		}
	}
	
	// Parser le statut si possible
	let status = if let Some(status_text) = html.select("#sousBlocMiddle").and_then(|els| els.text()) {
		if status_text.contains("Terminé") || status_text.contains("Fini") {
			MangaStatus::Completed
		} else if status_text.contains("En cours") || status_text.contains("Ongoing") {
			MangaStatus::Ongoing
		} else {
			MangaStatus::Unknown
		}
	} else {
		MangaStatus::Ongoing // Défaut pour AnimeSama
	};
	
	// Si aucun genre trouvé, ajouter des genres par défaut
	if tags.is_empty() {
		tags.push("Manga".to_string());
	}
	
	Ok(Manga {
		key: manga_key.clone(),
		title,
		authors: None,
		artists: None,
		description,
		url: Some(format!("{}{}", BASE_URL, manga_key)),
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
	
	// Essayer de récupérer le contenu JavaScript de la page
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
	
	// Parser les commandes JavaScript pour créer le mapping des chapitres
	let chapter_mappings = parse_chapter_mapping(&html_content);
	
	// Si on a trouvé des mappings, les utiliser pour créer les chapitres
	if !chapter_mappings.is_empty() {
		for mapping in chapter_mappings {
			chapters.push(Chapter {
				key: mapping.index.to_string(),
				title: Some(mapping.title),
				chapter_number: Some(mapping.chapter_number),
				volume_number: None,
				date_uploaded: None,
				scanlators: Some(vec!["AnimeSama".into()]),
				url: Some(build_chapter_url(&manga_key, mapping.index)),
				..Default::default()
			});
		}
	} else {
		// Fallback: chercher des sélecteurs HTML standards
		let selectors = vec![
			".chapter-list .chapter",
			".episodes li", 
			".scan-list li",
			".chapter-item",
			"#containerLEL div"
		];
		
		for selector in selectors {
			if let Some(chapter_elements) = html.select(selector) {
				for (index, chapter_el) in chapter_elements.enumerate() {
					let title = chapter_el.select("a, .chapter-title")
						.and_then(|els| els.text())
						.unwrap_or(format!("Chapitre {}", index + 1));
					
					let chapter_url = chapter_el.select("a")
						.and_then(|els| els.first())
						.and_then(|el| el.attr("href"))
						.unwrap_or_default();
					
					chapters.push(Chapter {
						key: (index + 1).to_string(),
						title: Some(title),
						chapter_number: Some((index + 1) as f32),
						volume_number: None,
						date_uploaded: None,
						scanlators: Some(vec!["AnimeSama".into()]),
						url: Some(if chapter_url.starts_with("http") { 
							chapter_url 
						} else { 
							build_chapter_url(&manga_key, index as i32 + 1)
						}),
						..Default::default()
					});
				}
				break; // Stop dès qu'on trouve des chapitres
			}
		}
		
		// Si toujours pas de chapitres trouvés, créer quelques chapitres par défaut
		if chapters.is_empty() {
			for i in 1i32..=20 {
				chapters.push(Chapter {
					key: i.to_string(),
					title: Some(format!("Chapitre {}", i)),
					chapter_number: Some(i as f32),
					volume_number: None,
					date_uploaded: None,
					scanlators: Some(vec!["AnimeSama".into()]),
					url: Some(build_chapter_url(&manga_key, i)),
					..Default::default()
				});
			}
		}
	}
	
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

// Estimer le nombre de pages d'un chapitre
fn estimate_page_count(_manga_name: &str, _chapter_index: i32) -> i32 {
	// Pour l'instant, retourner un nombre fixe
	// Dans l'implémentation complète, on pourrait faire une requête à l'API AnimeSama
	20
}