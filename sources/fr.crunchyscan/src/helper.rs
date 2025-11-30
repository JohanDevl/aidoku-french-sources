use aidoku::alloc::{String, Vec, format, string::ToString};

/// Extract CSRF token from HTML page
/// Looks for: <meta name="csrf-token" content="...">
pub fn extract_csrf_token(html: &str) -> Option<String> {
    // Look for csrf-token meta tag
    if let Some(start) = html.find("name=\"csrf-token\"") {
        // Find the content attribute after it
        let after_name = &html[start..];
        if let Some(content_start) = after_name.find("content=\"") {
            let content_after = &after_name[content_start + 9..]; // Skip 'content="'
            if let Some(end) = content_after.find('"') {
                return Some(content_after[..end].to_string());
            }
        }
    }

    // Alternative: look for content first then name
    if let Some(start) = html.find("content=\"") {
        let before = &html[..start];
        if before.ends_with("csrf-token\" ") || before.contains("csrf-token\"") {
            let after = &html[start + 9..];
            if let Some(end) = after.find('"') {
                return Some(after[..end].to_string());
            }
        }
    }

    None
}

pub fn urlencode(string: &str) -> String {
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


pub fn extract_slug_from_url(url: &str) -> String {
    let parts: Vec<&str> = url.split('/').collect();
    
    // Try multiple URL patterns
    let patterns = ["lecture-en-ligne", "manga", "series"];
    
    for pattern in &patterns {
        for (i, &part) in parts.iter().enumerate() {
            if part == *pattern && i + 1 < parts.len() {
                let slug = parts[i + 1];
                if !slug.is_empty() && !slug.contains('?') && !slug.contains('#') {
                    // Take only the part before any query parameters or fragments
                    return slug.split('?').next().unwrap_or(slug).split('#').next().unwrap_or(slug).to_string();
                }
            }
        }
    }
    
    // Fallback: try to extract any meaningful slug from the URL
    if let Some(last_part) = parts.iter().rev().find(|&&part| !part.is_empty() && part != "index.html" && part != "index.php") {
        let slug = last_part.split('?').next().unwrap_or(last_part).split('#').next().unwrap_or(last_part);
        if !slug.is_empty() && slug.len() > 1 {
            return slug.to_string();
        }
    }
    
    String::new()
}

pub fn build_manga_url(slug: &str) -> String {
    format!("https://crunchyscan.fr/lecture-en-ligne/{}", slug)
}

pub fn build_chapter_url(manga_slug: &str, chapter_slug: &str) -> String {
    format!("https://crunchyscan.fr/lecture-en-ligne/{}/read/{}", manga_slug, chapter_slug)
}

pub fn clean_title(title: &str) -> String {
    title
        .replace("Lire le manga ", "")
        .replace("couverture du ", "")
        .replace(" | Crunchyscan", "")
        .trim()
        .to_string()
}

pub fn parse_relative_time(time_str: &str) -> i64 {
    let now = 1672531200; // Fixed timestamp for consistency
    
    if time_str.contains("heure") {
        if let Ok(hours) = time_str.split_whitespace().next().unwrap_or("1").parse::<i64>() {
            return now - (hours * 3600);
        }
    } else if time_str.contains("jour") {
        if let Ok(days) = time_str.split_whitespace().next().unwrap_or("1").parse::<i64>() {
            return now - (days * 24 * 3600);
        }
    } else if time_str.contains("mois") {
        if let Ok(months) = time_str.split_whitespace().next().unwrap_or("1").parse::<i64>() {
            return now - (months * 30 * 24 * 3600);
        }
    } else if time_str.contains("semaine") {
        if let Ok(weeks) = time_str.split_whitespace().next().unwrap_or("1").parse::<i64>() {
            return now - (weeks * 7 * 24 * 3600);
        }
    }
    
    now
}

pub fn extract_chapter_number(chapter_title: &str) -> f32 {
    if let Some(captures) = chapter_title.split_whitespace().find(|s| s.chars().any(|c| c.is_ascii_digit())) {
        let number_str: String = captures.chars()
            .filter(|c| c.is_ascii_digit() || *c == '.')
            .collect();
        
        if let Ok(num) = number_str.parse::<f32>() {
            return num;
        }
    }
    
    1.0
}

pub fn make_absolute_url(base_url: &str, url: &str) -> String {
    if url.starts_with("http") {
        String::from(url)
    } else if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with("/") {
        format!("{}{}", base_url.trim_end_matches('/'), url)
    } else {
        format!("{}/{}", base_url.trim_end_matches('/'), url)
    }
}