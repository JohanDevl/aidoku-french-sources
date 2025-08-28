#![no_std]

extern crate alloc;
use aidoku_stable::prelude::*;
use alloc::{string::String, vec::Vec, format};

pub mod parser;
pub mod helper;

pub static BASE_URL: &str = "https://anime-sama.fr";
pub static CDN_URL: &str = "https://anime-sama.fr/s2/scans";

#[no_mangle]
pub extern "C" fn get_manga_list() -> *const u8 {
	// Pour l'instant, retourner la page d'accueil avec les dernières sorties
	let url = format!("{}", BASE_URL);
	
	// Faire la requête HTTP (placeholder car le wrapper ne supporte pas encore HTTP)
	// TODO: Implémenter l'appel HTTP réel quand le wrapper le supportera
	let mock_html = format!(r#"
		<div id="containerAjoutsScans">
			<div><h1>One Piece</h1><a href="/catalogue/one-piece"><img src="/images/one-piece.jpg"></a></div>
			<div><h1>Naruto</h1><a href="/catalogue/naruto"><img src="/images/naruto.jpg"></a></div>
			<div><h1>Dragon Ball</h1><a href="/catalogue/dragon-ball"><img src="/images/dragon-ball.jpg"></a></div>
		</div>
	"#);
	
	let html_node = Node::new(&mock_html);
	
	match parser::parse_manga_listing(html_node, "Dernières Sorties") {
		Ok(manga_result) => {
			// Convertir le résultat en JSON et le retourner
			// Pour l'instant, retourner une chaîne simple
			let result_json = format!(r#"{{"manga":[],"has_more":false}}"#);
			let boxed = alloc::boxed::Box::new(result_json.into_bytes());
			alloc::boxed::Box::into_raw(boxed) as *const u8
		}
		Err(_) => core::ptr::null()
	}
}

#[no_mangle]
pub extern "C" fn get_manga_details() -> *const u8 {
	// Cette fonction devrait prendre un manga_id en paramètre
	// Pour l'instant, créer un exemple avec un manga par défaut
	let manga_id = String::from("/catalogue/one-piece");
	
	// Créer un HTML mock pour les détails du manga
	let mock_html = format!(r#"
		<div id="titreOeuvre">One Piece</div>
		<img id="coverOeuvre" src="/images/one-piece-cover.jpg">
		<div id="sousBlocMiddle">
			<h2>Synopsis</h2>
			<p>L'histoire de One Piece suit les aventures de Monkey D. Luffy, un jeune homme dont le corps a acquis les propriétés du caoutchouc après avoir mangé un fruit du démon.</p>
			<h2>Genres</h2>
			<a>Action</a><a>Aventure</a><a>Comédie</a>
		</div>
	"#);
	
	let html_node = Node::new(&mock_html);
	
	match parser::parse_manga_details(manga_id, html_node) {
		Ok(manga) => {
			// Convertir le manga en JSON et le retourner
			let result_json = format!(r#"{{"id":"{}","title":"{}","description":"{}"}}"#, 
				manga.id, manga.title, manga.description.unwrap_or_default());
			let boxed = alloc::boxed::Box::new(result_json.into_bytes());
			alloc::boxed::Box::into_raw(boxed) as *const u8
		}
		Err(_) => core::ptr::null()
	}
}

#[no_mangle]
pub extern "C" fn get_chapter_list() -> *const u8 {
	// Cette fonction devrait prendre un manga_id en paramètre
	// Pour l'instant, créer un exemple avec One Piece
	let manga_id = String::from("/catalogue/one-piece");
	
	// Créer un HTML mock avec des scripts JavaScript pour les chapitres
	let mock_html = format!(r#"
		<html>
			<body>
				<div id="titreOeuvre">One Piece</div>
				<script>
					resetListe();
					creerListe(1, 1000);
					newSP("One Shot");
					finirListe(1001);
				</script>
			</body>
		</html>
	"#);
	
	let html_node = Node::new(&mock_html);
	let request_url = format!("{}{}", BASE_URL, manga_id);
	
	match parser::parse_chapter_list_dynamic_with_debug(manga_id, html_node, request_url) {
		Ok(chapters) => {
			// Convertir les chapitres en JSON simplifié
			let chapter_count = chapters.len();
			let result_json = format!(r#"{{"chapters":[],"count":{}}}"#, chapter_count);
			let boxed = alloc::boxed::Box::new(result_json.into_bytes());
			alloc::boxed::Box::into_raw(boxed) as *const u8
		}
		Err(_) => core::ptr::null()
	}
}

#[no_mangle]
pub extern "C" fn get_page_list() -> *const u8 {
	// Cette fonction devrait prendre manga_id et chapter_id en paramètres
	// Pour l'instant, créer un exemple avec One Piece chapitre 1
	let manga_id = String::from("/catalogue/one-piece");
	let chapter_id = String::from("1");
	
	// Créer un HTML mock avec des scripts JavaScript pour les pages
	let mock_html = format!(r#"
		<html>
			<body>
				<title>One Piece - Scans</title>
				<script>
					var eps1 = [];
					eps1.length = 20;
				</script>
			</body>
		</html>
	"#);
	
	let html_node = Node::new(&mock_html);
	
	match parser::parse_page_list(html_node, manga_id, chapter_id) {
		Ok(pages) => {
			// Convertir les pages en JSON simplifié
			let page_count = pages.len();
			let result_json = format!(r#"{{"pages":[],"count":{}}}"#, page_count);
			let boxed = alloc::boxed::Box::new(result_json.into_bytes());
			alloc::boxed::Box::into_raw(boxed) as *const u8
		}
		Err(_) => core::ptr::null()
	}
}