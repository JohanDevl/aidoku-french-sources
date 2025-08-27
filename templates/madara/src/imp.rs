use super::MadaraParams;
use crate::helper;
use aidoku::{
	AidokuError, Chapter, ContentRating, DeepLinkResult, FilterValue, Manga, MangaPageResult,
	Page, PageContent, Result, Viewer,
	alloc::{String, Vec, vec},
	imports::{net::Request},
	prelude::*,
};

use alloc::string::ToString;

pub trait Impl {
	fn new() -> Self;

	fn params(&self) -> MadaraParams;

	fn get_search_manga_list(
		&self,
		params: &MadaraParams,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// Default implementation - sites can override this
		let mut url = format!("{}/{}/", params.base_url, params.search_path);
		
		// Add search query if provided
		if let Some(query) = query {
			url.push_str(&format!("?s={}", query));
		}
		
		// Add page parameter
		if page > 1 {
			if url.contains('?') {
				url.push_str(&format!("&page={}", page));
			} else {
				url.push_str(&format!("?page={}", page));
			}
		}
		
		self.get_manga_from_page(params, url)
	}

	fn get_manga_details(&self, params: &MadaraParams, mut manga: Manga) -> Result<Manga> {
		let url = if manga.url.is_some() {
			manga.url.clone().unwrap()
		} else {
			format!("{}/{}/{}/", params.base_url, params.source_path, manga.key)
		};
		
		let mut req = Request::get(&url)?;
		if let Some(user_agent) = &params.user_agent {
			req = req.header("User-Agent", user_agent);
		}
		
		let html = req.html()?;
		
		// Extract title
		manga.title = html.select("h1").and_then(|el| el.text()).unwrap_or(manga.title);
		
		// Extract cover image
		if let Some(cover) = html.select(".manga-poster img, .thumb img").and_then(|el| el.attr("src")) {
			manga.cover = Some(cover);
		}
		
		// Extract author
		manga.author = html.select(params.author_selector.as_ref()).and_then(|el| el.text());
		
		// Extract description
		manga.description = html.select(params.description_selector.as_ref()).and_then(|el| el.text());
		
		// Extract categories
		let mut categories = Vec::new();
		if let Some(genre_elements) = html.select(params.genre_selector.as_ref()) {
			for genre_element in genre_elements {
				if let Some(genre) = genre_element.text() {
					categories.push(genre);
				}
			}
		}
		manga.categories = Some(categories);
		
		// Extract status
		let status_text = html
			.select("div.post-content_item:contains(Status) div.summary-content, .imptdt:contains(Statut) i")
			.and_then(|el| el.text())
			.map(|s| s.to_lowercase())
			.unwrap_or_default();
		
		manga.status = match status_text.as_str() {
			s if s.contains("ongoing") || s.contains("en cours") => Some("Ongoing".into()),
			s if s.contains("completed") || s.contains("terminé") => Some("Completed".into()),
			s if s.contains("cancelled") || s.contains("annulé") => Some("Cancelled".into()),
			s if s.contains("hiatus") || s.contains("en pause") => Some("Hiatus".into()),
			_ => None,
		};
		
		// Extract content rating
		let mut content_rating = ContentRating::Safe;
		if let Some(ref categories) = manga.categories {
			let suggestive_tags = ["ecchi", "mature", "adult"];
			for tag in suggestive_tags {
				if categories.iter().any(|v| v.to_lowercase() == tag) {
					content_rating = ContentRating::Suggestive;
					break;
				}
			}
		}
		manga.content_rating = Some(content_rating);
		
		// Determine viewer type
		let series_type = html
			.select("div.post-content_item:contains(Type) div.summary-content")
			.and_then(|el| el.text())
			.map(|s| s.to_lowercase())
			.unwrap_or_default();
		
		manga.viewer = Some(if series_type.contains("manhwa") || series_type.contains("manhua") || series_type.contains("webtoon") {
			Viewer::Scroll
		} else if series_type.contains("manga") {
			Viewer::RightToLeft
		} else {
			Viewer::Scroll
		});
		
		manga.url = Some(url);
		Ok(manga)
	}

	fn get_chapter_list(&self, params: &MadaraParams, manga: &Manga) -> Result<Vec<Chapter>> {
		let url = if let Some(ref manga_url) = manga.url {
			manga_url.clone()
		} else {
			format!("{}/{}/{}/", params.base_url, params.source_path, manga.key)
		};
		
		let mut req = Request::get(&url)?;
		if let Some(user_agent) = &params.user_agent {
			req = req.header("User-Agent", user_agent);
		}
		
		let html = req.html()?;
		let mut chapters = Vec::new();
		
		if let Some(chapter_elements) = html.select(params.chapter_selector.as_ref()) {
			for item in chapter_elements {
				if let Some(link) = item.select_first("a") {
					if let Some(href) = link.attr("href") {
						let chapter_title = link.text().unwrap_or_default();
						
						// Extract chapter number from URL or title
						let chapter_num = self.extract_chapter_number(&href, &chapter_title);
						
						// Parse date
						let date_str = item.select_first(".chapter-release-date")
							.and_then(|el| el.text())
							.unwrap_or_default();
						let date_updated = self.parse_chapter_date(&date_str);
						
						chapters.push(Chapter {
							key: href.clone(),
							title: Some(chapter_title),
							volume: None,
							chapter_number: if chapter_num >= 0.0 { Some(chapter_num) } else { None },
							date_updated: if date_updated > 0.0 { Some(date_updated) } else { None },
							scanlators: None,
							url: Some(href),
							lang: Some(params.lang.to_string()),
						});
					}
				}
			}
		}
		
		Ok(chapters)
	}

	fn get_page_list(&self, params: &MadaraParams, chapter: Chapter) -> Result<Vec<Page>> {
		let url = if let Some(chapter_url) = chapter.url {
			chapter_url
		} else {
			format!("{}/{}/", params.base_url, chapter.key)
		};
		
		let mut req = Request::get(&url)?;
		if let Some(user_agent) = &params.user_agent {
			req = req.header("User-Agent", user_agent);
		}
		
		let html = req.html()?;
		let mut pages = Vec::new();
		
		if let Some(img_elements) = html.select(params.image_selector.as_ref()) {
			for img_element in img_elements {
				// Try multiple image attributes
				let img_url = if let Some(url) = img_element.attr("data-lazy-src") {
					url
				} else if let Some(url) = img_element.attr("data-src") {
					url
				} else if let Some(url) = img_element.attr("src") {
					url
				} else {
					continue;
				};
				
				pages.push(Page {
					content: PageContent::url(img_url),
					..Default::default()
				});
			}
		}
		
		Ok(pages)
	}

	fn handle_deep_link(&self, params: &MadaraParams, url: String) -> Result<Option<DeepLinkResult>> {
		// Extract manga ID from URL
		let base_url = params.base_url.as_ref();
		if !url.starts_with(base_url) {
			return Ok(None);
		}
		
		let manga_path = format!("/{}/", params.source_path);
		if url.contains(&manga_path) {
			let manga_key = url
				.replace(base_url, "")
				.replace(&manga_path, "")
				.trim_start_matches('/')
				.trim_end_matches('/')
				.to_string();
			
			if !manga_key.is_empty() {
				return Ok(Some(DeepLinkResult::Manga { key: manga_key }));
			}
		}
		
		Ok(None)
	}

	// Helper methods that can be overridden
	fn get_manga_from_page(&self, params: &MadaraParams, url: String) -> Result<MangaPageResult> {
		let mut req = Request::get(&url)?;
		if let Some(user_agent) = &params.user_agent {
			req = req.header("User-Agent", user_agent);
		}
		
		let html = req.html()?;
		let mut entries = Vec::new();
		
		if let Some(manga_elements) = html.select(params.search_selector.as_ref()) {
			for item in manga_elements {
				if let Some(link) = item.select_first("a") {
					if let Some(href) = link.attr("href") {
						// Extract manga ID from URL
						let key = href
							.replace(params.base_url.as_ref(), "")
							.replace(&format!("/{}/", params.source_path), "")
							.trim_start_matches('/')
							.trim_end_matches('/')
							.to_string();
						
						if key.is_empty() {
							continue;
						}
						
						let title = link.attr("title").unwrap_or_else(|| {
							item.select_first("h3, h4, .title")
								.and_then(|el| el.text())
								.unwrap_or_default()
						});
						
						if title.is_empty() {
							continue;
						}
						
						// Get cover image
						let cover = if let Some(img) = item.select_first("img") {
							img.attr("data-lazy-src")
								.or_else(|| img.attr("data-src"))
								.or_else(|| img.attr("src"))
						} else {
							None
						};
						
						entries.push(Manga {
							key,
							title,
							cover,
							url: Some(href),
							..Default::default()
						});
					}
				}
			}
		}
		
		// Check for pagination
		let has_next_page = html.select(".pagination .next, .hpage .r").is_some();
		
		Ok(MangaPageResult {
			entries,
			has_next_page,
		})
	}

	fn extract_chapter_number(&self, chapter_url: &str, title: &str) -> f32 {
		// Extract chapter number from URL or title
		if let Some(chapter_pos) = chapter_url.to_lowercase().find("chapter-") {
			let after_chapter = &chapter_url[chapter_pos + 8..];
			if let Some(num_str) = after_chapter.split('-').next() {
				if let Ok(num) = num_str.parse::<f32>() {
					return num;
				}
			}
		}
		
		// Try to extract from title
		let words: Vec<&str> = title.split_whitespace().collect();
		for (i, word) in words.iter().enumerate() {
			if word.to_lowercase().contains("chapter") || word.to_lowercase().contains("chapitre") {
				if i + 1 < words.len() {
					if let Ok(num) = words[i + 1].parse::<f32>() {
						return num;
					}
				}
			}
		}
		
		-1.0
	}

	fn parse_chapter_date(&self, _date_str: &str) -> f64 {
		// Simple date parsing - this can be overridden by specific implementations
		0.0
	}
}