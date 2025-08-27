#![no_std]

use aidoku_stable::prelude::*;

pub static BASE_URL: &str = "https://poseidonscans.com";
pub static API_URL: &str = "https://poseidonscans.com/api";

#[no_mangle]
pub extern "C" fn get_manga_list() -> *const u8 {
	core::ptr::null()
}

#[no_mangle]
pub extern "C" fn get_manga_listing() -> *const u8 {
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

#[no_mangle]
pub extern "C" fn modify_image_request() -> *const u8 {
	core::ptr::null()
}