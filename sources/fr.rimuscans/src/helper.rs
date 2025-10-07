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
    let text_lower = text.to_lowercase();

    // Try parsing absolute date first (format: "septembre 18, 2025")
    if let Some(timestamp) = parse_absolute_date(&text_lower) {
        return Some(timestamp);
    }

    let mut offset: i64 = 0;

    if text_lower.contains("aujourd'hui") || text_lower.contains("today") {
        return Some(0);
    }

    if text_lower.contains("hier") || text_lower.contains("yesterday") {
        return Some(-86400);
    }

    if let Some(value_str) = text_lower.split_whitespace().next() {
        if let Ok(value) = value_str.parse::<i64>() {
            if text_lower.contains("heure") || text_lower.contains("hour") {
                offset = value * 3600;
            } else if text_lower.contains("min") {
                offset = value * 60;
            } else if text_lower.contains("jour") || text_lower.contains("day") {
                offset = value * 86400;
            } else if text_lower.contains("semaine") || text_lower.contains("week") {
                offset = value * 86400 * 7;
            } else if text_lower.contains("mois") || text_lower.contains("month") {
                offset = value * 86400 * 30;
            } else if text_lower.contains("an") || text_lower.contains("year") {
                offset = value * 86400 * 365;
            }
            return Some(-offset);
        }
    }

    None
}

fn parse_absolute_date(text: &str) -> Option<i64> {
    // Parse format: "septembre 18, 2025" or "18 septembre 2025"
    let parts: Vec<&str> = text.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let month_num = match parts[0] {
        "janvier" | "january" => 1,
        "février" | "february" => 2,
        "mars" | "march" => 3,
        "avril" | "april" => 4,
        "mai" | "may" => 5,
        "juin" | "june" => 6,
        "juillet" | "july" => 7,
        "août" | "august" => 8,
        "septembre" | "september" => 9,
        "octobre" | "october" => 10,
        "novembre" | "november" => 11,
        "décembre" | "december" => 12,
        _ => {
            // Try second word as month (format: "18 septembre 2025")
            if parts.len() >= 3 {
                match parts[1] {
                    "janvier" | "january" => 1,
                    "février" | "february" => 2,
                    "mars" | "march" => 3,
                    "avril" | "april" => 4,
                    "mai" | "may" => 5,
                    "juin" | "june" => 6,
                    "juillet" | "july" => 7,
                    "août" | "august" => 8,
                    "septembre" | "september" => 9,
                    "octobre" | "october" => 10,
                    "novembre" | "november" => 11,
                    "décembre" | "december" => 12,
                    _ => return None,
                }
            } else {
                return None;
            }
        }
    };

    // Determine if format is "month day, year" or "day month year"
    let (day_str, year_str) = if parts[0].chars().all(|c| c.is_ascii_digit()) {
        // Format: "18 septembre 2025"
        (parts[0], parts[2])
    } else {
        // Format: "septembre 18, 2025"
        (parts[1].trim_end_matches(','), parts[2])
    };

    let day = day_str.parse::<i64>().ok()?;
    let year = year_str.parse::<i64>().ok()?;

    // Calculate Unix timestamp (simplified - doesn't account for leap years perfectly)
    let days_since_epoch = (year - 1970) * 365 + (year - 1969) / 4 - (year - 1901) / 100 + (year - 1601) / 400;
    let days_in_months = [0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334];
    let total_days = days_since_epoch + days_in_months[(month_num - 1) as usize] + day - 1;

    Some(total_days * 86400)
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
