#![no_std]

extern crate alloc;
use aidoku_stable::prelude::*;
use alloc::{format, boxed::Box};

// Modules contenant la logique de parsing sophistiquée d'AnimeSama
// Ces modules sont conservés pour quand le wrapper aidoku-stable supportera les requêtes HTTP
pub mod parser;
pub mod helper;

pub static BASE_URL: &str = "https://anime-sama.fr";
pub static CDN_URL: &str = "https://anime-sama.fr/s2/scans";

#[no_mangle]
pub extern "C" fn get_manga_list() -> *const u8 {
	// Test avec des faux mangas pour vérifier que l'interface fonctionne
	let test_manga_json = format!(r#"{{
		"manga": [
			{{
				"id": "/catalogue/one-piece",
				"title": "One Piece",
				"cover": "https://anime-sama.fr/images/one-piece.jpg",
				"url": "{}/catalogue/one-piece"
			}},
			{{
				"id": "/catalogue/naruto",
				"title": "Naruto",
				"cover": "https://anime-sama.fr/images/naruto.jpg",
				"url": "{}/catalogue/naruto"
			}},
			{{
				"id": "/catalogue/dragon-ball",
				"title": "Dragon Ball",
				"cover": "https://anime-sama.fr/images/dragon-ball.jpg",
				"url": "{}/catalogue/dragon-ball"
			}}
		],
		"has_more": false
	}}"#, BASE_URL, BASE_URL, BASE_URL);
	
	let boxed = Box::new(test_manga_json.into_bytes());
	Box::into_raw(boxed) as *const u8
}

#[no_mangle]
pub extern "C" fn get_manga_details() -> *const u8 {
	// Test avec des détails de manga pour vérifier que l'interface fonctionne
	let test_details_json = format!(r#"{{
		"id": "/catalogue/one-piece",
		"title": "One Piece",
		"author": "Eiichiro Oda",
		"artist": "Eiichiro Oda",
		"description": "L'histoire de One Piece suit les aventures de Monkey D. Luffy, un jeune homme dont le corps a acquis les propriétés du caoutchouc après avoir mangé un fruit du démon. Avec son équipage de pirates, il explore Grand Line à la recherche du trésor ultime connu sous le nom de 'One Piece'.",
		"url": "{}/catalogue/one-piece",
		"cover": "https://anime-sama.fr/images/one-piece-cover.jpg",
		"categories": ["Action", "Aventure", "Comédie", "Shonen"],
		"status": 1,
		"nsfw": 0,
		"viewer": 0
	}}"#, BASE_URL);
	
	let boxed = Box::new(test_details_json.into_bytes());
	Box::into_raw(boxed) as *const u8
}

#[no_mangle]
pub extern "C" fn get_chapter_list() -> *const u8 {
	// Test avec des chapitres pour vérifier que l'interface fonctionne
	let test_chapters_json = format!(r#"[
		{{
			"id": "1",
			"title": "Romance Dawn",
			"volume": 1.0,
			"chapter": 1.0,
			"date_updated": 1640995200.0,
			"scanlator": "AnimeSama",
			"url": "{}/catalogue/one-piece/scan/vf/1",
			"lang": "fr"
		}},
		{{
			"id": "2", 
			"title": "L'homme au chapeau de paille",
			"volume": 1.0,
			"chapter": 2.0,
			"date_updated": 1640995200.0,
			"scanlator": "AnimeSama",
			"url": "{}/catalogue/one-piece/scan/vf/2",
			"lang": "fr"
		}},
		{{
			"id": "3",
			"title": "Entrez : Pirate Hunter Roronoa Zoro",
			"volume": 1.0,
			"chapter": 3.0,
			"date_updated": 1640995200.0,
			"scanlator": "AnimeSama", 
			"url": "{}/catalogue/one-piece/scan/vf/3",
			"lang": "fr"
		}}
	]"#, BASE_URL, BASE_URL, BASE_URL);
	
	let boxed = Box::new(test_chapters_json.into_bytes());
	Box::into_raw(boxed) as *const u8
}

#[no_mangle]
pub extern "C" fn get_page_list() -> *const u8 {
	// Test avec des pages pour vérifier que l'interface fonctionne
	let test_pages_json = format!(r#"[
		{{
			"content": {{
				"Url": "{}/s2/scans/one_piece/1/001.jpg"
			}},
			"has_description": false,
			"description": null
		}},
		{{
			"content": {{
				"Url": "{}/s2/scans/one_piece/1/002.jpg"
			}},
			"has_description": false,
			"description": null
		}},
		{{
			"content": {{
				"Url": "{}/s2/scans/one_piece/1/003.jpg"
			}},
			"has_description": false,
			"description": null
		}},
		{{
			"content": {{
				"Url": "{}/s2/scans/one_piece/1/004.jpg"
			}},
			"has_description": false,
			"description": null
		}},
		{{
			"content": {{
				"Url": "{}/s2/scans/one_piece/1/005.jpg"
			}},
			"has_description": false,
			"description": null
		}}
	]"#, CDN_URL, CDN_URL, CDN_URL, CDN_URL, CDN_URL);
	
	let boxed = Box::new(test_pages_json.into_bytes());
	Box::into_raw(boxed) as *const u8
}