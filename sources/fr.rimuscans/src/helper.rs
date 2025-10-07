use aidoku::alloc::{String, Vec, format, string::ToString};

extern crate alloc;

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

pub fn parse_relative_date(text: &str) -> Option<i64> {
    let text = text.to_lowercase();
    let mut offset: i64 = 0;

    if text.contains("aujourd'hui") || text.contains("today") {
        return Some(0);
    }

    if text.contains("hier") || text.contains("yesterday") {
        return Some(-86400);
    }

    if let Some(value_str) = text.split_whitespace().next() {
        if let Ok(value) = value_str.parse::<i64>() {
            if text.contains("heure") || text.contains("hour") {
                offset = value * 3600;
            } else if text.contains("min") {
                offset = value * 60;
            } else if text.contains("jour") || text.contains("day") {
                offset = value * 86400;
            } else if text.contains("semaine") || text.contains("week") {
                offset = value * 86400 * 7;
            } else if text.contains("mois") || text.contains("month") {
                offset = value * 86400 * 30;
            } else if text.contains("an") || text.contains("year") {
                offset = value * 86400 * 365;
            }
            return Some(-offset);
        }
    }

    None
}

pub fn make_absolute_url(base: &str, url: &str) -> String {
    if url.starts_with("http://") || url.starts_with("https://") {
        url.to_string()
    } else if url.starts_with("//") {
        format!("https:{}", url)
    } else if url.starts_with('/') {
        format!("{}{}", base.trim_end_matches('/'), url)
    } else {
        format!("{}/{}", base.trim_end_matches('/'), url)
    }
}
