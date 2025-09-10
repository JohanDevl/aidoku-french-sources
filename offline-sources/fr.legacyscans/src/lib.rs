#![no_std]
use madara_template::*;

extern crate alloc;

fn get_data() -> MadaraSiteData {
	MadaraSiteData {
		base_url: "https://legacy-scans.com".into(),
		lang: "fr".into(),
		source_path: "manga".into(),
		date_format: "dd/MM/yyyy".into(),
		status_filter_ongoing: "En cours".into(),
		status_filter_completed: "Terminé".into(),
		status_filter_cancelled: "Annulé".into(),
		status_filter_on_hold: "En pause".into(),
		popular: "Populaire".into(),
		trending: "Tendance".into(),
		alt_ajax: true,
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