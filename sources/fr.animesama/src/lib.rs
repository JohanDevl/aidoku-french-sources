#![no_std]

extern crate alloc;
use aidoku_stable::prelude::*;

// Modules contenant la logique de parsing sophistiquée d'AnimeSama
// Ces modules sont conservés pour quand le wrapper aidoku-stable supportera les requêtes HTTP
pub mod parser;
pub mod helper;

pub static BASE_URL: &str = "https://anime-sama.fr";
pub static CDN_URL: &str = "https://anime-sama.fr/s2/scans";

#[no_mangle]
pub extern "C" fn get_manga_list() -> *const u8 {
	// LIMITATION: aidoku-stable-wrapper n'implémente pas encore l'interface complète
	// L'app Aidoku s'attend à un format binaire spécifique qui n'est pas documenté
	// Toutes les sources françaises du projet ont le même problème "Unknown Error"
	// TODO: Attendre que le wrapper stable implémente la sérialisation correcte
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_manga_details() -> *const u8 {
	// LIMITATION: aidoku-stable-wrapper n'implémente pas encore l'interface complète
	// Toutes les sources françaises du projet ont le même problème "Unknown Error"
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_chapter_list() -> *const u8 {
	// LIMITATION: aidoku-stable-wrapper n'implémente pas encore l'interface complète
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_page_list() -> *const u8 {
	// LIMITATION: aidoku-stable-wrapper n'implémente pas encore l'interface complète
	core::ptr::null()
}