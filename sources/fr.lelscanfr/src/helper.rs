use aidoku::alloc::{String, Vec, format};
use aidoku::imports::html::Document;

const MAX_PAGINATION_PAGES: i32 = 150;
const MIN_PAGINATION_VALUE: i32 = 2;
const MAX_PAGINATION_VALUE: i32 = 200;
const MAX_VISIBLE_PAGES: i32 = 20;
const MAX_ESTIMATED_PAGES: i32 = 100;
const PAGE_MULTIPLIER: i32 = 3;
const DEFAULT_ELLIPSIS_ESTIMATE: i32 = 25;

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
			|| curr == b'-'
			|| curr == b'_'
			|| curr == b'.'
			|| curr == b'~'
		{
			result.push(curr);
		} else if curr == b' ' {
			result.push(b'+');
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}


pub fn extract_id_from_url(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    if parts.len() >= 5 {
        String::from(parts[4].trim())
    } else {
        String::new()
    }
}

pub fn make_absolute_url(base_url: &str, url: &str) -> String {
    if url.starts_with("http") {
        if url.starts_with("https://lelscanfr.com") || url.starts_with("http://lelscanfr.com") {
            String::from(url)
        } else {
            String::new()
        }
    } else if url.starts_with("/") {
        format!("{}{}", base_url, url)
    } else {
        format!("{}/{}", base_url, url)
    }
}

pub fn extract_pagination_total(text: &str) -> Option<i32> {
    // Method 1: Look for "Page X of Y" pattern
    if let Some(of_pos) = text.find(" of ") {
        let after_of = &text[of_pos + 4..];
        if let Some(first_number) = after_of.split_whitespace().next() {
            let clean_number: String = first_number
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();

            if let Ok(pages) = clean_number.parse::<i32>() {
                if pages >= MIN_PAGINATION_VALUE && pages <= MAX_PAGINATION_VALUE {
                    return Some(pages);
                }
            }
        }
    }
    
    // Method 2: Look for "Page X sur Y" pattern (French)
    if let Some(sur_pos) = text.find(" sur ") {
        let after_sur = &text[sur_pos + 5..];
        if let Some(first_number) = after_sur.split_whitespace().next() {
            let clean_number: String = first_number
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            
            if let Ok(pages) = clean_number.parse::<i32>() {
                if pages >= MIN_PAGINATION_VALUE && pages <= MAX_PAGINATION_VALUE {
                    return Some(pages);
                }
            }
        }
    }

    // Method 3: Find highest reasonable number (likely max page)
    let numbers: Vec<i32> = text
        .split_whitespace()
        .filter_map(|word| {
            let clean_word: String = word
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            clean_word.parse::<i32>().ok()
        })
        .filter(|&n| n >= MIN_PAGINATION_VALUE && n <= MAX_PAGINATION_VALUE)
        .collect();

    if !numbers.is_empty() {
        if let Some(&max_num) = numbers.iter().max() {
            if max_num >= MIN_PAGINATION_VALUE && max_num <= MAX_PAGINATION_PAGES {
                return Some(max_num);
            }
        }
    }
    
    // Method 4: Check for ellipsis indicating more pages
    if text.contains("…") || text.contains("...") {
        let existing_numbers: Vec<i32> = text
            .split_whitespace()
            .filter_map(|s| s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
            .filter(|&n| n >= MIN_PAGINATION_VALUE && n <= MAX_VISIBLE_PAGES)
            .collect();

        if let Some(&max_visible) = existing_numbers.iter().max() {
            return Some((max_visible * PAGE_MULTIPLIER).min(MAX_ESTIMATED_PAGES));
        } else {
            return Some(DEFAULT_ELLIPSIS_ESTIMATE);
        }
    }
    
    None
}

pub fn detect_pagination(html: &Document) -> i32 {
    let mut total_pages = 1;

    let pagination_containers = [".pagination", ".page-numbers", ".pages", "nav"];
    for selector in pagination_containers {
        if let Some(pagination_element) = html.select(selector) {
            if let Some(text) = pagination_element.text() {
                if let Some(total) = extract_pagination_total(&text) {
                    total_pages = total;
                    break;
                }
            }
        }
    }

    if total_pages == 1 {
        let limited_selectors = [".pagination-info", ".page-info", ".pagination-text"];
        for selector in limited_selectors {
            if let Some(element) = html.select(selector) {
                if let Some(text) = element.text() {
                    if let Some(total) = extract_pagination_total(&text) {
                        total_pages = total;
                        break;
                    }
                }
            }
        }
    }

    if total_pages == 1 {
        if let Some(body) = html.select("body") {
            let body_text = body.text().unwrap_or_default();
            if let Some(total) = extract_pagination_total(&body_text) {
                total_pages = total;
            }
        }
    }

    if total_pages > MAX_PAGINATION_PAGES {
        total_pages = MAX_PAGINATION_PAGES;
    }

    total_pages
}

pub fn has_next_page(html: &Document) -> bool {
    if let Some(pagination_elements) = html.select("div, span, p") {
        for elem in pagination_elements {
            if let Some(text) = elem.text() {
                if text.contains("Page ") && text.contains(" of ") {
                    if let Some(of_pos) = text.find(" of ") {
                        let after_of = &text[of_pos + 4..].trim();
                        if let Some(total_pages_str) = after_of.split_whitespace().next() {
                            if let Ok(total_pages) = total_pages_str.parse::<i32>() {
                                if let Some(page_start) = text.find("Page ") {
                                    let after_page = &text[page_start + 5..of_pos];
                                    if let Ok(current_page) = after_page.trim().parse::<i32>() {
                                        return current_page < total_pages;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    let pagination_selectors = [".pagination", ".page-numbers", ".pages"];
    for selector in pagination_selectors {
        if let Some(pagination) = html.select(selector) {
            if let Some(first_pagination) = pagination.first() {
                if let Some(pagination_text) = first_pagination.text() {
                    if pagination_text.contains("…") || pagination_text.contains("...") {
                        return true;
                    }
                }
                break;
            }
        }
    }

    false
}

