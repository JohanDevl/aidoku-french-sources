#![no_std]
use aidoku_wrapper::prelude::*;
use madara_template::*;

extern crate alloc;

fn get_data() -> MadaraSiteData {
	MadaraSiteData {
		base_url: "https://reaperscans.fr".into(),
		lang: "fr".into(),
		source_path: "serie".into(),
		date_format: "dd/MM/yyyy".into(),
		status_filter_ongoing: "En cours".into(),
		status_filter_completed: "Terminé".into(),
		status_filter_cancelled: "Annulé".into(),
		status_filter_on_hold: "En pause".into(),
		popular: "Populaire".into(),
		trending: "Tendance".into(),
		alt_ajax: true,
		user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 17_4_1 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.3.1 Mobile/15E148 Safari/604.1".into()),
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