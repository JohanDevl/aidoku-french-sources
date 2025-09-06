#![no_std]

use aidoku::{
	Chapter, FilterValue, Listing, ListingProvider, Manga, MangaPageResult, 
	Page, Result, Source,
	alloc::{String, Vec, string::ToString},
	imports::net::Request,
	prelude::*,
};

mod parser;
mod helper;

pub const BASE_URL: &str = "https://phenix-scans.com";
pub const API_URL: &str = "https://phenix-scans.com/api";

struct PhenixScans;

impl Source for PhenixScans {
	fn new() -> Self {
		Self
	}

	fn get_search_manga_list(
		&self,
		query: Option<String>,
		page: i32,
		filters: Vec<FilterValue>,
	) -> Result<MangaPageResult> {
		// Process filters to build API query parameters
		let mut query_params = String::new();
		let mut genre_ids: Vec<String> = Vec::new();
		
		for filter in filters {
			match filter {
				FilterValue::Select { id, value } => {
					match id.as_str() {
						"status" => {
							match value.as_str() {
								"En cours" => query_params.push_str("&status=ongoing"),
								"Terminé" => query_params.push_str("&status=completed"),
								"En pause" => query_params.push_str("&status=hiatus"),
								_ => continue,
							}
						}
						"type" => {
							match value.as_str() {
								"Manga" => query_params.push_str("&type=Manga"),
								"Manhwa" => query_params.push_str("&type=Manhwa"),
								"Manhua" => query_params.push_str("&type=Manhua"),
								_ => continue,
							}
						}
						"Genres" => {
							// Genre selection - map genre names to their IDs
							let genre_id = match value.as_str() {
								"Action" => "67bd1aefae810159afa2675e",
								"Aventure" => "67bd1aefae810159afa2675f",
								"Combat" => "67bd1aefae810159afa26760",
								"Comedie" => "67bd1aefae810159afa26761",
								"Fantaisie" => "67bd1aefae810159afa26762",
								"Isekai" => "67bd1aefae810159afa26763",
								"Arts-martiaux" => "67bd1b03ae810159afa26779",
								"Fantastique" => "67bd1b03ae810159afa2677a",
								"Historique" => "67bd1b03ae810159afa2677b",
								"Réincarnation" => "67bd1b03ae810159afa2677c",
								"Adventure" => "67bd1ba0ae810159afa267e7",
								"Fantasy" => "67bd1ba0ae810159afa267e8",
								"School Life" => "67bd1ba0ae810159afa267e9",
								"Shounen" => "67bd1ba0ae810159afa267ea",
								"Drama" => "67bd1ca0ae810159afa268f7",
								"Partenaire" => "67bd1ca0ae810159afa268f8",
								"Romance" => "67bd1ca0ae810159afa268f9",
								"Shoujo" => "67bd1ca0ae810159afa268fa",
								"Drame" => "67bd1cd6ae810159afa2692f",
								"Supernatural" => "67bd1dffae810159afa26a3b",
								"Mature" => "67bd1f24ae810159afa26b6d",
								"Tragedy" => "67bd1f24ae810159afa26b6e",
								"Martial Arts" => "67bd1f53ae810159afa26b8d",
								"Seinen" => "67bd1f53ae810159afa26b8e",
								"Harem" => "67bd203cae810159afa26c80",
								"Murim" => "67bd203cae810159afa26c81",
								"Régression" => "67bd203cae810159afa26c82",
								"Vengeance" => "67bd203cae810159afa26c83",
								"Historical" => "67bd211bae810159afa26d9d",
								"Comedy" => "67bd21a8ae810159afa26dfc",
								"Slice of Life" => "67bd21a8ae810159afa26dfd",
								"Horreur" => "67bd225aae810159afa26eb9",
								"Ecchi" => "67bd22e6ae810159afa26f36",
								"Magie" => "67bd22e6ae810159afa26f37",
								"Monstre" => "67bd22e6ae810159afa26f38",
								"Dragon" => "67bd260cae810159afa27194",
								"Josei" => "67bd2668ae810159afa271dd",
								"Psychological" => "67bd2668ae810159afa271de",
								"Sci-fi" => "67bd26e8ae810159afa27249",
								"Psychologique" => "67bd272bae810159afa2727c",
								"Tragédie" => "67bd272bae810159afa2727d",
								"Mystère" => "67bd2de7ae810159afa27712",
								"Demons" => "67bd2de7ae810159afa27713",
								"Surnaturel" => "67bd2deeae810159afa27719",
								"Système" => "67bd2e07ae810159afa27728",
								"Combats" => "67bd2e28ae810159afa27743",
								"Mystery" => "67bd2e6dae810159afa27779",
								"Donjon" => "67bd2f7fae810159afa2785e",
								"Necromancy" => "67bd2f80ae810159afa27860",
								"Adult" => "67bd335bae810159afa27aa6",
								"Thriller" => "67bd3374ae810159afa27abc",
								"Aventurier" => "67bd3889ae810159afa27ee9",
								"Manhwa" => "67bd3a5dae810159afa2805d",
								"Webtoons" => "67bd3a5dae810159afa2805e",
								"Amitier" => "67bd3a5dae810159afa2805f",
								"Smut" => "67bd3c38ae810159afa281f2",
								"Manga" => "67bd3c38ae810159afa281f3",
								"Vie Scolaire" => "67bd43a9ae810159afa287cf",
								"Tranche de vie" => "67bd43c0ae810159afa287e1",
								"Magic" => "67bd44c8ae810159afa288e4",
								"R18" => "67bd4eaaae810159afa28fd4",
								"Académie" => "67bd4eaaae810159afa28fd5",
								"Academy" => "67bd5719ae810159afa2960f",
								"Heroes" => "67bd57f2ae810159afa296d0",
								"Post apocalyptique" => "67bd680eae810159afa2a430",
								"Regresseur" => "67bd68daae810159afa2a4df",
								"Jeu" => "67bd6c16ae810159afa2a6d3",
								"Transmigration" => "67bd708bae810159afa2aa74",
								"Amitié" => "67bd7373ae810159afa2ad11",
								"Sports" => "67bd8859ae810159afa2bf6a",
								_ => "",
							};
							
							if !genre_id.is_empty() && value != "Tout" {
								genre_ids.push(genre_id.to_string());
							}
						}
						_ => continue,
					}
				}
				_ => continue,
			}
		}

		// Build final URL based on search query and filters
		if let Some(search_query) = query {
			// Search endpoint with query
			let url = format!("{}/front/manga/search?query={}", API_URL, helper::urlencode(&search_query));
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Accept-Encoding", "gzip, deflate, br")
				.header("Referer", "https://phenix-scans.com/")
				.header("Origin", "https://phenix-scans.com")
				.string()?;
			
			parser::parse_search_list(response)
		} else {
			// Listing/filtering endpoint with parameters
			let genre_param = if genre_ids.is_empty() {
				String::new()
			} else {
				format!("&genre={}", genre_ids.join(","))
			};
			
			let url = format!("{}/front/manga?page={}&limit=20{}{}", 
				API_URL, page, query_params, genre_param);
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Accept-Encoding", "gzip, deflate, br")
				.header("Referer", "https://phenix-scans.com/")
				.header("Origin", "https://phenix-scans.com")
				.string()?;
			
			parser::parse_manga_list(response)
		}
	}

	fn get_manga_update(
		&self,
		mut manga: Manga,
		needs_details: bool,
		needs_chapters: bool,
	) -> Result<Manga> {
		if needs_details || needs_chapters {
			// Utiliser le vrai endpoint API avec headers Cloudflare
			let url = format!("{}/front/manga/{}", API_URL, manga.key);
			let response = Request::get(&url)?
				.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
				.header("Accept", "application/json, text/plain, */*")
				.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
				.header("Referer", "https://phenix-scans.com/")
				.string()?;
			
			if needs_details {
				let detailed_manga = parser::parse_manga_details(manga.key.clone(), response.clone())?;
				manga.title = detailed_manga.title;
				manga.authors = detailed_manga.authors;
				manga.artists = detailed_manga.artists;
				manga.description = detailed_manga.description;
				manga.url = detailed_manga.url;
				manga.cover = detailed_manga.cover;
				manga.tags = detailed_manga.tags;
				manga.status = detailed_manga.status;
				manga.content_rating = detailed_manga.content_rating;
				manga.viewer = detailed_manga.viewer;
			}

			if needs_chapters {
				let chapters = parser::parse_chapter_list(manga.key.clone(), response)?;
				manga.chapters = Some(chapters);
			}
		}

		Ok(manga)
	}

	fn get_page_list(&self, manga: Manga, chapter: Chapter) -> Result<Vec<Page>> {
		// Utiliser le vrai endpoint API pour les pages avec headers Cloudflare
		let url = format!("{}/front/manga/{}/chapter/{}", API_URL, manga.key, chapter.key);
		let response = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
			.header("Accept", "application/json, text/plain, */*")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Referer", "https://phenix-scans.com/")
			.string()?;
		parser::parse_page_list(response)
	}
}

impl ListingProvider for PhenixScans {
	fn get_manga_list(
		&self,
		listing: Listing,
		page: i32,
	) -> Result<MangaPageResult> {
		// Utiliser les vrais endpoints API homepage
		let url = if listing.name == "Dernières Sorties" {
			format!("{}/front/homepage?page={}&section=latest&limit=20", API_URL, page)
		} else if listing.name == "Populaire" {
			format!("{}/front/homepage?section=top", API_URL)
		} else {
			return Err(aidoku::AidokuError::message("Unimplemented listing"));
		};
		
		// Faire la requête API JSON avec headers Cloudflare
		let response = Request::get(&url)?
			.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
			.header("Accept", "application/json, text/plain, */*")
			.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
			.header("Referer", "https://phenix-scans.com/")
			.string()?;
		
		parser::parse_manga_listing(response, &listing.name)
	}
}

register_source!(PhenixScans, ListingProvider);