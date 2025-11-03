#![no_std]

use aidoku::{
	alloc::{format, String, Vec},
	imports::{html::Document, net::Request, std::send_partial_result},
	prelude::*,
	Chapter, FilterValue, ImageRequestProvider, Listing, ListingProvider, Manga, MangaPageResult,
	Page, PageContext, Result, Source,
};

extern crate alloc;

mod helper;
mod parser;

use helper::urlencode;
use parser::{
	has_next_page, parse_chapter_list, parse_manga_details, parse_manga_list, parse_page_list,
};

pub static BASE_URL: &str = "https://rimuscans.com";
pub static USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

pub struct RimuScans;

impl Source for RimuScans {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		let search_query = query.unwrap_or_default();

		let mut order_filter = String::from("update");
		let mut genre_filters: Vec<String> = Vec::new();
		let mut status_filter = String::new();

		for filter in filters {
			match filter {
				FilterValue::Select { id, value } => {
					if id == "order" && !value.is_empty() {
						order_filter = value;
					} else if id == "status" && !value.is_empty() {
						status_filter = value;
					}
				}
				FilterValue::MultiSelect {
					id,
					included,
					excluded: _,
				} => {
					if id == "genre" {
						for genre_id in included {
							if !genre_id.is_empty() {
								genre_filters.push(genre_id);
							}
						}
					}
				}
				_ => {}
			}
		}

		let mut url = if !search_query.is_empty() {
			let encoded_query = urlencode(search_query);
			if page == 1 {
				format!(
					"{}/manga/?s={}&order={}",
					BASE_URL, encoded_query, order_filter
				)
			} else {
				format!(
					"{}/manga/?s={}&order={}&page={}",
					BASE_URL, encoded_query, order_filter, page
				)
			}
		} else {
			Self::build_listing_url(&order_filter, page)
		};

		// Add genre filters
		for genre in &genre_filters {
			url.push_str(&format!("&genre%5B%5D={}", genre));
		}

		// Add status filter
		if !status_filter.is_empty() {
			url.push_str(&format!("&status%5B%5D={}", status_filter));
		}

		let html = Self::create_html_request(&url)?;

		let mangas = parse_manga_list(&html, BASE_URL);
		let has_more = has_next_page(&html);

		Ok(MangaPageResult {
			entries: mangas,
			has_next_page: has_more,
		})
	}

	fn get_manga_update(
		&self,
		manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		let mut updated_manga = manga.clone();

		if needs_details || needs_chapters {
			let manga_url = if let Some(url) = &manga.url {
				url.clone()
			} else {
				manga.key.clone()
			};

			let html = Self::create_html_request(&manga_url)?;

			if needs_details {
				let new_details = parse_manga_details(&html, manga.key.clone(), BASE_URL)?;

				updated_manga.title = new_details.title;
				updated_manga.cover = new_details.cover.or(updated_manga.cover);
				updated_manga.description = new_details.description.or(updated_manga.description);
				updated_manga.authors = new_details.authors.or(updated_manga.authors);
				updated_manga.artists = new_details.artists.or(updated_manga.artists);
				updated_manga.tags = new_details.tags.or(updated_manga.tags);
				updated_manga.status = new_details.status;
				updated_manga.content_rating = new_details.content_rating;
				updated_manga.viewer = new_details.viewer;
				updated_manga.url = new_details.url.or(updated_manga.url);

				send_partial_result(&updated_manga);
			}

			if needs_chapters {
				let chapters = parse_chapter_list(&html);
				let chapter_count = chapters.len();
				updated_manga.chapters = Some(chapters);
			}
		}

		Ok(updated_manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		let chapter_url = if let Some(url) = &chapter.url {
			url.clone()
		} else {
			chapter.key.clone()
		};

		let html = Self::create_html_request(&chapter_url)?;

		Ok(parse_page_list(&html))
	}
}

impl ListingProvider for RimuScans {
	fn get_manga_list(&self, listing: Listing, page: i32) -> Result<MangaPageResult> {
		match listing.id.as_str() {
			"popular" => self.get_popular_manga(page),
			"latest" => self.get_latest_manga(page),
			_ => self.get_latest_manga(page),
		}
	}
}

impl ImageRequestProvider for RimuScans {
	fn get_image_request(&self, url: String, _context: Option<PageContext>) -> Result<Request> {
		Ok(Request::get(&url)?
			.header("User-Agent", USER_AGENT)
			.header("Referer", BASE_URL)
			.header(
				"Accept",
				"image/avif,image/webp,image/apng,image/svg+xml,image/*,*/*;q=0.8",
			))
	}
}

impl RimuScans {
	fn create_html_request(url: &str) -> Result<Document> {
		Ok(Request::get(url)?
			.header("User-Agent", USER_AGENT)
			.header(
				"Accept",
				"text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8",
			)
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Accept-Encoding", "gzip, deflate, br")
			.header("DNT", "1")
			.header("Connection", "keep-alive")
			.header("Upgrade-Insecure-Requests", "1")
			.header("Referer", BASE_URL)
			.html()?)
	}

	fn build_listing_url(order: &str, page: i32) -> String {
		if page == 1 {
			format!("{}/manga/?order={}", BASE_URL, order)
		} else {
			format!("{}/manga/?order={}&page={}", BASE_URL, order, page)
		}
	}

	fn get_popular_manga(&self, page: i32) -> Result<MangaPageResult> {
		let url = Self::build_listing_url("popular", page);

		let html = Self::create_html_request(&url)?;

		let mangas = parse_manga_list(&html, BASE_URL);
		let has_more = has_next_page(&html);

		Ok(MangaPageResult {
			entries: mangas,
			has_next_page: has_more,
		})
	}

	fn get_latest_manga(&self, page: i32) -> Result<MangaPageResult> {
		let url = Self::build_listing_url("update", page);

		let html = Self::create_html_request(&url)?;

		let mangas = parse_manga_list(&html, BASE_URL);
		let has_more = has_next_page(&html);

		Ok(MangaPageResult {
			entries: mangas,
			has_next_page: has_more,
		})
	}
}

register_source!(RimuScans, ListingProvider, ImageRequestProvider);
