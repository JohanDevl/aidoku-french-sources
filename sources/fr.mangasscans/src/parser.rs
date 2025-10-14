use aidoku::{
    Chapter, ContentRating, Manga, MangaStatus, Page, PageContent,
    Result, UpdateStrategy, Viewer,
    alloc::{String, Vec, string::ToString, format},
    imports::html::Document,
};

use crate::helper::{extract_chapter_number, make_absolute_url, parse_status};

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

pub fn parse_manga_list(html: &Document, base_url: &str) -> Vec<Manga> {
    let mut mangas = Vec::new();

    let manga_selectors = [
        ".listupd .bs",
        ".listupd .bsx",
        ".utao .uta .imgu",
        ".page-item-detail",
    ];

    for selector in &manga_selectors {
        if let Some(items) = html.select(selector) {
            if !items.is_empty() {
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

                    let href = link.attr("href").unwrap_or_default();
                    let key = href
                        .trim_start_matches("https://")
                        .trim_start_matches("http://")
                        .trim_start_matches("mangas-scans.com/")
                        .trim_start_matches("manga/")
                        .trim_end_matches('/')
                        .to_string();

                    let title = link
                        .attr("title")
                        .or_else(|| {
                            item.select("h3 a, h5 a, .tt, .entry-title")
                                .and_then(|els| els.first())
                                .and_then(|el| el.text())
                        })
                        .unwrap_or_default();

                    let cover = item
                        .select("img")
                        .and_then(|imgs| imgs.first())
                        .and_then(|img| {
                            img.attr("data-src")
                                .or_else(|| img.attr("data-lazy-src"))
                                .or_else(|| img.attr("src"))
                        })
                        .map(|url| make_absolute_url(base_url, &url));

                    if !key.is_empty() && !title.is_empty() {
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
                            url: Some(make_absolute_url(base_url, &href)),
                            next_update_time: None,
                            update_strategy: UpdateStrategy::Always,
                        });
                    }
                }
                break;
            }
        }
    }

    mangas
}

pub fn has_next_page(html: &Document) -> bool {
    // Check for specific "next" link selectors
    if html.select(".pagination .next").is_some() {
        return true;
    }

    if html.select(".nextpostslink").is_some() {
        return true;
    }

    if html.select(".nav-links a[rel='next']").is_some() {
        return true;
    }

    if html.select(".hpage a.r").is_some() {
        return true;
    }

    // Check for pagination links with "next" or "›" text
    if let Some(links) = html.select(".pagination a, .nav-links a") {
        for link in links {
            if let Some(text) = link.text() {
                let text_lower = text.to_lowercase();
                if text_lower.contains("next")
                    || text_lower.contains("suivant")
                    || text_lower.contains("›")
                    || text_lower.contains("»") {
                    return true;
                }
            }

            // Check for rel="next" attribute
            if let Some(rel) = link.attr("rel") {
                if rel == "next" {
                    return true;
                }
            }
        }
    }

    false
}

pub fn parse_manga_details(html: &Document, base_url: &str, manga_key: String) -> Result<Manga> {
    let title_selectors = [
        "h1.entry-title",
        ".wp-manga-title",
        ".post-title h1",
        ".ts-breadcrumb li:last-child span",
        "h1",
    ];

    let mut title = String::new();
    for selector in &title_selectors {
        if let Some(elems) = html.select(selector) {
            if let Some(elem) = elems.first() {
                if let Some(text) = elem.text() {
                    title = text;
                    break;
                }
            }
        }
    }

    let cover_selectors = [
        ".infomanga > div[itemprop=image] img",
        ".thumb img",
        ".wp-post-image",
        ".manga-poster img",
        ".summary_image img",
        "div.bigcontent img",
        ".series-thumb img",
        "img[itemprop=image]",
    ];

    let mut cover = None;
    for selector in &cover_selectors {
        if let Some(imgs) = html.select(selector) {
            if let Some(img) = imgs.first() {
                let img_url = img
                    .attr("data-src")
                    .or_else(|| img.attr("data-lazy-src"))
                    .or_else(|| img.attr("src"));

                if let Some(url) = img_url {
                    let absolute = make_absolute_url(base_url, &url);
                    if !absolute.is_empty() {
                        cover = Some(absolute);
                        break;
                    }
                }
            }
        }
    }

    let description_selectors = [
        ".desc",
        ".entry-content[itemprop=description]",
        ".summary__content",
        "div[itemprop=description]",
        ".manga-excerpt",
    ];

    let mut description = None;
    for selector in &description_selectors {
        if let Some(elems) = html.select(selector) {
            if let Some(elem) = elems.first() {
                if let Some(text) = elem.text() {
                    let trimmed = text.trim().to_string();
                    if !trimmed.is_empty() {
                        description = Some(trimmed);
                        break;
                    }
                }
            }
        }
    }

    let mut authors = None;

    // First try: parse from .infotable by finding the Auteur/Author row
    if let Some(rows) = html.select(".infotable tr") {
        for row in rows {
            if let Some(cells) = row.select("td") {
                let cells_vec: Vec<_> = cells.collect();
                if cells_vec.len() >= 2 {
                    if let Some(label) = cells_vec[0].text() {
                        let label_lower = label.to_lowercase();
                        if label_lower.contains("auteur") || label_lower.contains("author") {
                            if let Some(value) = cells_vec[1].text() {
                                let trimmed = value.trim().to_string();
                                if !trimmed.is_empty() {
                                    authors = Some(trimmed);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to other selectors if not found
    if authors.is_none() {
        let author_selectors = [
            ".tsinfo .imptdt:contains(Auteur) i",
            ".author-content a",
            ".fmed:contains(Auteur) span",
        ];

        for selector in &author_selectors {
            if let Some(elems) = html.select(selector) {
                if let Some(elem) = elems.first() {
                    if let Some(text) = elem.text() {
                        let trimmed = text.trim().to_string();
                        if !trimmed.is_empty() {
                            authors = Some(trimmed);
                            break;
                        }
                    }
                }
            }
        }
    }

    let mut status = MangaStatus::Unknown;

    // First try: parse from .infotable by finding the Status/Statut row
    if let Some(rows) = html.select(".infotable tr") {
        for row in rows {
            if let Some(cells) = row.select("td") {
                let cells_vec: Vec<_> = cells.collect();
                if cells_vec.len() >= 2 {
                    if let Some(label) = cells_vec[0].text() {
                        let label_lower = label.to_lowercase();
                        if label_lower.contains("status") || label_lower.contains("statut") {
                            if let Some(value) = cells_vec[1].text() {
                                status = parse_status(&value);
                                if status != MangaStatus::Unknown {
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Fallback to other selectors if not found
    if status == MangaStatus::Unknown {
        let status_selectors = [
            ".tsinfo .imptdt:contains(Statut) i",
            ".status",
            ".fmed:contains(Status) span",
        ];

        for selector in &status_selectors {
            if let Some(elems) = html.select(selector) {
                if let Some(elem) = elems.first() {
                    if let Some(text) = elem.text() {
                        status = parse_status(&text);
                        if status != MangaStatus::Unknown {
                            break;
                        }
                    }
                }
            }
        }
    }

    let genre_selectors = [
        "div.gnr a",
        ".mgen a",
        ".seriestugenre a",
        ".genres a",
        "a[rel=tag]",
    ];

    let mut tags: Vec<String> = Vec::new();
    for selector in &genre_selectors {
        if let Some(links) = html.select(selector) {
            for link in links {
                if let Some(genre_text) = link.text() {
                    let genre = genre_text.trim().to_string();
                    if !genre.is_empty() && !tags.contains(&genre) {
                        tags.push(genre);
                    }
                }
            }
            if !tags.is_empty() {
                break;
            }
        }
    }

    let manga_url = make_absolute_url(base_url, &format!("/manga/{}/", manga_key));

    let tags_opt = if !tags.is_empty() { Some(tags) } else { None };
    let content_rating = calculate_content_rating(&tags_opt);
    let viewer = calculate_viewer(&tags_opt);

    Ok(Manga {
        key: manga_key.clone(),
        cover,
        title,
        authors: authors.map(|a| alloc::vec![a]),
        artists: None,
        description,
        tags: tags_opt,
        status,
        content_rating,
        viewer,
        chapters: None,
        url: Some(manga_url),
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    })
}

pub fn parse_chapter_list(html: &Document, base_url: &str) -> Vec<Chapter> {
    let mut chapters: Vec<Chapter> = Vec::new();

    let chapter_selectors = [
        "div.eplister ul li",
        "div.bxcl li",
        "div.cl li",
        "#chapterlist li",
        ".wp-manga-chapter",
        "li.wp-manga-chapter",
        ".chapter-list li",
        ".listing-chapters_wrap li",
    ];

    for selector in &chapter_selectors {
        if let Some(items) = html.select(selector) {
            if !items.is_empty() {
                for item in items {
                    let link_selectors = ["div.eph-num a", ".eph-num a", "a"];
                    let mut link = None;

                    for link_selector in &link_selectors {
                        if let Some(links) = item.select(link_selector) {
                            if let Some(l) = links.first() {
                                link = Some(l);
                                break;
                            }
                        }
                    }

                    if let Some(link) = link {
                        let href = link.attr("href").unwrap_or_default();
                        let key = href
                            .trim_start_matches("https://")
                            .trim_start_matches("http://")
                            .trim_start_matches("mangas-scans.com/")
                            .trim_end_matches('/')
                            .to_string();

                        let title_text = item
                            .select(".chapternum, .chapter-title")
                            .and_then(|els| els.first())
                            .and_then(|el| el.text())
                            .unwrap_or_else(|| link.text().unwrap_or_default())
                            .trim()
                            .to_string();

                        let date_selectors = [
                            ".chapterdate",
                            "span.chapterdate",
                            ".eph-num .chapterdate",
                            ".epl-num .chapterdate",
                            "time",
                            ".dt",
                        ];

                        let mut date_uploaded = None;
                        for date_selector in &date_selectors {
                            if let Some(date_els) = item.select(date_selector) {
                                if let Some(date_el) = date_els.first() {
                                    if let Some(date_text) = date_el.text() {
                                        use crate::helper::parse_chapter_date;
                                        if let Some(timestamp) = parse_chapter_date(&date_text) {
                                            date_uploaded = Some(timestamp);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        let chapter_num = extract_chapter_number(&title_text);

                        if !key.is_empty() {
                            let chapter_url = make_absolute_url(base_url, &format!("/{}/", key));

                            chapters.push(Chapter {
                                key: key.clone(),
                                title: if !title_text.is_empty() {
                                    Some(title_text)
                                } else {
                                    None
                                },
                                date_uploaded,
                                url: Some(chapter_url),
                                chapter_number: if chapter_num > 0.0 {
                                    Some(chapter_num)
                                } else {
                                    None
                                },
                                volume_number: None,
                                scanlators: None,
                                language: None,
                                thumbnail: None,
                                locked: false,
                            });
                        }
                    }
                }
                break;
            }
        }
    }

    chapters
}

pub fn parse_page_list(html: &Document, base_url: &str) -> Vec<Page> {
    let mut pages: Vec<Page> = Vec::new();

    let selectors = [
        "div#readerarea img",
        ".rdminimal img",
        ".reader-area img",
        "#chapter_imgs img",
        ".chapter-content img",
    ];

    for selector in &selectors {
        if let Some(imgs) = html.select(selector) {
            if !imgs.is_empty() {
                for img in imgs {
                    let img_url = img
                        .attr("data-src")
                        .or_else(|| img.attr("data-lazy-src"))
                        .or_else(|| img.attr("data-cfsrc"))
                        .or_else(|| img.attr("src"));

                    if let Some(url) = img_url {
                        let absolute_url = make_absolute_url(base_url, &url);
                        if !absolute_url.is_empty()
                            && !absolute_url.contains("loading")
                            && !absolute_url.contains("spinner")
                        {
                            pages.push(Page {
                                content: PageContent::Url(absolute_url, None),
                                thumbnail: None,
                                has_description: false,
                                description: None,
                            });
                        }
                    }
                }
                break;
            }
        }
    }

    pages
}
