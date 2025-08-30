#![no_std]
use aidoku_stable::prelude::*;
use madara_stable_template::*;

extern crate alloc;

fn get_data() -> MadaraSiteData {
	MadaraSiteData {
		base_url: "https://manga-scantrad.io".into(),
		lang: "fr".into(),
		description_selector: "div.description-summary .summary__content".into(),
		date_format: "d MMMM yyyy".into(),
		status_filter_ongoing: "En cours".into(),
		status_filter_completed: "Terminé".into(),
		status_filter_cancelled: "Annulé".into(),
		status_filter_on_hold: "En pause".into(),
		popular: "Populaire".into(),
		trending: "Tendance".into(),
		alt_ajax: true,
		user_agent: Some("Mozilla/5.0 (iPhone; CPU iPhone OS 17_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) GSA/300.0.598994205 Mobile/15E148 Safari/605.1.15".into()),
		..MadaraSiteData::default()
	}
}

#[get_manga_list]
fn get_manga_list_impl(filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
	// Log: Starting get_manga_list with page: {}
	let data = get_data();
	// Log: Using base_url: {}
	let result = madara_stable_template::get_manga_list(filters, page, data);
	// Log: get_manga_list result: success/error
	result
}

#[get_manga_listing]
fn get_manga_listing_impl(listing: Listing, page: i32) -> Result<MangaPageResult> {
	// Log: Starting get_manga_listing: {} page: {}
	let data = get_data();
	// Log: Using base_url: {} with alt_ajax: {}
	let result = madara_stable_template::get_manga_listing(data, listing, page);
	// Log: get_manga_listing result: success/error
	result
}

#[get_manga_details]
fn get_manga_details_impl(id: String) -> Result<Manga> {
	// Log: Starting get_manga_details for id: {}
	let data = get_data();
	// Log: Using base_url: {} with description_selector: {}
	let result = madara_stable_template::get_manga_details(id, data);
	// Log: get_manga_details result: success/error
	result
}

#[get_chapter_list]
fn get_chapter_list_impl(id: String) -> Result<Vec<Chapter>> {
	// Log: Starting get_chapter_list for id: {}
	let data = get_data();
	// Log: Using base_url: {} with alt_ajax: {} date_format: {}
	let result = madara_stable_template::get_chapter_list(id, data);
	// Log: get_chapter_list result: success/error
	result
}

#[get_page_list]
fn get_page_list_impl(_manga_id: String, chapter_id: String) -> Result<Vec<Page>> {
	// Log: Starting get_page_list for chapter_id: {}
	let data = get_data();
	// Log: Using base_url: {} with image_selector: {}
	let result = madara_stable_template::get_page_list(chapter_id, data);
	// Log: get_page_list result: success/error
	result
}

