#![no_std]

use aidoku::{
	Chapter, ContentRating, FilterValue, Manga, MangaPageResult, MangaStatus, Page, PageContent,
	Result, Source, Viewer,
	alloc::{String, Vec, vec},
	imports::std::send_partial_result,
	prelude::*,
};


// Modules contenant la logique de parsing sophistiquée d'AnimeSama
// TODO: Réactiver quand les API seront adaptées pour aidoku
// pub mod parser;
// pub mod helper;

pub const BASE_URL: &str = "https://anime-sama.fr";
pub const CDN_URL: &str = "https://anime-sama.fr/s2/scans";

struct AnimeSama;

impl Source for AnimeSama {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		_filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// Test avec des mangas hardcodés pour vérifier que l'interface fonctionne
		let mut entries = vec![
			Manga {
				key: "/catalogue/one-piece".into(),
				title: "One Piece".into(),
				authors: Some(vec!["Eiichiro Oda".into()]),
				artists: Some(vec!["Eiichiro Oda".into()]),
				description: Some("L'histoire de One Piece suit les aventures de Monkey D. Luffy.".into()),
				url: Some(format!("{}/catalogue/one-piece", BASE_URL)),
				cover: Some("https://anime-sama.fr/images/one-piece.jpg".into()),
				tags: Some(vec!["Action".into(), "Aventure".into(), "Shonen".into()]),
				status: MangaStatus::Ongoing,
				content_rating: ContentRating::Safe,
				viewer: Viewer::RightToLeft,
				..Default::default()
			},
			Manga {
				key: "/catalogue/naruto".into(),
				title: "Naruto".into(),
				authors: Some(vec!["Masashi Kishimoto".into()]),
				artists: Some(vec!["Masashi Kishimoto".into()]),
				description: Some("L'histoire de Naruto Uzumaki, un jeune ninja.".into()),
				url: Some(format!("{}/catalogue/naruto", BASE_URL)),
				cover: Some("https://anime-sama.fr/images/naruto.jpg".into()),
				tags: Some(vec!["Action".into(), "Aventure".into(), "Shonen".into()]),
				status: MangaStatus::Completed,
				content_rating: ContentRating::Safe,
				viewer: Viewer::RightToLeft,
				..Default::default()
			},
		];

		// Si une query est fournie, filtrer les résultats
		if let Some(search_query) = query {
			let search_lower = search_query.to_lowercase();
			entries = entries
				.into_iter()
				.filter(|manga| manga.title.to_lowercase().contains(&search_lower))
				.collect();
		}

		Ok(MangaPageResult {
			entries,
			has_next_page: page < 2, // Simuler la pagination
		})
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		// Test avec des données hardcodées pour One Piece
		if needs_details {
			manga.title = "One Piece".into();
			manga.authors = Some(vec!["Eiichiro Oda".into()]);
			manga.artists = Some(vec!["Eiichiro Oda".into()]);
			manga.description = Some("L'histoire de One Piece suit les aventures de Monkey D. Luffy, un jeune homme dont le corps a acquis les propriétés du caoutchouc après avoir mangé un fruit du démon. Avec son équipage de pirates, il explore Grand Line à la recherche du trésor ultime connu sous le nom de 'One Piece'.".into());
			manga.url = Some(format!("{}{}", BASE_URL, manga.key));
			manga.cover = Some("https://anime-sama.fr/images/one-piece-cover.jpg".into());
			manga.tags = Some(vec!["Action".into(), "Aventure".into(), "Comédie".into(), "Shonen".into()]);
			manga.status = MangaStatus::Ongoing;
			manga.content_rating = ContentRating::Safe;
			manga.viewer = Viewer::RightToLeft;

			if needs_chapters {
				send_partial_result(&manga);
			}
		}

		if needs_chapters {
			// Test avec quelques chapitres de One Piece
			manga.chapters = Some(vec![
				Chapter {
					key: "1".into(),
					title: Some("Romance Dawn".into()),
					chapter_number: Some(1.0),
					volume_number: Some(1.0),
					date_uploaded: Some(1640995200),
					scanlators: Some(vec!["AnimeSama".into()]),
					url: Some(format!("{}/catalogue/one-piece/scan/vf/1", BASE_URL)),
					..Default::default()
				},
				Chapter {
					key: "2".into(),
					title: Some("L'homme au chapeau de paille".into()),
					chapter_number: Some(2.0),
					volume_number: Some(1.0),
					date_uploaded: Some(1640995200),
					scanlators: Some(vec!["AnimeSama".into()]),
					url: Some(format!("{}/catalogue/one-piece/scan/vf/2", BASE_URL)),
					..Default::default()
				},
				Chapter {
					key: "3".into(),
					title: Some("Entrez : Pirate Hunter Roronoa Zoro".into()),
					chapter_number: Some(3.0),
					volume_number: Some(1.0),
					date_uploaded: Some(1640995200),
					scanlators: Some(vec!["AnimeSama".into()]),
					url: Some(format!("{}/catalogue/one-piece/scan/vf/3", BASE_URL)),
					..Default::default()
				},
			]);
		}

		Ok(manga)
	}

	fn get_page_list(&self, _manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Test avec des pages de One Piece chapitre 1
		let chapter_num = chapter.key.parse::<i32>().unwrap_or(1);
		let mut pages = Vec::new();

		// Générer 5 pages de test
		for i in 1..=5 {
			let page_url = format!("{}/s2/scans/one_piece/{}/{:03}.jpg", CDN_URL, chapter_num, i);
			pages.push(Page {
				content: PageContent::url(page_url),
				has_description: false,
				description: None,
				thumbnail: None,
			});
		}

		Ok(pages)
	}
}

register_source!(AnimeSama);