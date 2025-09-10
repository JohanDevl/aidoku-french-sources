#![no_std]
use aidoku_wrapper::prelude::*;
use mmrcms_template::*;

extern crate alloc;
use alloc::string::String;

fn get_source() -> MMRCMSSource<'static> {
	MMRCMSSource {
		base_url: "http://mangascan-fr.com",
		lang: "fr",
		manga_path: "manga",
		category: "Category",
		tags: "Tag",
		category_parser: |_, categories| {
			let mut nsfw = MangaContentRating::Safe;
			let mut viewer = MangaViewer::Rtl;
			for category in categories {
				match category.as_str() {
					"Adult" | "Smut" | "Mature" | "18+" | "Hentai" | "Erotique" => {
						nsfw = MangaContentRating::Nsfw
					}
					"Ecchi" | "16+" => {
						nsfw = match nsfw {
							MangaContentRating::Nsfw => MangaContentRating::Nsfw,
							_ => MangaContentRating::Suggestive,
						}
					}
					"Webtoon" | "Manhwa" | "Manhua" => viewer = MangaViewer::Scroll,
					_ => continue,
				}
			}
			(nsfw, viewer)
		},
		category_mapper: |idx| {
			String::from(match idx {
				1 => "1", // Action
				2 => "2", // Adventure
				3 => "3", // Comédie
				4 => "4", // Doujinshi
				5 => "5", // Drame
				6 => "6", // Ecchi
				7 => "7", // Fantasy
				8 => "8", // Webtoon
				9 => "9", // Harem
				10 => "10", // Historique
				11 => "11", // Horreur
				12 => "12", // Thriller
				13 => "13", // Arts Martiaux
				14 => "14", // Mature
				15 => "15", // Tragique
				16 => "16", // Mystère
				17 => "17", // One Shot
				18 => "18", // Psychologique
				19 => "19", // Romance
				20 => "20", // School Life
				21 => "21", // Science-fiction
				22 => "22", // Seinen
				23 => "23", // Erotique
				24 => "24", // Shoujo Ai
				25 => "25", // Shounen
				26 => "26", // Shounen Ai
				27 => "27", // Slice of Life
				28 => "28", // Sports
				29 => "29", // Surnaturel
				30 => "30", // Tragedy
				31 => "31", // Gangster
				32 => "32", // Crime
				33 => "33", // Biographique
				34 => "34", // Fantastique
				_ => "",
			})
		},
		tags_mapper: |_| String::new(),
		use_search_engine: true,
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