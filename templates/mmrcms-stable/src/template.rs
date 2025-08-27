use aidoku_stable_wrapper::{
    prelude::*,
};

use crate::helper::{append_protocol, extract_f32_from_string};

// Simplified cache without Mutex since we don't have async in stable
static mut CACHED_MANGA_HTML: Option<String> = None;
static mut CACHED_MANGA_ID: Option<String> = None;

/// Internal attribute to control if the source should fall
/// back to self searching after failing to use the search
/// engine first time.
static mut INTERNAL_USE_SEARCH_ENGINE: bool = true;

pub fn cache_manga_page(url: &str) {
    unsafe {
        if CACHED_MANGA_HTML.is_some() && CACHED_MANGA_ID.as_ref() == Some(&url.to_string()) {
            return;
        }
    }

    if let Ok(html) = Request::new(url, HttpMethod::Get).html() {
        // Simplified - no cfemail decode
        unsafe {
            CACHED_MANGA_HTML = Some(html.html().read());
            CACHED_MANGA_ID = Some(String::from(url));
        }
    }
}

pub struct MMRCMSSource<'a> {
    pub base_url: &'a str,
    pub lang: &'a str,
    /// {base_url}/{manga_path}/{manga_id}
    pub manga_path: &'a str,

    /// Localization
    pub category: &'a str,
    pub tags: &'a str,

    pub category_parser: fn(&NodeSelection, Vec<String>) -> (MangaContentRating, MangaViewer),
    pub category_mapper: fn(i32) -> String,
    pub tags_mapper: fn(i32) -> String,

    pub use_search_engine: bool,
}

#[derive(Default)]
struct MMRCMSSearchResult {
    pub data: String,
    pub value: String,
}

impl<'a> Default for MMRCMSSource<'a> {
    fn default() -> Self {
        MMRCMSSource {
            base_url: "",
            lang: "en",
            manga_path: "manga",

            category: "Category",
            tags: "Tag",

            category_parser: |_, categories| {
                let mut nsfw = MangaContentRating::Safe;
                let mut viewer = MangaViewer::Rtl;
                for category in categories {
                    match category.as_str() {
                        "Adult" | "Smut" | "Mature" | "18+" | "Hentai" => {
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
                if idx != 0 {
                    format!("{}", idx)
                } else {
                    String::new()
                }
            },
            tags_mapper: |_| String::new(),
            use_search_engine: true,
        }
    }
}

impl<'a> MMRCMSSource<'a> {
    fn guess_cover(&self, url: &str, id: &str) -> String {
        if url.ends_with("no-image.png") || url.is_empty() {
            format!(
                "{base_url}/uploads/manga/{id}/cover/cover_250x350.jpg",
                base_url = self.base_url
            )
        } else {
            append_protocol(String::from(url))
        }
    }

    fn self_search<T: AsRef<str>>(&self, query: T) -> Result<MangaPageResult> {
        let query = query.as_ref();
        let html = Request::new(
            &format!("{}/changeMangaList?type=text", self.base_url),
            HttpMethod::Get,
        )
        .html()?;
        
        let mut manga = Vec::new();
        for elem in html.select("ul.manga-list a").array() {
            let node = elem.as_node().expect("Failed to get as node");
            let title = node.text().read();
            if title.to_lowercase().contains(&query.to_lowercase()) {
                let url = node.attr("href").read();
                let id = url
                    .split('/')
                    .last()
                    .map(String::from)
                    .unwrap_or_else(|| url.replace(&format!("{}/{}", self.base_url, self.manga_path), ""));
                let cover = self.guess_cover("", &id);
                manga.push(Manga {
                    id,
                    title,
                    cover: Some(cover),
                    url: Some(url),
                    ..Default::default()
                });
            }
        }

        Ok(MangaPageResult {
            manga,
            has_more: false,
        })
    }

    pub fn get_manga_list(&self, filters: Vec<Filter>, page: i32) -> Result<MangaPageResult> {
        let mut query: Vec<String> = Vec::new();
        let mut title = String::new();
        for filter in filters {
            match filter {
                Filter::Text { id, value } => {
                    if id == "title" {
                        if value.is_empty() {
                            continue;
                        }
                        title = urlencode(value);
                        break;
                    }
                }
                Filter::Text { id, value } if id == "author" => {
                    query.push(format!("artist={}", urlencode(value)));
                }
                Filter::Sort { value, ascending, .. } => {
                    let sort_type = match value.as_str() {
                        "name" => "name",
                        "views" => "views", 
                        "last_release" => "last_release",
                        _ => continue,
                    };
                    query.push(format!("sortBy={}", sort_type));
                    query.push(format!("asc={}", ascending));
                }
                Filter::Select { id, value } => {
                    if id == self.category {
                        query.push(format!("cat={}", (self.category_mapper)(value)));
                    } else if id == self.tags {
                        query.push(format!("tag={}", (self.tags_mapper)(value)));
                    }
                }
                _ => continue,
            }
        }

        if !title.is_empty() {
            unsafe {
                if self.use_search_engine && INTERNAL_USE_SEARCH_ENGINE {
                    let url = format!("{}/search?query={}", self.base_url, title);
                    // Simplified search - no JSON parsing, fall back to self search
                    INTERNAL_USE_SEARCH_ENGINE = false;
                    self.self_search(title)
                } else {
                    self.self_search(title)
                }
            }
        } else {
            let url = format!(
                "{}/filterList?page={}&{}",
                self.base_url,
                page,
                query.join("&")
            );
            let html = Request::new(&url, HttpMethod::Get).html()?;
            let node = html.select("div[class^=col-sm-]");
            let elems = node.array();
            let mut manga = Vec::with_capacity(elems.len());
            let has_more: bool = !elems.is_empty();

            for elem in elems {
                let manga_node = elem.as_node().expect("Failed to get as node");
                let url = manga_node
                    .select(&format!("a[href*='{}/{}']", self.base_url, self.manga_path))
                    .attr("href")
                    .read();
                let id = url.replace(&format!("{}/{}/", self.base_url, self.manga_path), "");
                let cover = self.guess_cover(
                    &manga_node
                        .select(&format!(
                            "a[href*='{}/{}'] img",
                            self.base_url, self.manga_path
                        ))
                        .attr("src")
                        .read(),
                    &id,
                );
                let title = manga_node.select("a.chart-title strong").text().read();
                manga.push(Manga {
                    id: id.clone(),
                    title,
                    cover: Some(cover),
                    url: Some(url),
                    ..Default::default()
                });
            }
            Ok(MangaPageResult { manga, has_more })
        }
    }

    pub fn get_manga_details(&self, id: String) -> Result<Manga> {
        let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
        cache_manga_page(&url);
        
        let html_content = unsafe { CACHED_MANGA_HTML.clone().unwrap_or_default() };
        let html = Node::new(&html_content);
        
        let cover = append_protocol(html.select("img[class^=img-]").attr("src").read());
        let title = html
            .select("h2.widget-title, h1.widget-title, .listmanga-header, div.panel-heading")
            .first()
            .text()
            .read();
        let description = html.select(".row .well p").text().read();
        
        let mut manga = Manga {
            id,
            title,
            cover: Some(cover),
            description: if description.is_empty() { None } else { Some(description) },
            url: Some(url),
            ..Default::default()
        };

        let mut categories = Vec::new();
        for elem in html.select(".row .dl-horizontal dt").array() {
            let node = elem.as_node().expect("Failed to get as node");
            let text = node.text().read().to_lowercase();
            
            // Simplified parsing without substring helpers
            if text.contains("author") || text.contains("autor") {
                // Get next sibling - simplified
                manga.author = Some(node.text().read());
            } else if text.contains("artist") {
                manga.artist = Some(node.text().read());
            } else if text.contains("categories") || text.contains("genre") {
                for cat_elem in node.select("a").array() {
                    let cat_node = cat_elem.as_node().expect("Failed to get category node");
                    categories.push(cat_node.text().read());
                }
            } else if text.contains("status") {
                let status_text = node.text().read().to_lowercase();
                manga.status = match status_text.trim() {
                    s if s.contains("complete") => MangaStatus::Completed,
                    s if s.contains("ongoing") => MangaStatus::Ongoing,
                    s if s.contains("hiatus") => MangaStatus::Hiatus,
                    s if s.contains("cancel") => MangaStatus::Cancelled,
                    _ => MangaStatus::Unknown,
                };
            }
        }
        
        categories.sort();
        manga.categories = Some(categories.clone());
        (manga.nsfw, manga.viewer) = (self.category_parser)(&html.select(""), categories);
        
        // Check for NSFW warning
        if !html.select("div.alert.alert-danger").array().is_empty() {
            manga.nsfw = MangaContentRating::Nsfw;
        }
        
        Ok(manga)
    }

    pub fn get_chapter_list(&self, id: String) -> Result<Vec<Chapter>> {
        let url = format!("{}/{}/{}", self.base_url, self.manga_path, id);
        cache_manga_page(&url);
        
        let html_content = unsafe { CACHED_MANGA_HTML.clone().unwrap_or_default() };
        let html = Node::new(&html_content);
        
        let node = html.select("li:has(.chapter-title-rtl)");
        let elems = node.array();
        let title = html
            .select("h2.widget-title, h1.widget-title, .listmanga-header, div.panel-heading")
            .first()
            .text()
            .read();
        let should_extract_chapter_title = html.select("em").array().is_empty();
        
        let mut chapters = Vec::new();
        for elem in elems {
            let chapter_node = elem.as_node().expect("Failed to get chapter node");
            let url = chapter_node.select("a").attr("href").read();

            if let Some(chapter_id) = url.split('/').nth(5).map(String::from) {
                let volume = extract_f32_from_string(
                    String::from("volume-"),
                    chapter_node.attr("class").read(),
                );
                let chapter_title = chapter_node.select("a").first().text().read();

                let chapter = extract_f32_from_string(title.clone(), chapter_title.clone());
                let mut title = chapter_node.select("em").text().read();
                if title.is_empty() && should_extract_chapter_title {
                    title = chapter_title;
                }

                let date_updated = chapter_node
                    .select("div.date-chapter-title-rtl, div.col-md-4")
                    .first()
                    .text()
                    .as_date("dd MMM yyyy", Some("en_US"), None)
                    .unwrap_or(-1.0);

                chapters.push(Chapter {
                    id: chapter_id,
                    title: if title.is_empty() { None } else { Some(title) },
                    volume: if volume < 0.0 { None } else { Some(volume) },
                    chapter: if chapter < 0.0 { None } else { Some(chapter) },
                    date_updated: if date_updated < 0.0 { None } else { Some(date_updated) },
                    url: Some(url),
                    lang: String::from(self.lang),
                    ..Default::default()
                });
            }
        }
        Ok(chapters)
    }

    pub fn get_page_list(&self, manga_id: String, id: String) -> Result<Vec<Page>> {
        let url = format!("{}/{}/{}/{}", self.base_url, self.manga_path, manga_id, id);
        let html = Request::new(&url, HttpMethod::Get).html()?;
        
        // Simplified - no JSON parsing, look for images directly
        let mut pages = Vec::new();
        for (idx, elem) in html.select("img").array().into_iter().enumerate() {
            let node = elem.as_node().expect("Failed to get image node");
            let img_url = node.attr("src").read();
            if !img_url.is_empty() {
                let url = if img_url.starts_with("http") {
                    img_url
                } else {
                    format!("{}/uploads/manga/{}/chapters/{}/{}", 
                            self.base_url, manga_id, id, img_url)
                };
                pages.push(Page {
                    content: PageContent::url(url),
                    has_description: false,
                    description: None,
                });
            }
        }
        Ok(pages)
    }

    pub fn modify_image_request(&self, request: Request) {
        request.header("Referer", self.base_url);
    }

    pub fn handle_url(&self, url: String) -> Result<DeepLink> {
        let mut split = url.split('/');
        if let Some(id) = split.nth(4).map(String::from) {
            let manga = Some(self.get_manga_details(id)?);
            let chapter = split.next().map(String::from).map(|id| Chapter {
                id,
                ..Default::default()
            });
            Ok(DeepLink { manga, chapter })
        } else {
            Err(AidokuError::new("Failed to parse URL"))
        }
    }
}

// Helper function for URL encoding
fn urlencode(input: String) -> String {
    // Simplified URL encoding
    input.replace(' ', "%20")
         .replace('&', "%26")
         .replace('=', "%3D")
}