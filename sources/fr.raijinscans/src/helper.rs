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

pub fn decode_base64(encoded: &str) -> Option<String> {
    const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    let input = encoded.trim().as_bytes();
    let mut output = Vec::new();
    let mut buffer: u32 = 0;
    let mut bits_collected: u32 = 0;

    for &byte in input {
        if byte == b'=' {
            break;
        }

        let value = BASE64_CHARS.iter().position(|&c| c == byte)?;

        buffer = (buffer << 6) | (value as u32);
        bits_collected += 6;

        if bits_collected >= 8 {
            bits_collected -= 8;
            output.push((buffer >> bits_collected) as u8);
            buffer &= (1 << bits_collected) - 1;
        }
    }

    String::from_utf8(output).ok()
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

pub fn validate_image_url(url: &str) -> bool {
    if url.is_empty() {
        return false;
    }

    if url.starts_with("javascript:")
        || url.starts_with("data:")
        || url.starts_with("file:")
        || url.starts_with("vbscript:") {
        return false;
    }

    url.starts_with("http://") || url.starts_with("https://") || url.starts_with("//") || url.starts_with('/')
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

pub fn clean_description(text: String) -> String {
    let mut result = text;

    result = result
        .replace("<br>", "\n")
        .replace("<br/>", "\n")
        .replace("<br />", "\n")
        .replace("</p>", "\n");

    let mut cleaned = String::new();
    let mut in_tag = false;
    let chars: Vec<char> = result.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '<' {
            in_tag = true;
            i += 1;
            continue;
        }

        if chars[i] == '>' {
            in_tag = false;
            i += 1;
            continue;
        }

        if !in_tag {
            cleaned.push(chars[i]);
        }

        i += 1;
    }

    cleaned
        .replace("&lt;", "")
        .replace("&gt;", "")
        .replace("&amp;", "&")
        .replace("&#039;", "'")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
        .trim()
        .to_string()
}
