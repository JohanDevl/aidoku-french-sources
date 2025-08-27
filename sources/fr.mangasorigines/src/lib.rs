#![no_std]
use aidoku_stable::prelude::*;
use madara_stable_template::*;

extern crate alloc;

fn get_data() -> MadaraSiteData {
	MadaraSiteData {
		base_url: "https://mangas-origines.fr".into(),
		lang: "fr".into(),
		source_path: "oeuvre".into(),
		date_format: "dd/MM/yyyy".into(),
		description_selector: "div.summary__content p".into(),
		author_selector: "div.manga-authors".into(),
		status_filter_ongoing: "En cours".into(),
		status_filter_completed: "Terminé".into(),
		status_filter_cancelled: "Annulé".into(),
		status_filter_on_hold: "En pause".into(),
		popular: "Populaire".into(),
		trending: "Tendance".into(),
		alt_ajax: true,
		user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/604".into()),
		..MadaraSiteData::default()
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