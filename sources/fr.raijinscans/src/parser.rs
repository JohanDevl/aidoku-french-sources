use crate::helper::{decode_base64, make_absolute_url, parse_relative_date};
use aidoku::{
	alloc::{format, string::ToString, vec, String, Vec},
	imports::html::Document,
	Chapter, ContentRating, Manga, MangaStatus, Page, PageContent, Result, UpdateStrategy, Viewer,
};

extern crate alloc;

pub fn parse_manga_details(html: &Document, manga_key: String, base_url: &str) -> Result<Manga> {
	let title = if let Some(title_elems) = html.select("h1.serie-title") {
		if let Some(elem) = title_elems.first() {
			elem.text().unwrap_or_default()
		} else {
			String::new()
		}
	} else {
		String::new()
	};

	let author = if let Some(author_elems) =
		html.select("div.stat-item:has(span:contains(Auteur)) span.stat-value")
	{
		if let Some(elem) = author_elems.first() {
			let text = elem.text().unwrap_or_default();
			if !text.is_empty() {
				Some(vec![text])
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	let artist = if let Some(artist_elems) =
		html.select("div.stat-item:has(span:contains(Artiste)) span.stat-value")
	{
		if let Some(elem) = artist_elems.first() {
			let text = elem.text().unwrap_or_default();
			if !text.is_empty() {
				Some(vec![text])
			} else {
				None
			}
		} else {
			None
		}
	} else {
		None
	};

	let mut description = None;
	if let Some(scripts) = html.select("script") {
		for script in scripts {
			if let Some(html_content) = script.html() {
				if html_content.contains("content.innerHTML") {
					if let Some(start) = html_content.find("content.innerHTML = `") {
						let desc_start = start + 21;
						if let Some(end) = html_content[desc_start..].find("`;") {
							description =
								Some(html_content[desc_start..desc_start + end].to_string());
							break;
						}
					}
				}
			}
		}
	}

	if description.is_none() {
		if let Some(desc_elems) = html.select("div.description-content") {
			if let Some(elem) = desc_elems.first() {
				let text = elem.text().unwrap_or_default();
				if !text.is_empty() {
					description = Some(text);
				}
			}
		}
	}

	let tags = if let Some(genre_elems) = html.select("div.genre-list div.genre-link") {
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

	let cover = if let Some(cover_elems) = html.select("img.cover") {
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

	let status = if let Some(status_elems) =
		html.select("div.stat-item:has(span:contains(État)) span.manga")
	{
		if let Some(elem) = status_elems.first() {
			let status_text = elem.text().unwrap_or_default().to_lowercase();
			if status_text.contains("en cours") {
				MangaStatus::Ongoing
			} else if status_text.contains("terminé") || status_text.contains("termine") {
				MangaStatus::Completed
			} else {
				MangaStatus::Unknown
			}
		} else {
			MangaStatus::Unknown
		}
	} else {
		MangaStatus::Unknown
	};

	Ok(Manga {
		key: manga_key.clone(),
		cover,
		title,
		authors: author,
		artists: artist,
		description,
		tags,
		status,
		content_rating: ContentRating::Safe,
		viewer: Viewer::LeftToRight,
		chapters: None,
		url: Some(manga_key),
		next_update_time: None,
		update_strategy: UpdateStrategy::Always,
	})
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

fn extract_chapter_number_from_title(title: &str) -> Option<f32> {
	if let Some(pos) = title.find("Chapitre ") {
		let after_chapitre = &title[pos + 9..];
		let num_str: String = after_chapitre
			.chars()
			.take_while(|c| c.is_ascii_digit() || *c == '.')
			.collect();

		if let Ok(num) = num_str.parse::<f32>() {
			return Some(num);
		}
	}

	if title.starts_with("Ch.") || title.starts_with("Ch ") {
		let num_str: String = title[3..]
			.chars()
			.take_while(|c| c.is_ascii_digit() || *c == '.')
			.collect();

		if let Ok(num) = num_str.parse::<f32>() {
			return Some(num);
		}
	}

	None
}

pub fn parse_chapter_list(html: &Document) -> Vec<Chapter> {
	let mut chapters = Vec::new();

	if let Some(items) = html.select("ul.scroll-sm li.item") {
		for item in items {
			// Skip premium chapters
			if let Some(class) = item.attr("class") {
				if class.contains("premium-chapter") {
					continue;
				}
			}

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

			if url.is_empty() || url.contains("/connexion") {
				continue;
			}

			let date_uploaded = if let Some(spans) = item.select("a span:nth-of-type(2)") {
				if let Some(span) = spans.first() {
					let date_text = span.text().unwrap_or_default().to_lowercase();
					parse_relative_date(&date_text)
				} else {
					None
				}
			} else {
				None
			};

			let chapter_number = extract_chapter_number_from_url(&url)
				.or_else(|| extract_chapter_number_from_title(&title));

			let formatted_title = if let Some(num) = chapter_number {
				if title.is_empty() || !title.contains("Chapitre") {
					Some(format!("Chapitre {}", num))
				} else {
					Some(format!("Chapitre {} - {}", num, title))
				}
			} else {
				if !title.is_empty() {
					Some(title.clone())
				} else {
					None
				}
			};

			chapters.push(Chapter {
				key: url.clone(),
				title: formatted_title,
				date_uploaded,
				url: Some(url),
				chapter_number,
				volume_number: None,
				scanlators: None,
				language: Some(String::from("fr")),
				thumbnail: None,
				locked: false,
			});
		}
	}

	chapters
}

pub fn parse_page_list(html: &Document) -> Vec<Page> {
	let mut pages = Vec::new();

	if let Some(items) = html.select("div.protected-image-data") {
		for item in items {
			let encoded = item.attr("data-src").unwrap_or_default();

			if !encoded.is_empty() {
				let url = if let Some(decoded) = decode_base64(&encoded) {
					decoded
				} else {
					encoded
				};

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
	if let Some(next_elems) = html.select("li.page-item:not(.disabled) a[rel=next]") {
		if !next_elems.is_empty() {
			return true;
		}
	}

	if let Some(load_more) = html.select("a#load-more-manga") {
		if !load_more.is_empty() {
			return true;
		}
	}

	false
}
