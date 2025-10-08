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

pub fn make_absolute_url(base_url: &str, url: &str) -> String {
    if url.starts_with("http") {
        String::from(url)
    } else if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with('/') {
        format!("{}{}", base_url, url)
    } else {
        format!("{}/{}", base_url, url)
    }
}

pub fn extract_chapter_number(title: &str) -> f32 {
    let title_lower = title.to_lowercase();

    if let Some(idx) = title_lower.find("chapitre") {
        let after = &title_lower[idx + 8..].trim_start();
        if let Some(num_str) = after.split_whitespace().next() {
            let cleaned = num_str.replace(',', ".");
            if let Ok(num) = cleaned.parse::<f32>() {
                return num;
            }
        }
    }

    if let Some(idx) = title_lower.find("chapter") {
        let after = &title_lower[idx + 7..].trim_start();
        if let Some(num_str) = after.split_whitespace().next() {
            let cleaned = num_str.replace(',', ".");
            if let Ok(num) = cleaned.parse::<f32>() {
                return num;
            }
        }
    }

    if let Some(idx) = title_lower.find("ch.") {
        let after = &title_lower[idx + 3..].trim_start();
        if let Some(num_str) = after.split_whitespace().next() {
            let cleaned = num_str.replace(',', ".");
            if let Ok(num) = cleaned.parse::<f32>() {
                return num;
            }
        }
    }

    -1.0
}

pub fn parse_status(status_text: &str) -> aidoku::MangaStatus {
    use aidoku::MangaStatus;

    let status_lower = status_text.to_lowercase();

    if status_lower.contains("en cours") || status_lower.contains("ongoing") {
        return MangaStatus::Ongoing;
    }
    if status_lower.contains("complété")
        || status_lower.contains("completed")
        || status_lower.contains("termine") {
        return MangaStatus::Completed;
    }
    if status_lower.contains("en pause") || status_lower.contains("hiatus") {
        return MangaStatus::Hiatus;
    }
    if status_lower.contains("abandonné")
        || status_lower.contains("cancelled")
        || status_lower.contains("dropped") {
        return MangaStatus::Cancelled;
    }

    MangaStatus::Unknown
}
