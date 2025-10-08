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

pub fn parse_chapter_date(text: &str) -> Option<i64> {
    let text_lower = text.to_lowercase();

    let parts: Vec<&str> = text_lower.split_whitespace().collect();

    if parts.len() < 3 {
        return None;
    }

    let mut month_num = 0;
    let mut month_index = 0;

    for (i, part) in parts.iter().enumerate() {
        month_num = match *part {
            "janvier" | "january" => 1,
            "février" | "february" | "fevrier" => 2,
            "mars" | "march" => 3,
            "avril" | "april" => 4,
            "mai" | "may" => 5,
            "juin" | "june" => 6,
            "juillet" | "july" => 7,
            "août" | "august" | "aout" => 8,
            "septembre" | "september" => 9,
            "octobre" | "october" => 10,
            "novembre" | "november" => 11,
            "décembre" | "december" | "decembre" => 12,
            _ => continue,
        };

        if month_num > 0 {
            month_index = i;
            break;
        }
    }

    if month_num == 0 {
        return None;
    }

    let day_index = if month_index > 0 && parts[month_index - 1].chars().all(|c| c.is_ascii_digit()) {
        month_index - 1
    } else if month_index + 1 < parts.len() {
        month_index + 1
    } else {
        return None;
    };

    let year_index = if day_index < month_index {
        if month_index + 1 < parts.len() { month_index + 1 } else { return None; }
    } else {
        if day_index + 1 < parts.len() { day_index + 1 } else { return None; }
    };

    let day = parts[day_index].trim_end_matches(',').parse::<i64>().ok()?;
    let year = parts[year_index].trim_end_matches(',').parse::<i64>().ok()?;

    let mut total_days: i64 = 0;

    for y in 1970..year {
        if is_leap_year(y) {
            total_days += 366;
        } else {
            total_days += 365;
        }
    }

    let days_in_month = [31, if is_leap_year(year) { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    for m in 1..month_num {
        total_days += days_in_month[(m - 1) as usize];
    }

    total_days += day - 1;

    Some(total_days * 86400)
}

fn is_leap_year(year: i64) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

pub fn extract_and_clean_title(title: &str) -> (String, Option<i64>) {
    let parts: Vec<&str> = title.split_whitespace().collect();
    let mut cleaned_parts = Vec::new();
    let mut date_found = false;
    let mut skip_count = 0;

    for (_i, part) in parts.iter().enumerate() {
        if skip_count > 0 {
            skip_count -= 1;
            continue;
        }

        let part_lower = part.to_lowercase();

        let is_month = matches!(part_lower.as_str(),
            "janvier" | "février" | "fevrier" | "mars" | "avril" | "mai" | "juin" |
            "juillet" | "août" | "aout" | "septembre" | "octobre" | "novembre" | "décembre" | "decembre" |
            "january" | "february" | "march" | "april" | "may" | "june" |
            "july" | "august" | "september" | "october" | "november" | "december"
        );

        if is_month {
            date_found = true;
            skip_count = 2;
            continue;
        }

        if date_found && (part.chars().all(|c| c.is_ascii_digit() || c == ',')) {
            continue;
        }

        cleaned_parts.push(*part);
    }

    let cleaned_title = cleaned_parts.join(" ");
    let date = if date_found { parse_chapter_date(title) } else { None };

    (cleaned_title, date)
}
