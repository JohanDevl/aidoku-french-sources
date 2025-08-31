use aidoku::alloc::{String, Vec, format};

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
                if pages > 1 && pages <= 200 {
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
                if pages > 1 && pages <= 200 {
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
        .filter(|&n| n > 1 && n <= 200) // Expanded page range for large manga
        .collect();
    
    if !numbers.is_empty() {
        if let Some(&max_num) = numbers.iter().max() {
            // Accept higher pagination numbers for manga with many chapters
            if max_num >= 2 && max_num <= 150 {
                return Some(max_num);
            }
        }
    }
    
    // Method 4: Check for ellipsis indicating more pages - be more aggressive
    if text.contains("…") || text.contains("...") {
        // When ellipsis is present, assume there are many more pages
        // Look for any existing numbers and multiply, or use conservative estimate
        let existing_numbers: Vec<i32> = text
            .split_whitespace()
            .filter_map(|s| s.chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok())
            .filter(|&n| n > 1 && n <= 20)
            .collect();
        
        if let Some(&max_visible) = existing_numbers.iter().max() {
            return Some((max_visible * 3).min(100)); // Estimate there could be 3x more pages
        } else {
            return Some(25); // More aggressive estimate when ellipsis is present
        }
    }
    
    None
}

