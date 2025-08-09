use aidoku::{
	prelude::*,
	error::Result,
	std::{
		html::Node,
		String, Vec,
		json::parse
	},
	Manga, Page, MangaPageResult, MangaStatus, MangaContentRating, MangaViewer, Chapter
};

pub fn parse_manga_list(html: Node) -> Result<MangaPageResult> {
	let mut mangas: Vec<Manga> = Vec::new();

	for item in html.select(".manga-item, .listupd .bs, .bsx").array() {
		let item = item
			.as_node()
			.expect("Failed to get data as array of nodes");
		
		let title = item.select("a").attr("title").read();
		let url = item.select("a").attr("href").read();
		let id = String::from(url.split('/').filter(|s| !s.is_empty()).last().unwrap_or(""));
		let cover = item.select("img").attr("data-src").read();
		let cover = if cover.is_empty() { 
			item.select("img").attr("src").read() 
		} else { 
			cover 
		};

		if !title.is_empty() && !id.is_empty() {
			mangas.push(Manga {
				id,
				cover,
				title,
				author: String::new(),
				artist: String::new(),
				description: String::new(),
				url,
				categories: Vec::new(),
				status: MangaStatus::Unknown,
				nsfw: MangaContentRating::Safe,
				viewer: MangaViewer::Rtl
			});
		}
	}

	let has_more = !html.select(".pagination .next, .hpage .r, .next_page").text().read().is_empty();

	Ok(MangaPageResult {
		manga: mangas,
		has_more,
	})
}

pub fn parse_manga_listing(html: Node) -> Result<MangaPageResult> {
	parse_manga_list(html)
}

pub fn parse_manga_details(base_url: String, manga_id: String, html: Node) -> Result<Manga> {	
	let cover = html.select(".thumb img, .wp-post-image").attr("src").read();
	let title = html.select("h1.entry-title, .post-title h1").text().read();
	
	let author = html.select(".infotable tr:contains(Auteur) td:last-child, .fmed b:contains(Author)+span").text().read();
	let artist = html.select(".infotable tr:contains(Artiste) td:last-child, .fmed b:contains(Artist)+span").text().read();
	
	let description = html.select(".desc p, .entry-content p, [itemprop=description]").text().read();
	
	let status_text = html.select(".infotable tr:contains(Statut) td:last-child, .imptdt:contains(Status) i").text().read().to_lowercase();
	let status = match status_text.as_str() {
		s if s.contains("en cours") => MangaStatus::Ongoing,
		s if s.contains("terminé") => MangaStatus::Completed,
		s if s.contains("abandonné") => MangaStatus::Cancelled,
		s if s.contains("en pause") => MangaStatus::Hiatus,
		_ => MangaStatus::Unknown,
	};
	
	let mut categories = Vec::new();
	for genre in html.select(".seriestugenre a, .mgen a").array() {
		let genre_name = genre.as_node().expect("genre node").text().read();
		if !genre_name.is_empty() {
			categories.push(genre_name);
		}
	}
	
	let url = format!("{}/catalogue/{}", base_url, manga_id);
	
	Ok(Manga {
		id: manga_id,
		cover,
		title,
		author,
		artist,
		description,
		url,
		categories,
		status,
		nsfw: MangaContentRating::Safe,
		viewer: MangaViewer::Rtl
	})
}

pub fn parse_chapter_list(_base_url: String, _manga_id: String, html: Node) -> Result<Vec<Chapter>> {
	let mut chapters: Vec<Chapter> = Vec::new();

	for chapter_element in html.select("#chapterlist li, .eplister li").array() {
		let chapter_element = chapter_element
			.as_node()
			.expect("Failed to get data as array of nodes");

		let chapter_url = chapter_element.select("a").attr("href").read();
		if chapter_url.is_empty() {
			continue;
		}

		let chapter_title = chapter_element.select(".chapternum, .epl-num").text().read();
		let chapter_date = chapter_element.select(".chapterdate, .epl-date").text().read();
		
		let chapter_id = String::from(chapter_url.split('/').filter(|s| !s.is_empty()).last().unwrap_or(""));
		
		let chapter_num: f32 = chapter_title
			.split_whitespace()
			.find_map(|s| s.parse::<f32>().ok())
			.unwrap_or(-1.0);

		chapters.push(Chapter {
			id: chapter_id,
			title: chapter_title,
			volume: -1.0,
			chapter: chapter_num,
			date_updated: parse_date(&chapter_date),
			scanlator: String::new(),
			url: chapter_url,
			lang: String::from("fr"),
		});
	}

	Ok(chapters)
}

pub fn parse_page_list(html: Node) -> Result<Vec<Page>> {
	let mut pages: Vec<Page> = Vec::new();
	
	// First try the ts_reader method (specific to SushiScan)
	let script_content = html.select("script").html().read();
	
	if let Some(start) = script_content.find("ts_reader.run(") {
		let json_start = start + "ts_reader.run(".len();
		if let Some(end) = script_content[json_start..].find(");") {
			let json_str = &script_content[json_start..json_start + end];
			
			if let Ok(json) = parse(json_str.as_bytes()) {
				if let Ok(json_obj) = json.as_object() {
					if let Ok(sources) = json_obj.get("sources").as_array() {
						// Get first source from the sources array
						if let Ok(first_source) = sources.get(0).as_object() {
							if let Ok(images) = first_source.get("images").as_array() {
								for (index, image) in images.enumerate() {
									if let Ok(image_url) = image.as_string() {
										let mut url = image_url.read();
										if url.starts_with("http://") {
											url = url.replace("http://", "https://");
										}
										pages.push(Page {
											index: index as i32,
											url,
											base64: String::new(),
											text: String::new(),
										});
									}
								}
								return Ok(pages);
							}
						}
					}
				}
			}
		}
	}
	
	// Fallback to standard image parsing
	for (index, img) in html.select("#readerarea img, .reading-content img").array().enumerate() {
		let img_node = img.as_node().expect("image node");
		let mut url = img_node.attr("data-src").read();
		if url.is_empty() {
			url = img_node.attr("src").read();
		}
		
		if !url.is_empty() && !url.starts_with("data:") {
			if url.starts_with("http://") {
				url = url.replace("http://", "https://");
			}
			pages.push(Page {
				index: index as i32,
				url,
				base64: String::new(),
				text: String::new(),
			});
		}
	}

	Ok(pages)
}

fn parse_date(date_str: &str) -> f64 {
	// Basic date parsing - could be improved with more sophisticated logic
	// For now, return -1.0 for unknown dates
	if date_str.is_empty() {
		-1.0
	} else {
		// TODO: Implement proper date parsing for French dates
		-1.0
	}
}