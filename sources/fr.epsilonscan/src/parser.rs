use aidoku::{
    Chapter, Manga, ContentRating, MangaPageResult, MangaStatus, Viewer, UpdateStrategy,
    Page, PageContent, Result, AidokuError,
    alloc::{String, Vec, vec, format, string::ToString},
    imports::html::Document,
};

use crate::{BASE_URL, helper};

pub fn parse_manga_list(html: Document) -> Result<MangaPageResult> {
    let mut entries: Vec<Manga> = Vec::new();

    if let Some(items) = html.select("div.page-item-detail") {
        for item in items {
            if let Some(_novel) = item.select(".web-novel") {
                if !_novel.text().unwrap_or_default().is_empty() {
                    continue;
                }
            }

            let href = item.select("h3.h5 > a")
                .and_then(|nodes| nodes.first())
                .and_then(|node| node.attr("href"))
                .unwrap_or_default();

            let key = href
                .replace(BASE_URL, "")
                .replace("/manga/", "")
                .replace('/', "")
                .trim()
                .to_string();

            let title_badges = item.select("span.manga-title-badges")
                .and_then(|nodes| nodes.first())
                .map(|node| node.text().unwrap_or_default())
                .unwrap_or_default();

            let mut title = item.select("h3.h5 > a")
                .and_then(|nodes| nodes.first())
                .map(|node| node.text().unwrap_or_default())
                .unwrap_or_default();

            if title.contains(&title_badges) {
                title = title.replace(&title_badges, "");
                title = String::from(title.trim());
            }

            let cover = item.select("img")
                .and_then(|nodes| nodes.first())
                .map(|node| helper::get_image_url(node))
                .filter(|url| !url.is_empty());

            entries.push(Manga {
                key,
                title,
                cover,
                authors: None,
                artists: None,
                description: None,
                url: None,
                tags: None,
                status: MangaStatus::Unknown,
                content_rating: ContentRating::Safe,
                viewer: Viewer::LeftToRight,
                chapters: None,
                next_update_time: None,
                update_strategy: UpdateStrategy::Always,
            });
        }
    }

    let has_next_page = !entries.is_empty();

    Ok(MangaPageResult { entries, has_next_page })
}

pub fn parse_manga_details(manga_id: &str, html: &Document) -> Result<Manga> {
    let title_badges = html.select("span.manga-title-badges")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    let mut title = html.select("div.post-title h1")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    if title.contains(&title_badges) {
        title = title.replace(&title_badges, "");
        title = String::from(title.trim());
    }

    let cover = html.select("div.summary_image img")
        .and_then(|nodes| nodes.first())
        .map(|node| helper::get_image_url(node))
        .filter(|url| !url.is_empty());

    let author = html.select("div.author-content a")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    let artist = html.select("div.artist-content a")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    let description = html.select("div.description-summary div.summary__content p")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    let mut tags: Vec<String> = Vec::new();
    if let Some(genre_links) = html.select("div.genres-content a") {
        for genre_link in genre_links {
            let genre = genre_link.text().unwrap_or_default();
            if !genre.is_empty() {
                tags.push(genre);
            }
        }
    }

    let status_str = html
        .select("div.post-content_item:contains(Statut) div.summary-content")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default()
        .trim()
        .to_lowercase();

    let status = match status_str.as_str() {
        "en cours" | "ongoing" => MangaStatus::Ongoing,
        "terminé" | "completed" => MangaStatus::Completed,
        "annulé" | "cancelled" | "canceled" => MangaStatus::Cancelled,
        "en pause" | "hiatus" | "on hold" => MangaStatus::Hiatus,
        _ => MangaStatus::Unknown,
    };

    let series_type = html
        .select("div.post-content_item:contains(Type) div.summary-content")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default()
        .to_lowercase();

    let viewer = if series_type.contains("manhwa")
        || series_type.contains("manhua")
        || series_type.contains("webtoon") {
        Viewer::Scroll
    } else if series_type.contains("manga") {
        Viewer::Rtl
    } else {
        Viewer::Scroll
    };

    let nsfw_tags = ["adult", "mature", "pornhwa", "smut", "ecchi"];
    let mut content_rating = ContentRating::Safe;

    let adult_badge = html.select(".manga-title-badges.adult")
        .and_then(|nodes| nodes.first())
        .map(|node| node.text().unwrap_or_default())
        .unwrap_or_default();

    if !adult_badge.is_empty() {
        content_rating = ContentRating::Nsfw;
    } else {
        for tag in nsfw_tags {
            if tags.iter().any(|v| v.to_lowercase().contains(tag)) {
                content_rating = if tag == "ecchi" {
                    ContentRating::Suggestive
                } else {
                    ContentRating::Nsfw
                };
                break;
            }
        }
    }

    let authors = if author.is_empty() { None } else { Some(vec![author]) };
    let artists = if artist.is_empty() { None } else { Some(vec![artist]) };

    Ok(Manga {
        key: manga_id.to_string(),
        title,
        cover,
        authors,
        artists,
        description: if description.is_empty() { None } else { Some(description) },
        url: Some(format!("{}/manga/{}/", BASE_URL, manga_id)),
        tags: Some(tags),
        status,
        content_rating,
        viewer,
        chapters: None,
        next_update_time: None,
        update_strategy: UpdateStrategy::Always,
    })
}

pub fn extract_post_id(html: &Document) -> Result<String> {
    let post_id = html.select("div.manga-id")
        .and_then(|nodes| nodes.first())
        .and_then(|node| node.attr("data-post"))
        .unwrap_or_default();

    if !post_id.is_empty() {
        return Ok(post_id);
    }

    let rating_post_id = html.select("input#manga_id_post")
        .and_then(|nodes| nodes.first())
        .and_then(|node| node.attr("value"))
        .unwrap_or_default();

    if !rating_post_id.is_empty() {
        return Ok(rating_post_id);
    }

    Err(AidokuError::Unimplemented)
}

pub fn parse_chapter_list(manga_id: &str, html: Document) -> Result<Vec<Chapter>> {
    let mut chapters: Vec<Chapter> = Vec::new();

    if let Some(chapter_items) = html.select("li.wp-manga-chapter") {
        for item in chapter_items {
            let href = item.select("a")
                .and_then(|nodes| nodes.first())
                .and_then(|node| node.attr("href"))
                .unwrap_or_default();

            let key = href
                .replace(BASE_URL, "")
                .replace("/manga/", "")
                .replace(manga_id, "")
                .replace('/', "")
                .trim()
                .to_string();

            let title = item.select("a")
                .and_then(|nodes| nodes.first())
                .map(|node| node.text().unwrap_or_default())
                .unwrap_or_default()
                .trim()
                .to_string();

            let date_str = item.select("span.chapter-release-date")
                .and_then(|nodes| nodes.first())
                .map(|node| node.text().unwrap_or_default())
                .unwrap_or_default()
                .trim()
                .to_string();

            let date_uploaded = parse_date(&date_str);

            chapters.push(Chapter {
                key,
                title,
                volume_number: -1.0,
                chapter_number: -1.0,
                date_uploaded,
                scanlator: String::new(),
                url: href,
                lang: String::from("fr"),
                warning_watermark: None,
            });
        }
    }

    Ok(chapters)
}

pub fn parse_page_list(html: Document) -> Result<Vec<Page>> {
    let mut pages: Vec<Page> = Vec::new();
    let mut index = 0;

    if let Some(page_items) = html.select("div.page-break, li.blocks-gallery-item") {
        for item in page_items {
            if let Some(img_nodes) = item.select("img") {
                if let Some(img) = img_nodes.first() {
                    let url = helper::get_image_url(img);

                    if !url.is_empty() {
                        pages.push(Page {
                            index,
                            content: PageContent::Url(url),
                        });
                        index += 1;
                    }
                }
            }
        }
    }

    Ok(pages)
}

fn parse_date(date_str: &str) -> f64 {
    let date_lower = date_str.to_lowercase();

    if date_lower.contains("il y a") {
        let now = aidoku::imports::std::current_date();

        if date_lower.contains("minute") {
            if let Some(mins) = extract_number(&date_lower) {
                return now - (mins as f64 * 60.0);
            }
        } else if date_lower.contains("heure") {
            if let Some(hours) = extract_number(&date_lower) {
                return now - (hours as f64 * 3600.0);
            }
        } else if date_lower.contains("jour") {
            if let Some(days) = extract_number(&date_lower) {
                return now - (days as f64 * 86400.0);
            }
        } else if date_lower.contains("semaine") {
            if let Some(weeks) = extract_number(&date_lower) {
                return now - (weeks as f64 * 604800.0);
            }
        } else if date_lower.contains("mois") {
            if let Some(months) = extract_number(&date_lower) {
                return now - (months as f64 * 2592000.0);
            }
        } else if date_lower.contains("an") {
            if let Some(years) = extract_number(&date_lower) {
                return now - (years as f64 * 31536000.0);
            }
        }
    }

    if let Ok(timestamp) = parse_french_date(date_str) {
        return timestamp;
    }

    -1.0
}

fn extract_number(text: &str) -> Option<i32> {
    for word in text.split_whitespace() {
        if let Ok(num) = word.parse::<i32>() {
            return Some(num);
        }
    }
    None
}

fn parse_french_date(date_str: &str) -> Result<f64> {
    let parts: Vec<&str> = date_str.split('/').collect();

    if parts.len() == 3 {
        if let (Ok(day), Ok(month), Ok(mut year)) = (
            parts[0].parse::<i32>(),
            parts[1].parse::<i32>(),
            parts[2].parse::<i32>(),
        ) {
            if year < 100 {
                year += 2000;
            }

            let timestamp = calculate_timestamp(year, month, day);
            return Ok(timestamp);
        }
    }

    Err(AidokuError::Unimplemented)
}

fn calculate_timestamp(year: i32, month: i32, day: i32) -> f64 {
    let days_in_months = [31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);

    let mut total_days = (year - 1970) * 365;
    total_days += (year - 1969) / 4;
    total_days -= (year - 1901) / 100;
    total_days += (year - 1601) / 400;

    for m in 1..month {
        total_days += days_in_months[(m - 1) as usize];
        if m == 2 && is_leap_year {
            total_days += 1;
        }
    }

    total_days += day - 1;

    (total_days as f64) * 86400.0
}
