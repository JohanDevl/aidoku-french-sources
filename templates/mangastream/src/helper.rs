use core::ptr;
use crate::types::prelude::*;
use crate::template::MangaStreamSource;

extern crate hashbrown;
use hashbrown::HashMap;

pub const USER_AGENT: &str = "Mozilla/5.0 (iPhone; CPU iPhone OS 16_1_2 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/16.0 Mobile/15E148 Safari/604.1";

// generate url for listing page
pub fn get_listing_url(
    listing: [&str; 3],
    base_url: String,
    pathname: String,
    listing_name: String,
    page: i32,
) -> String {
    let list_type = if listing_name == listing[0] {
        "order=update"
    } else if listing_name == listing[1] {
        "order=popular"
    } else if listing_name == listing[2] {
        "order=latest"
    } else {
        ""
    };
    match page {
        1 => format!("{}/{}/?{}", base_url, pathname, list_type),
        _ => format!("{}/{}/?page={}&{}", base_url, pathname, page, list_type),
    }
}

// return the manga status
pub fn manga_status(
    status: String,
    status_options: [&'static str; 5],
    status_options_2: [&'static str; 5],
) -> MangaStatus {
    if (!status_options[0].is_empty() && status.contains(status_options[0]))
        || (!status_options_2[0].is_empty() && status.contains(status_options_2[0]))
    {
        MangaStatus::Ongoing
    } else if (!status_options[1].is_empty() && status.contains(status_options[1]))
        || (!status_options_2[1].is_empty() && status.contains(status_options_2[1]))
    {
        MangaStatus::Completed
    } else if (!status_options[2].is_empty() && status.contains(status_options[2]))
        || (!status_options_2[2].is_empty() && status.contains(status_options_2[2]))
    {
        MangaStatus::Hiatus
    } else if (!status_options[3].is_empty() && status.contains(status_options[3]))
        || (!status_options[4].is_empty() && status.contains(status_options[4]))
        || (!status_options_2[3].is_empty() && status.contains(status_options_2[3]))
        || (!status_options_2[4].is_empty() && status.contains(status_options_2[4]))
    {
        MangaStatus::Cancelled
    } else {
        MangaStatus::Unknown
    }
}

//converts integer(i32) to string
pub fn i32_to_string(mut integer: i32) -> String {
    if integer == 0 {
        return String::from("0");
    }
    let mut string = String::with_capacity(11);
    let pos = if integer < 0 {
        string.insert(0, '-');
        1
    } else {
        0
    };
    while integer != 0 {
        let mut digit = integer % 10;
        if pos == 1 {
            digit *= -1;
        }
        string.insert(pos, char::from_u32((digit as u32) + ('0' as u32)).unwrap());
        integer /= 10;
    }
    string
}

/// Converts `<br>` and `\n` into newlines.
pub fn text_with_newlines(node: NodeSelection) -> String {
    let html = node.html().read();
    if !String::from(html.trim()).is_empty() {
        // Simplified version without Node::new_fragment
        html.replace("<br>", "\n")
            .replace("\\n", "\n")
            .trim()
            .to_string()
    } else {
        String::new()
    }
}

// return chapter number from string
pub fn get_chapter_number(id: String) -> f32 {
    id.chars()
        .filter(|a| (*a >= '0' && *a <= '9') || *a == ' ' || *a == '.')
        .collect::<String>()
        .split(' ')
        .collect::<Vec<&str>>()
        .into_iter()
        .map(|a| a.parse::<f32>().unwrap_or(0.0))
        .find(|a| *a > 0.0)
        .unwrap_or(0.0)
}

// generates the search, filter and homepage url
pub fn get_search_url(
    source: &MangaStreamSource,
    query: String,
    page: i32,
    included_tags: Vec<String>,
    excluded_tags: Vec<String>,
    status: String,
    manga_type: String,
) -> String {
    let mut url = format!("{}/{}", source.base_url, source.traverse_pathname);
    if query.is_empty() && included_tags.is_empty() && status.is_empty() && manga_type.is_empty() {
        return get_listing_url(
            source.listing,
            source.base_url.clone(),
            String::from(source.traverse_pathname),
            String::from(source.listing[0]),
            page,
        );
    }
    if !query.is_empty() {
        url.push_str(&format!("/page/{}?s={}", page, query.replace(' ', "+")))
    } else {
        url.push_str(&format!("/?page={}", page));
    }
    if !included_tags.is_empty() || !excluded_tags.is_empty() {
        if excluded_tags.is_empty() {
            for tag in included_tags {
                url.push_str(&format!("&genre%5B%5D={}", tag));
            }
        } else if !included_tags.is_empty() && !excluded_tags.is_empty() {
            for tag in included_tags {
                url.push_str(&format!("&genre%5B%5D={}", tag));
            }
            for tag in excluded_tags {
                url.push_str(&format!("&genre%5B%5D=-{}", tag));
            }
        } else {
            for tag in excluded_tags {
                url.push_str(&format!("&genre%5B%5D=-{}", tag));
            }
        }
    }
    if !status.is_empty() {
        url.push_str(&format!("&status={}", status));
    }
    if !manga_type.is_empty() {
        url.push_str(&format!("&type={}", urlencode(manga_type)));
    }
    url
}

// return the date depending on the language
pub fn get_date(source: &MangaStreamSource, raw_date: StringRef) -> f64 {
    match source.base_url.contains(source.date_string) {
        true => raw_date
            .as_date(source.chapter_date_format_2, Some(source.locale_2), None)
            .unwrap_or(0.0),
        _ => raw_date
            .as_date(source.chapter_date_format, Some(source.locale), None)
            .unwrap_or(0.0),
    }
}

// encoding non alpha-numeric characters to utf8
pub fn img_url_encode(string: String) -> String {
    let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
    let hex = "0123456789abcdef".as_bytes();
    let bytes = string.as_bytes();

    for byte in bytes {
        let curr = *byte;
        if curr == b'-' {
            result.push(b'-');
        } else if curr == b'.' {
            result.push(b'.');
        } else if curr == b'_' {
            result.push(b'_');
        } else if curr.is_ascii_lowercase() || curr.is_ascii_uppercase() || curr.is_ascii_digit() {
            result.push(curr);
        } else {
            result.push(b'%');
            if hex[curr as usize >> 4] >= 97 && hex[curr as usize >> 4] <= 122 {
                result.push(hex[curr as usize >> 4] - 32);
            } else {
                result.push(hex[curr as usize >> 4]);
            }
            if hex[curr as usize & 15] >= 97 && hex[curr as usize & 15] <= 122 {
                result.push(hex[curr as usize & 15] - 32);
            } else {
                result.push(hex[curr as usize & 15]);
            }
        }
    }
    String::from_utf8(result).unwrap_or_default()
}

//get the image sources as some images are in base64 format
pub fn get_image_src(node: NodeSelection) -> String {
    let mut image = String::new();
    
    // Since node is already a NodeSelection, we can directly access attributes
    // We'll assume node already contains the img element
    let src = node.attr("src").read();
    let data_lazy = node.attr("data-lazy-src").read();
    let data_src = node.attr("data-src").read();
    
    if !src.starts_with("data") && !src.is_empty() {
        image = src.replace("?resize=165,225", "");
    } else if !data_lazy.starts_with("data") && !data_lazy.is_empty() {
        image = data_lazy.replace("?resize=165,225", "");
    } else if !data_src.starts_with("data") && !data_src.is_empty() {
        image = data_src.replace("?resize=165,225", "");
    }
    
    let img_split = image.split('/').collect::<Vec<&str>>();
    let last_encoded = img_url_encode(String::from(img_split[img_split.len() - 1]));
    let mut encoded_img = String::new();

    (0..img_split.len() - 1).for_each(|i| {
        encoded_img.push_str(img_split[i]);
        encoded_img.push('/');
    });
    encoded_img.push_str(&last_encoded);
    append_protocol(encoded_img)
}

pub fn append_protocol(url: String) -> String {
    if url.starts_with("https") || url.starts_with("http") {
        url
    } else {
        format!("{}{}", "https:", url)
    }
}

pub fn urlencode<T: AsRef<[u8]>>(url: T) -> String {
    let bytes = url.as_ref();
    let hex = "0123456789ABCDEF".as_bytes();

    let mut result: Vec<u8> = Vec::with_capacity(bytes.len() * 3);

    for byte in bytes {
        let curr = *byte;
        if curr.is_ascii_alphanumeric() || b";,/?:@&=+$-_.!~*'()#".contains(&curr) {
            result.push(curr);
        } else {
            result.push(b'%');
            result.push(hex[curr as usize >> 4]);
            result.push(hex[curr as usize & 15]);
        }
    }
    String::from_utf8(result).unwrap_or_default()
}

/// This function is used to get the permanent url of a manga or chapter
pub fn get_permanet_url(original_url: String) -> String {
    let mut original_url = original_url;

    if original_url.ends_with('/') {
        original_url.pop();
    };

    let garbage = original_url
        .split('/')
        .last()
        .expect("Failed to split url by /")
        .split('-')
        .next()
        .expect("Failed to split url by -");

    if garbage.parse::<u64>().is_ok() && garbage.len() == 10 {
        original_url.replace(&format!("{}{}", garbage, "-"), "")
    } else {
        original_url
    }
}

/// This function is used to get the id from a url
pub fn get_id_from_url(url: String) -> String {
    let mut url = url;

    if url.ends_with('/') {
        url.pop();
    };

    // Simplified version without substring helpers
    if url.contains("p=") {
        let parts: Vec<&str> = url.split("p=").collect();
        if parts.len() > 1 {
            let post_part = parts[1];
            let end_parts: Vec<&str> = post_part.split("&").collect();
            return String::from(end_parts[0]);
        }
    }

    let id = url.split('/').last().expect("Failed to parse id from url");
    String::from(id)
}

pub fn get_lang_code() -> String {
    // Simplified version without defaults_get
    String::from("fr")
}

static mut CACHED_MANGA_URL_TO_POSTID_MAPPING: Option<HashMap<String, String>> = None;
static mut CACHED_MAPPING_AT: f64 = 0.0;

/// Generate a hashmap of manga url to postid mappings
fn generate_manga_url_to_postid_mapping(
    url: &str,
    pathname: &str,
) -> Result<HashMap<String, String>> {
    unsafe {
        if current_date() - CACHED_MAPPING_AT < 600.0 {
            if let Some(mapping) = &mut *ptr::addr_of_mut!(CACHED_MANGA_URL_TO_POSTID_MAPPING) {
                return Ok(mapping.clone());
            }
        }
    }

    let all_manga_listing_url = format!("{}/{}/list-mode", url, pathname);

    let html = Request::new(&all_manga_listing_url, HttpMethod::Get)
        .header("User-Agent", USER_AGENT)
        .html()?;
    let mut mapping = HashMap::new();

    for node in html.select(".soralist .series").array() {
        let manga = node.as_node()?;

        let url = manga.attr("href").read();
        let post_id = manga.attr("rel").read();

        mapping.insert(url, post_id);
    }

    unsafe {
        CACHED_MANGA_URL_TO_POSTID_MAPPING = Some(mapping.clone());
        CACHED_MAPPING_AT = current_date();
    }

    Ok(mapping)
}

/// Search the `MANGA_URL_TO_POSTID_MAPPING` for the postid from a manga url
pub fn get_postid_from_manga_url(url: String, base_url: &str, pathname: &str) -> Result<String> {
    let manga_url_to_postid_mapping = generate_manga_url_to_postid_mapping(base_url, pathname)?;
    let id = manga_url_to_postid_mapping.get(&url).ok_or(
        AidokuError::new("Postid not found for manga URL")
    )?;

    Ok(String::from(id))
}

/// Generate a hashmap of chapter url to postid mappings
pub fn generate_chapter_url_to_postid_mapping(
    post_id: String,
    base_url: &str,
) -> Result<HashMap<String, String>> {
    let ajax_url = format!("{}/wp-admin/admin-ajax.php", base_url);

    let start = current_date();

    let body = format!("action=get_chapters&id={}", post_id);
    let html = Request::new(&ajax_url, HttpMethod::Post)
        .body(body.as_bytes())
        .header("Referer", base_url)
        .header("User-Agent", USER_AGENT)
        .html()?;

    // Simplified retry logic
    if html.select("title").text().read() == "429 Too Many Requests" {
        // Simple delay simulation - in real implementation this would be more sophisticated
        if start + 10.0 < current_date() {
            return generate_chapter_url_to_postid_mapping(post_id, base_url);
        }
    }

    let mut mapping = HashMap::new();

    for node in html.select("option").array() {
        let chapter = node.as_node()?;

        let url = chapter.attr("value").read();
        let post_id = chapter.attr("data-id").read();

        mapping.insert(url, post_id);
    }

    Ok(mapping)
}