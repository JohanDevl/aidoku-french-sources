#![no_std]
use aidoku_wrapper::prelude::*;
use mangastream_template::*;

extern crate alloc;

fn get_instance() -> MangaStreamSource {
	MangaStreamSource {
		base_url: "https://sushiscan.net".into(),
		listing: ["Dernières", "Populaire", "Nouveau"],
		status_options: ["En Cours", "Terminé", "En Pause", "", "Abandonné"],
		traverse_pathname: "catalogue",
		last_page_text: "Suivant",
		last_page_text_2: "Next",
		status_options_search: ["", "ongoing", "completed", "hiatus", "en pause"],
		type_options_search: ["", "manga", "manhwa", "manhua", "comic", "fanfiction", "bande-dessinée", "global-manga", "artbook", "anime-comics", "guidebook", ""],
		manga_details_categories: ".seriestugenre a",
		manga_details_author: ".infotable td:contains(Auteur)+td",
		manga_details_artist: ".infotable td:contains(Dessinateur)+td",
		manga_details_status: ".infotable td:contains(Statut)+td",
		manga_details_type: ".infotable td:contains(Type)+td",
		chapter_date_format : "MMMM dd, yyyy",
		language: "fr",
		locale: "fr-FR",
		alt_pages: true,
		..MangaStreamSource::default()
	}
}

#[no_mangle]
pub extern "C" fn get_manga_list() -> *const u8 {
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_manga_details() -> *const u8 {
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_chapter_list() -> *const u8 {
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_page_list() -> *const u8 {
	core::ptr::null()
}