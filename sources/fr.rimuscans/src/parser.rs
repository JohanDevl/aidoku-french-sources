use crate::helper::{make_absolute_url, parse_relative_date};
use aidoku::{
	alloc::{string::ToString, String, Vec},
	imports::html::Document,
	Chapter, ContentRating, Manga, MangaStatus, Page, PageContent, Result, UpdateStrategy, Viewer,
};

extern crate alloc;

fn calculate_content_rating(tags: &Option<Vec<String>>) -> ContentRating {
	if let Some(tags) = tags {
		for tag in tags {
			let tag_lower = tag.to_lowercase();
			match tag_lower.as_str() {
				"adult" | "adulte" | "mature" | "hentai" | "smut" | "érotique" => {
					return ContentRating::NSFW;
				}
				"ecchi" | "suggestif" | "suggestive" => {
					return ContentRating::Suggestive;
				}
				_ => {}
			}
		}
	}
	ContentRating::Safe
}

fn calculate_viewer(tags: &Option<Vec<String>>) -> Viewer {
	if let Some(tags) = tags {
		for tag in tags {
			let tag_lower = tag.to_lowercase();
			match tag_lower.as_str() {
				"manhwa" | "manhua" | "webtoon" | "scroll" | "vertical" => {
					return Viewer::Vertical;
				}
				"manga" => {
					return Viewer::RightToLeft;
				}
				_ => {}
			}
		}
	}
	Viewer::RightToLeft
}

fn extract_chapter_number_from_title(title: &str) -> Option<f32> {
	let title_lower = title.to_lowercase();

	if let Some(pos) = title_lower.find("chapitre ") {
		let after = &title_lower[pos + 9..];
		let num_str: String = after
			.chars()
			.take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
			.collect();

		let cleaned = num_str.replace(',', ".");
		if let Ok(num) = cleaned.parse::<f32>() {
			return Some(num);
		}
	}

	if title_lower.starts_with("ch.") || title_lower.starts_with("ch ") {
		let start_pos = if title_lower.starts_with("ch.") { 3 } else { 3 };
		let num_str: String = title_lower
			.chars()
			.skip(start_pos)
			.take_while(|c| c.is_ascii_digit() || *c == '.' || *c == ',')
			.collect();

		let cleaned = num_str.replace(',', ".");
		if let Ok(num) = cleaned.parse::<f32>() {
			return Some(num);
		}
	}

	None
}

fn extract_chapter_number_from_url(url: &str) -> Option<f32> {
	if let Some(last_part) = url.split('/').last() {
		if let Ok(num) = last_part.parse::<f32>() {
			return Some(num);
		}

		if last_part.contains('-') {
			if let Some(num_str) = last_part.split('-').last() {
				if let Ok(num) = num_str.parse::<f32>() {
					return Some(num);
				}
			}
		}
	}
	None
}

pub fn parse_manga_list(html: &Document, base_url: &str) -> Vec<Manga> {
	let mut mangas = Vec::new();

	if let Some(items) = html.select(".listupd .bs .bsx, .utao .uta .imgu") {
		for item in items {
			let link = if let Some(links) = item.select("a") {
				if let Some(l) = links.first() {
					l
				} else {
					continue;
				}
			} else {
				continue;
			};

			let url = link.attr("href").unwrap_or_default();
			let title = link.attr("title").unwrap_or_default();

			if url.is_empty() || title.is_empty() {
				continue;
			}

			let key = url.clone();

			let cover = if let Some(imgs) = item.select("img") {
				if let Some(img) = imgs.first() {
					let cover_url = img
						.attr("data-lazy-src")
						.or_else(|| img.attr("data-src"))
						.or_else(|| img.attr("src"))
						.unwrap_or_default();

					if !cover_url.is_empty() {
						Some(make_absolute_url(base_url, &cover_url))
					} else {
						None
					}
				} else {
					None
				}
			} else {
				None
			};

			let tags: Option<Vec<String>> = None;
			let content_rating = calculate_content_rating(&tags);
			let viewer = calculate_viewer(&tags);

			mangas.push(Manga {
				key: key.clone(),
				cover,
				title,
				authors: None,
				artists: None,
				description: None,
				tags,
				status: MangaStatus::Unknown,
				content_rating,
				viewer,
				chapters: None,
				url: Some(make_absolute_url(base_url, &url)),
				next_update_time: None,
				update_strategy: UpdateStrategy::Always,
			});
		}
	}

	mangas
}

pub fn parse_manga_details(html: &Document, manga_key: String, base_url: &str) -> Result<Manga> {
	let title = if let Some(title_elems) = html.select("h1.entry-title") {
		if let Some(elem) = title_elems.first() {
			elem.text().unwrap_or_default()
		} else {
			String::new()
		}
	} else {
		String::new()
	};

	let author = None; // Site doesn't show author info
	let artist = None; // Site doesn't show artist info

	let description = if let Some(desc_elems) = html.select("div.entry-content-single") {
		if let Some(elem) = desc_elems.first() {
			let text = elem.text().unwrap_or_default().trim().to_string();
			if !text.is_empty() {
				Some(text)
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	let tags = if let Some(genre_elems) = html.select("div.wd-full span.mgen a") {
		let mut tags_vec = Vec::new();
		for elem in genre_elems {
			let tag = elem.text().unwrap_or_default();
			if !tag.is_empty() {
				tags_vec.push(tag);
			}
		}
		if !tags_vec.is_empty() {
			Some(tags_vec)
		} else {
			None
		}
	} else {
		None
	};

	let cover = if let Some(cover_elems) = html.select("div.thumb img") {
		if let Some(elem) = cover_elems.first() {
			let src = elem.attr("src").unwrap_or_default();
			if !src.is_empty() {
				Some(make_absolute_url(base_url, &src))
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	let status = if let Some(status_items) = html.select("div.imptdt") {
		let mut found_status = MangaStatus::Unknown;
		for item in status_items {
			let text = item.text().unwrap_or_default().to_lowercase();
			if text.contains("status") {
				if text.contains("ongoing") || text.contains("en cours") {
					found_status = MangaStatus::Ongoing;
				} else if text.contains("completed") || text.contains("terminé") {
					found_status = MangaStatus::Completed;
				}
				break;
			}
		}
		found_status
	} else {
		MangaStatus::Unknown
	};

	let content_rating = calculate_content_rating(&tags);
	let viewer = calculate_viewer(&tags);

	Ok(Manga {
		key: manga_key.clone(),
		cover,
		title,
		authors: author,
		artists: artist,
		description,
		tags,
		status,
		content_rating,
		viewer,
		chapters: None,
		url: Some(manga_key),
		next_update_time: None,
		update_strategy: UpdateStrategy::Always,
	})
}

pub fn parse_chapter_list(html: &Document) -> Vec<Chapter> {
	let mut chapters = Vec::new();

	if let Some(items) = html.select("div.eplister ul li") {
		let mut temp_chapters = Vec::new();
		let mut max_chapter_number: Option<f32> = None;

		for item in items {
			let link = if let Some(links) = item.select("div.eph-num a") {
				if let Some(l) = links.first() {
					l
				} else {
					continue;
				}
			} else {
				continue;
			};

			let url = link.attr("href").unwrap_or_default();

			if url.is_empty() {
				continue;
			}

			let title = if let Some(title_span) = item.select("span.chapternum") {
				if let Some(span) = title_span.first() {
					span.text().unwrap_or_default()
				} else {
					String::new()
				}
			} else {
				String::new()
			};

			let date_uploaded = if let Some(date_span) = item.select("span.chapterdate") {
				if let Some(span) = date_span.first() {
					let date_text = span.text().unwrap_or_default().to_lowercase();
					parse_relative_date(&date_text)
				} else {
					None
				}
			} else {
				None
			};

			let chapter_number = extract_chapter_number_from_title(&title)
				.or_else(|| extract_chapter_number_from_url(&url));

			if let Some(num) = chapter_number {
				if max_chapter_number.map_or(true, |max| num > max) {
					max_chapter_number = Some(num);
				}
			}

			temp_chapters.push((url, title, date_uploaded, chapter_number));
		}

		let mut unnumbered_offset = 1.0;
		for (url, title, date_uploaded, chapter_number) in temp_chapters {
			let final_chapter_number = match chapter_number {
				Some(num) => Some(num),
				None => max_chapter_number.map(|n| {
					let result = n + unnumbered_offset;
					unnumbered_offset += 1.0;
					result
				}),
			};

			chapters.push(Chapter {
				key: url.clone(),
				title: if !title.is_empty() { Some(title) } else { None },
				date_uploaded,
				url: Some(url),
				chapter_number: final_chapter_number,
				volume_number: None,
				scanlators: None,
				language: None,
				thumbnail: None,
				locked: false,
			});
		}
	}

	chapters
}

pub fn parse_page_list(html: &Document) -> Vec<Page> {
	let mut pages = Vec::new();

	if let Some(items) = html.select("#content img, div#readerarea img") {
		for item in items {
			let url = item
				.attr("data-lazy-src")
				.or_else(|| item.attr("data-src"))
				.or_else(|| item.attr("src"))
				.unwrap_or_default();

			if !url.is_empty() && !url.contains("logo") && !url.contains("icon") {
				pages.push(Page {
					content: PageContent::Url(url, None),
					thumbnail: None,
					has_description: false,
					description: None,
				});
			}
		}
	}

	pages
}

pub fn has_next_page(html: &Document) -> bool {
	if let Some(voir_plus) = html.select("div.hpage a.r") {
		if !voir_plus.is_empty() {
			return true;
		}
	}

	false
}
