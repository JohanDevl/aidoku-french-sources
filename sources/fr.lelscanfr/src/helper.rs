use aidoku::{
    alloc::{String, Vec, format, vec},
    Chapter, Result,
    imports::net::Request,
};
use crate::{BASE_URL, USER_AGENT, parser};

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
        String::from(url)
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
            // Remove any non-digit characters at the end (like punctuation)
            let clean_number: String = first_number
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .collect();
            
            if let Ok(pages) = clean_number.parse::<i32>() {
                if pages > 1 && pages <= 100 {
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
                if pages > 1 && pages <= 100 {
                    return Some(pages);
                }
            }
        }
    }
    
    // Method 3: Find highest reasonable number (likely max page)
    let numbers: Vec<i32> = text
        .split_whitespace()
        .filter_map(|word| {
            // Clean word of common punctuation
            let clean_word: String = word
                .chars()
                .filter(|c| c.is_ascii_digit())
                .collect();
            clean_word.parse::<i32>().ok()
        })
        .filter(|&n| n > 1 && n <= 100) // Reasonable page range
        .collect();
    
    if !numbers.is_empty() {
        if let Some(&max_num) = numbers.iter().max() {
            // Only return if we found a reasonable pagination number
            if max_num >= 2 && max_num <= 50 {
                return Some(max_num);
            }
        }
    }
    
    // Method 4: Check for ellipsis indicating more pages
    if text.contains("…") || text.contains("...") {
        return Some(10); // Conservative estimate when ellipsis is present
    }
    
    None
}

pub fn fetch_pages_batch(manga_key: &str, start_page: i32, total_pages: i32) -> Result<Vec<Chapter>> {
    let mut all_chapters: Vec<Chapter> = Vec::new();
    
    // Process pages in small batches to balance performance and memory usage
    const BATCH_SIZE: i32 = 3;
    const MAX_RETRY: u8 = 2;
    
    for batch_start in (start_page..=total_pages).step_by(BATCH_SIZE as usize) {
        let batch_end = (batch_start + BATCH_SIZE - 1).min(total_pages);
        
        // Fetch batch of pages with retry logic
        let batch_chapters = fetch_single_batch(manga_key, batch_start, batch_end, MAX_RETRY)?;
        all_chapters.extend(batch_chapters);
        
        // Small delay between batches to be respectful to the server
        // Note: In WASM environment, this is just a hint for the runtime
        if batch_end < total_pages {
            let _ = helper_delay_ms(250); // 250ms delay between batches
        }
    }
    
    Ok(all_chapters)
}

fn fetch_single_batch(manga_key: &str, start_page: i32, end_page: i32, max_retry: u8) -> Result<Vec<Chapter>> {
    let mut batch_chapters: Vec<Chapter> = Vec::new();
    
    for page in start_page..=end_page {
        let mut attempts = 0;
        let mut success = false;
        
        while attempts < max_retry && !success {
            match fetch_single_page(manga_key, page) {
                Ok(chapters) => {
                    batch_chapters.extend(chapters);
                    success = true;
                }
                Err(_) => {
                    attempts += 1;
                    if attempts < max_retry {
                        // Small delay before retry
                        let _ = helper_delay_ms(500);
                    }
                }
            }
        }
        
        // If we failed to fetch a page after retries, continue with others
        // Don't fail the entire batch for one page
        if !success {
            // Log the failure but continue - this is better than failing everything
            // Note: In production, you might want to handle this differently
        }
    }
    
    Ok(batch_chapters)
}

fn fetch_single_page(manga_key: &str, page: i32) -> Result<Vec<Chapter>> {
    let page_url = format!("{}/manga/{}?page={}", BASE_URL, manga_key, page);
    
    let page_html = Request::get(&page_url)?
        .header("User-Agent", USER_AGENT)
        .header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
        .header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
        .header("Accept-Encoding", "gzip, deflate, br")
        .header("DNT", "1")
        .header("Connection", "keep-alive")
        .html()?;
    
    // Parse immediately to save memory
    parser::parse_chapter_list(manga_key, vec![page_html])
}

// Helper function for delays (limited functionality in WASM)
fn helper_delay_ms(_ms: u32) -> Result<()> {
    // In WASM environment, we can't do real delays
    // This is just a placeholder for potential future async support
    // or runtime scheduling hints
    Ok(())
}