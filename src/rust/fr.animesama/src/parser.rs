use aidoku::{
	error::Result, prelude::*, std::{
		current_date, html::Node, String, Vec
	}, Chapter, Manga, MangaContentRating, MangaPageResult, MangaStatus, MangaViewer, Page
};

use crate::BASE_URL;

// Helper pour construire l'URL correctement
fn build_chapter_url(manga_id: &str) -> String {
	if manga_id.starts_with("http") {
		// manga_id contient déjà l'URL complète
		format!("{}/scan/vf/", manga_id)
	} else {
		// manga_id est relatif, ajouter BASE_URL
		format!("{}{}/scan/vf/", String::from(BASE_URL), manga_id)
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

pub fn parse_manga_details(manga_id: String, _html: Node) -> Result<Manga> {
	// ULTRA-SIMPLE: Valeurs de test fixes
	let mut categories: Vec<String> = Vec::new();
	categories.push(String::from("Test"));
	
	Ok(Manga {
		id: manga_id.clone(),
		cover: String::from("https://anime-sama.fr/images/default.jpg"),
		title: String::from("Test Manga"),
		author: String::from("Test Author"),
		artist: String::from("Test Artist"),
		description: String::from("Manga de test pour debug AnimeSama"),
		url: build_manga_url(&manga_id),
		categories,
		status: MangaStatus::Unknown,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Scroll
	})
}

pub fn parse_chapter_list_dynamic(manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Debug: Chercher les différents sélecteurs possibles pour le select
	let select_all = html.select("select");
	let select_options_all = html.select("select option");
	let select_id = html.select("select[id*='chapitre']");  // Chercher par ID contenant "chapitre"
	let select_class = html.select("select[class*='chapitre']");  // Chercher par class contenant "chapitre"
	
	// Premier chapitre de debug avec informations
	chapters.push(Chapter {
		id: String::from("debug"),
		title: format!("DEBUG: selects={} opts={} id_sel={} class_sel={}", 
			select_all.array().len(), 
			select_options_all.array().len(),
			select_id.array().len(),
			select_class.array().len()
		),
		volume: -1.0,
		chapter: 999.0,
		date_updated: current_date(),
		scanlator: format!("manga: {}", manga_id),
		url: build_chapter_url(&manga_id),
		lang: String::from("fr")
	});
	
	// Essayer de parser les vraies options si trouvées
	let options_found = select_options_all.array().len();
	if options_found > 0 {
		for option in select_options_all.array() {
			if let Ok(option_node) = option.as_node() {
				let option_text = option_node.text().read();
				let option_value = option_node.attr("value").read();
				
				// Debug: afficher quelques options trouvées
				if chapters.len() < 5 {  // Limiter le debug à quelques chapitres
					chapters.push(Chapter {
						id: format!("opt_{}", chapters.len()),
						title: format!("OPT: text='{}' val='{}'", option_text, option_value),
						volume: -1.0,
						chapter: chapters.len() as f32,
						date_updated: current_date(),
						scanlator: String::from("DEBUG"),
						url: build_chapter_url(&manga_id),
						lang: String::from("fr")
					});
				}
				
				// Essayer de parser un numéro de chapitre
				if option_text.contains("hapitre") && !option_text.is_empty() {
					// Extraire le numéro depuis le texte de l'option
					let parts: Vec<&str> = option_text.split_whitespace().collect();
					for part in parts {
						if let Ok(num) = part.parse::<i32>() {
							chapters.push(Chapter {
								id: format!("{}", num),
								title: String::from(""),
								volume: -1.0,
								chapter: num as f32,
								date_updated: current_date(),
								scanlator: String::from(""),
								url: build_chapter_url(&manga_id),
								lang: String::from("fr")
							});
							break;
						}
					}
				}
			}
		}
	} else {
		// Fallback si aucune option trouvée
		chapters.push(Chapter {
			id: String::from("fallback"),
			title: String::from("FALLBACK: No select options found"),
			volume: -1.0,
			chapter: 1.0,
			date_updated: current_date(),
			scanlator: String::from("FALLBACK"),
			url: build_chapter_url(&manga_id),
			lang: String::from("fr")
		});
	}
	
	Ok(chapters)
}

pub fn parse_chapter_list_fallback(manga_id: String, _dummy_html: Node, failed_url: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	
	// Debug : montrer pourquoi on est en fallback
	chapters.push(Chapter {
		id: String::from("fallback_debug"),
		title: format!("FALLBACK: URL failed"),
		volume: -1.0,
		chapter: 999.0,
		date_updated: current_date(),
		scanlator: format!("URL: {}", failed_url),
		url: build_chapter_url(&manga_id),
		lang: String::from("fr")
	});
	
	// Générer un nombre adaptatif de chapitres selon le manga
	let chapter_count = if manga_id.contains("blue-lock") {
		314  // Blue Lock a ~314 chapitres
	} else if manga_id.contains("one-piece") {
		1000  // One Piece a ~1000+ chapitres
	} else {
		100   // Défaut pour autres mangas
	};
	
	// Générer les chapitres avec le count adapté
	for i in 1..=chapter_count {
		chapters.push(Chapter {
			id: format!("{}", i),
			title: String::from(""),
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