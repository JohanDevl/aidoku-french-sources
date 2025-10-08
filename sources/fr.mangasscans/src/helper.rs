use aidoku::alloc::{String, Vec, format};
use aidoku::FilterValue;

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

pub fn build_filter_params(filters: Vec<FilterValue>) -> String {
    let mut params = Vec::new();

    for filter in filters {
        match filter {
            FilterValue::Select { id, value } => {
                let filter_id = id.as_str();

                match filter_id {
                    "status" => {
                        let status = match value.as_str() {
                            "En cours" => "ongoing",
                            "Complété" => "completed",
                            "En pause" => "hiatus",
                            "Partenaire" => "partenaire",
                            _ => "",
                        };
                        if !status.is_empty() {
                            params.push(format!("status={}", status));
                        }
                    }
                    "type" => {
                        let typ = match value.as_str() {
                            "Manga" => "manga",
                            "Manhwa" => "manhwa",
                            "Manhua" => "manhua",
                            "Comic" => "comic",
                            "LN" => "novel",
                            _ => "",
                        };
                        if !typ.is_empty() {
                            params.push(format!("type={}", typ));
                        }
                    }
                    "order" => {
                        let order = match value.as_str() {
                            "A-Z" => "title",
                            "Z-A" => "titlereverse",
                            "Mise à jour" => "update",
                            "Ajout" => "latest",
                            "Popularité" => "popular",
                            _ => "",
                        };
                        if !order.is_empty() {
                            params.push(format!("order={}", order));
                        }
                    }
                    _ => {}
                }
            }
            FilterValue::MultiSelect { id, included, excluded } => {
                if id.as_str() == "genre" {
                    for genre_id in included {
                        if !genre_id.is_empty() {
                            params.push(format!("genre%5B%5D={}", genre_id));
                        }
                    }
                    for genre_id in excluded {
                        if !genre_id.is_empty() {
                            params.push(format!("genre%5B%5D=-{}", genre_id));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if params.is_empty() {
        String::new()
    } else {
        format!("&{}", params.join("&"))
    }
}
