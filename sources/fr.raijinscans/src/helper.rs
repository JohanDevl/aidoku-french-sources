use aidoku::alloc::{String, Vec, format, string::ToString};
use aidoku::imports::std::current_date;

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
    const SECONDS_PER_MINUTE: i64 = 60;
    const SECONDS_PER_HOUR: i64 = 3600;
    const SECONDS_PER_DAY: i64 = 86400;
    const DAYS_PER_WEEK: i64 = 7;
    const DAYS_PER_MONTH: i64 = 30;
    const DAYS_PER_YEAR: i64 = 365;

    let text_lower = text.trim().to_lowercase();

    if text_lower.contains("aujourd'hui") || text_lower.contains("today") {
        return Some(current_date());
    }

    if text_lower.contains("hier") || text_lower.contains("yesterday") {
        return Some(current_date() - SECONDS_PER_DAY);
    }

    let text_clean = text_lower
        .trim_start_matches("il y a")
        .trim();

    let parts: Vec<&str> = text_clean.split_whitespace().collect();

    let (value, unit_text) = if !parts.is_empty() {
        if let Ok(num) = parts[0].parse::<i64>() {
            let unit = parts.get(1).unwrap_or(&"").to_string();
            (num, unit)
        } else {
            let mut num_str = String::new();
            let mut unit_str = String::new();
            let mut parsing_number = true;

            for ch in parts[0].chars() {
                if ch.is_numeric() && parsing_number {
                    num_str.push(ch);
                } else {
                    parsing_number = false;
                    unit_str.push(ch);
                }
            }

            if let Ok(num) = num_str.parse::<i64>() {
                (num, unit_str)
            } else {
                return None;
            }
        }
    } else {
        return None;
    };

    if unit_text.is_empty() && parts.len() == 1 {
        return None;
    }

    let offset = if text_lower.contains(" min") || (unit_text == "min" && !text_lower.contains("mois")) {
        value * SECONDS_PER_MINUTE
    } else if unit_text.starts_with('h') || text_lower.contains("heure") || text_lower.contains("hour") {
        value * SECONDS_PER_HOUR
    } else if unit_text.starts_with('j') || text_lower.contains("jour") || text_lower.contains("day") {
        value * SECONDS_PER_DAY
    } else if text_lower.contains("semaine") || text_lower.contains("week") {
        value * SECONDS_PER_DAY * DAYS_PER_WEEK
    } else if unit_text.starts_with('m') || text_lower.contains("mois") || text_lower.contains("month") {
        value * SECONDS_PER_DAY * DAYS_PER_MONTH
    } else if text_lower.contains(" an") || text_lower.contains("year") {
        value * SECONDS_PER_DAY * DAYS_PER_YEAR
    } else {
        return None;
    };

    Some(current_date() - offset)
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
        .replace("&#91;", "[")
        .replace("&#93;", "]")
        .replace("&#40;", "(")
        .replace("&#41;", ")")
        .replace("&#123;", "{")
        .replace("&#125;", "}")
        .replace("&#171;", "\u{AB}")
        .replace("&#187;", "\u{BB}")
        .replace("&#8249;", "\u{2039}")
        .replace("&#8250;", "\u{203A}")
        .replace("&#8211;", "\u{2013}")
        .replace("&#8212;", "\u{2014}")
        .replace("&#8216;", "\u{2018}")
        .replace("&#8217;", "\u{2019}")
        .replace("&#8220;", "\u{201C}")
        .replace("&#8221;", "\u{201D}")
        .replace("&#224;", "\u{E0}")
        .replace("&#192;", "\u{C0}")
        .replace("&#226;", "\u{E2}")
        .replace("&#194;", "\u{C2}")
        .replace("&#231;", "\u{E7}")
        .replace("&#199;", "\u{C7}")
        .replace("&#232;", "\u{E8}")
        .replace("&#200;", "\u{C8}")
        .replace("&#233;", "\u{E9}")
        .replace("&#201;", "\u{C9}")
        .replace("&#234;", "\u{EA}")
        .replace("&#202;", "\u{CA}")
        .replace("&#235;", "\u{EB}")
        .replace("&#203;", "\u{CB}")
        .replace("&#238;", "\u{EE}")
        .replace("&#206;", "\u{CE}")
        .replace("&#239;", "\u{EF}")
        .replace("&#207;", "\u{CF}")
        .replace("&#244;", "\u{F4}")
        .replace("&#212;", "\u{D4}")
        .replace("&#249;", "\u{F9}")
        .replace("&#217;", "\u{D9}")
        .replace("&#251;", "\u{FB}")
        .replace("&#219;", "\u{DB}")
        .replace("&#252;", "\u{FC}")
        .replace("&#220;", "\u{DC}")
        .replace("&#255;", "\u{FF}")
        .replace("&#376;", "\u{178}")
        .replace("&#339;", "\u{153}")
        .replace("&#338;", "\u{152}")
        .replace("&#230;", "\u{E6}")
        .replace("&#198;", "\u{C6}")
        .replace("&#8364;", "\u{20AC}")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&")
        .replace("&apos;", "'")
        .replace("&#039;", "'")
        .replace("&quot;", "\"")
        .replace("&nbsp;", " ")
        .replace("&laquo;", "\u{AB}")
        .replace("&raquo;", "\u{BB}")
        .replace("&lsaquo;", "\u{2039}")
        .replace("&rsaquo;", "\u{203A}")
        .replace("&ndash;", "\u{2013}")
        .replace("&mdash;", "\u{2014}")
        .replace("&lsquo;", "\u{2018}")
        .replace("&rsquo;", "\u{2019}")
        .replace("&ldquo;", "\u{201C}")
        .replace("&rdquo;", "\u{201D}")
        .replace("&agrave;", "\u{E0}")
        .replace("&Agrave;", "\u{C0}")
        .replace("&acirc;", "\u{E2}")
        .replace("&Acirc;", "\u{C2}")
        .replace("&ccedil;", "\u{E7}")
        .replace("&Ccedil;", "\u{C7}")
        .replace("&egrave;", "\u{E8}")
        .replace("&Egrave;", "\u{C8}")
        .replace("&eacute;", "\u{E9}")
        .replace("&Eacute;", "\u{C9}")
        .replace("&ecirc;", "\u{EA}")
        .replace("&Ecirc;", "\u{CA}")
        .replace("&euml;", "\u{EB}")
        .replace("&Euml;", "\u{CB}")
        .replace("&icirc;", "\u{EE}")
        .replace("&Icirc;", "\u{CE}")
        .replace("&iuml;", "\u{EF}")
        .replace("&Iuml;", "\u{CF}")
        .replace("&ocirc;", "\u{F4}")
        .replace("&Ocirc;", "\u{D4}")
        .replace("&ugrave;", "\u{F9}")
        .replace("&Ugrave;", "\u{D9}")
        .replace("&ucirc;", "\u{FB}")
        .replace("&Ucirc;", "\u{DB}")
        .replace("&uuml;", "\u{FC}")
        .replace("&Uuml;", "\u{DC}")
        .replace("&yuml;", "\u{FF}")
        .replace("&Yuml;", "\u{178}")
        .replace("&oelig;", "\u{153}")
        .replace("&OElig;", "\u{152}")
        .replace("&aelig;", "\u{E6}")
        .replace("&AElig;", "\u{C6}")
        .replace("&euro;", "\u{20AC}")
        .trim()
        .to_string()
}
