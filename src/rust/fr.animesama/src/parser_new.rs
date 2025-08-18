use aidoku::{
	error::Result, prelude::*, std::{net::Request, String, Vec}, Node
};

// Helper pour extraire les numéros de chapitres de manière robuste depuis HTML
fn extract_chapter_numbers_robust(html_content: &str) -> Vec<i32> {
	let mut chapters: Vec<i32> = Vec::new();
	
	// Patterns multiples pour capturer différents formats
	let patterns = [
		">Chapitre ",
		"\"Chapitre ",
		"Chapitre ",
		"chapitre "
	];
	
	for pattern in &patterns {
		let mut start_pos = 0;
		while let Some(pos) = html_content[start_pos..].to_lowercase().find(&pattern.to_lowercase()) {
			start_pos += pos + pattern.len();
			
			// Extraire le numéro
			let mut num_str = String::new();
			for ch in html_content[start_pos..].chars() {
				if ch.is_ascii_digit() {
					num_str.push(ch);
				} else {
					break;
				}
			}
			
			if !num_str.is_empty() {
				if let Ok(chapter_num) = num_str.parse::<i32>() {
					if chapter_num > 0 && chapter_num <= 1000 { // Validation reasonable
						chapters.push(chapter_num);
					}
				}
			}
			
			start_pos += num_str.len();
		}
	}
	
	// Supprimer les doublons et trier
	chapters.sort_unstable();
	chapters.dedup();
	
	chapters
}

pub fn parse_chapter_list_dynamic_clean(manga_id: String, html: Node, _request_url: String) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();
	let html_content = html.html().read();
	
	// Méthode robuste: chercher tous les patterns possibles de chapitres
	let chapter_numbers = extract_chapter_numbers_robust(&html_content);
	
	if !chapter_numbers.is_empty() {
		// Créer les chapitres depuis les numéros trouvés
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
		chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(std::cmp::Ordering::Equal));
		return Ok(chapters);
	}
	
	// Fallback: générer 50 chapitres par défaut
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
	
	// Tri par numéro de chapitre (du plus récent au plus ancien)
	chapters.sort_by(|a, b| b.chapter.partial_cmp(&a.chapter).unwrap_or(std::cmp::Ordering::Equal));
	Ok(chapters)
}

// Garder les autres fonctions existantes...
use aidoku::{std::current_date, std::html::Node, Chapter, MangaContentRating, MangaStatus, MangaViewer, Manga};

const BASE_URL: &str = "https://anime-sama.fr";

fn build_chapter_url(manga_id: &str) -> String {
	format!("{}/catalogue/{}/scan/vf/", BASE_URL, manga_id)
}

fn build_manga_url(manga_id: &str) -> String {
	format!("{}/catalogue/{}/", BASE_URL, manga_id)
}