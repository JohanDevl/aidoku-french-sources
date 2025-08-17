use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, net::{Request, HttpMethod}, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::{BASE_URL, CDN_URL};
use crate::helper;

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
	let description = html.select("#sousBlocMiddle p.text-sm.text-gray-400").text().read();
	let genre_text = html.select("#sousBlocMiddle a.text-sm.text-gray-300").text().read();
	
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
	
	// Extraire le titre du manga pour les requêtes episodes.js
	let manga_title = html.select("#titreOeuvre").text().read();
	
	if manga_title.is_empty() {
		return Ok(chapters);
	}
	
	// Chercher les scripts qui contiennent panneauScan
	let script_content = html.select("script:contains(panneauScan)").text().read();
	
	if script_content.is_empty() {
		return Ok(chapters);
	}
	
	// Parser les appels panneauScan pour extraire les groupes de scans
	let lines: Vec<&str> = script_content.split(';').collect();
	
	for line in lines {
		if line.contains("panneauScan(") {
			// Extraire le nom et l'URL du groupe de scan
			if let Some(start) = line.find("panneauScan(\"") {
				let line_part = &line[start + 13..];
				if let Some(end_name) = line_part.find("\", \"") {
					let scan_title = &line_part[..end_name];
					let remaining = &line_part[end_name + 4..];
					if let Some(end_url) = remaining.find("\"") {
						let scan_url = &remaining[..end_url];
						
						// Ignorer les URLs contenant "va" (versions animées)
						if !scan_url.contains("va") {
							// Nettoyer le nom du groupe de scan
							let scanlator = String::from(scan_title.replace("Scans", "").replace("(", "").replace(")", "").trim());
							
							// Faire une requête pour obtenir les chapitres de ce groupe
							let full_url = format!("{}{}/{}", String::from(BASE_URL), manga_id, scan_url);
							if let Ok(scan_html) = Request::new(&full_url, HttpMethod::Get).html() {
								let mut scan_chapters = parse_chapters_from_scan_page(scan_html, &manga_title, &scanlator)?;
								chapters.append(&mut scan_chapters);
							}
						}
					}
				}
			}
		}
	}
	
	// Trier les chapitres par ID
	chapters.sort_by(|a, b| {
		let id_a = extract_chapter_id(&a.url);
		let id_b = extract_chapter_id(&b.url);
		id_a.cmp(&id_b)
	});
	
	// Inverser l'ordre pour avoir les derniers chapitres en premier
	chapters.reverse();
	
	Ok(chapters)
}

fn parse_chapters_from_scan_page(html: Node, manga_title: &str, scanlator: &str) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Construire l'URL pour episodes.js
	let episodes_url = format!("{}/episodes.js?title={}", String::from(BASE_URL), helper::urlencode(manga_title));
	
	// Récupérer le contenu JavaScript
	if let Ok(js_response) = Request::new(&episodes_url, HttpMethod::Get).string() {
		// Parser les épisodes depuis le JavaScript
		let mut episode_numbers: Vec<i32> = Vec::new();
		
		// Chercher tous les "eps" suivis d'un nombre
		let mut pos = 0;
		while let Some(eps_pos) = js_response[pos..].find("eps") {
			pos += eps_pos + 3;
			let remaining = &js_response[pos..];
			let mut number_str = String::new();
			
			for ch in remaining.chars() {
				if ch.is_ascii_digit() {
					number_str.push(ch);
				} else {
					break;
				}
			}
			
			if !number_str.is_empty() {
				if let Ok(ep_num) = number_str.parse::<i32>() {
					if !episode_numbers.contains(&ep_num) {
						episode_numbers.push(ep_num);
					}
				}
			}
		}
		
		// Trier et inverser pour avoir les épisodes dans l'ordre décroissant
		episode_numbers.sort();
		episode_numbers.reverse();
		
		// Parser les commandes JavaScript depuis la page
		let script_content = html.select("script:contains(resetListe())").text().read();
		let script_commands: Vec<&str> = script_content.split(';').collect();
		
		let mut chapter_count = 0;
		
		for command in script_commands {
			if command.contains("creerListe(") {
				// Parser creerListe(start, end)
				if let Some(start_idx) = command.find("creerListe(") {
					let params_str = &command[start_idx + 11..];
					if let Some(end_idx) = params_str.find(")") {
						let params = &params_str[..end_idx];
						let parts: Vec<&str> = params.split(',').collect();
						if parts.len() == 2 {
							if let (Ok(start), Ok(end)) = (parts[0].trim().parse::<i32>(), parts[1].trim().parse::<i32>()) {
								for i in start..=end {
									chapter_count += 1;
									let chapter_url = format!("{}/episodes.js?title={}&id={}", String::from(BASE_URL), helper::urlencode(manga_title), chapter_count);
									chapters.push(Chapter {
										id: format!("{}", chapter_count),
										title: format!("Chapitre {}", i),
										volume: -1.0,
										chapter: i as f32,
										date_updated: current_date(),
										scanlator: String::from(scanlator),
										url: chapter_url,
										lang: String::from("fr")
									});
								}
							}
						}
					}
				}
			} else if command.contains("newSP(") {
				// Parser newSP(title) pour les chapitres spéciaux
				if let Some(start_idx) = command.find("newSP(") {
					let params_str = &command[start_idx + 6..];
					if let Some(end_idx) = params_str.find(")") {
						let param = &params_str[..end_idx];
						let title = param.trim_matches('"').trim();
						chapter_count += 1;
						let chapter_url = format!("{}/episodes.js?title={}&id={}", String::from(BASE_URL), helper::urlencode(manga_title), chapter_count);
						chapters.push(Chapter {
							id: format!("{}", chapter_count),
							title: format!("Chapitre {}", title),
							volume: -1.0,
							chapter: -1.0,
							date_updated: current_date(),
							scanlator: String::from(scanlator),
							url: chapter_url,
							lang: String::from("fr")
						});
					}
				}
			}
		}
		
		// Ajouter les chapitres restants basés sur le nombre d'épisodes trouvés
		for _index in chapter_count as usize..episode_numbers.len() {
			chapter_count += 1;
			let chapter_url = format!("{}/episodes.js?title={}&id={}", String::from(BASE_URL), helper::urlencode(manga_title), chapter_count);
			chapters.push(Chapter {
				id: format!("{}", chapter_count),
				title: format!("Chapitre {}", chapter_count),
				volume: -1.0,
				chapter: chapter_count as f32,
				date_updated: current_date(),
				scanlator: String::from(scanlator),
				url: chapter_url,
				lang: String::from("fr")
			});
		}
	}
	
	Ok(chapters)
}

fn extract_chapter_id(url: &str) -> i32 {
	if let Some(id_start) = url.find("&id=") {
		let id_str = &url[id_start + 4..];
		if let Some(id_end) = id_str.find('&') {
			id_str[..id_end].parse().unwrap_or(0)
		} else {
			id_str.parse().unwrap_or(0)
		}
	} else {
		0
	}
}

pub fn parse_page_list(_html: Node, _manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// Le chapter_id est en fait l'URL vers episodes.js avec les paramètres
	// Extraire le titre et l'ID du chapitre depuis l'URL
	let mut title = String::new();
	let mut chapter_num = String::new();
	
	// Parser l'URL pour extraire title et id
	if let Some(title_start) = chapter_id.find("title=") {
		let title_part = &chapter_id[title_start + 6..];
		if let Some(title_end) = title_part.find("&") {
			title = helper::urldecode(&title_part[..title_end]);
		} else {
			title = helper::urldecode(title_part);
		}
	}
	
	if let Some(id_start) = chapter_id.find("id=") {
		let id_part = &chapter_id[id_start + 3..];
		if let Some(id_end) = id_part.find("&") {
			chapter_num = String::from(&id_part[..id_end]);
		} else {
			chapter_num = String::from(id_part);
		}
	}
	
	if title.is_empty() || chapter_num.is_empty() {
		return Ok(pages);
	}
	
	// Faire une requête vers episodes.js pour obtenir le contenu JavaScript
	if let Ok(js_content) = Request::new(&chapter_id, HttpMethod::Get).string() {
		// Chercher la définition du chapitre spécifique (eps{chapter_num})
		let eps_pattern = format!("eps{}", chapter_num);
		
		// Trouver la définition de ce chapitre
		if let Some(eps_start) = js_content.find(&eps_pattern) {
			let remaining = &js_content[eps_start..];
			
			// Chercher le nombre de pages pour ce chapitre
			let mut page_count = 0;
			
			// Cas 1: eps{n} = [array] - compter les éléments
			if let Some(array_start) = remaining.find("[") {
				if let Some(array_end) = remaining.find("]") {
					if array_start < array_end {
						let array_content = &remaining[array_start + 1..array_end];
						// Compter les éléments séparés par des virgules
						if !array_content.trim().is_empty() {
							page_count = array_content.split(',').count();
						}
					}
				}
			}
			
			// Cas 2: eps{n}.length = {number}
			if page_count == 0 {
				if let Some(length_start) = remaining.find(".length") {
					let length_part = &remaining[length_start + 7..];
					if let Some(eq_pos) = length_part.find("=") {
						let number_part = &length_part[eq_pos + 1..];
						let mut number_str = String::new();
						
						for ch in number_part.trim().chars() {
							if ch.is_ascii_digit() {
								number_str.push(ch);
							} else {
								break;
							}
						}
						
						if let Ok(count) = number_str.parse::<i32>() {
							page_count = count as usize;
						}
					}
				}
			}
			
			// Générer les pages
			for page_index in 1..=page_count {
				let image_url = format!("{}{}/{}/{}.jpg", String::from(CDN_URL), title, chapter_num, page_index);
				pages.push(Page {
					index: page_index as i32,
					url: image_url.clone(),
					base64: String::new(),
					text: String::new()
				});
			}
		}
	}
	
	Ok(pages)
}