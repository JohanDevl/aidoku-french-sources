use aidoku::alloc::{format, string::ToString, String, Vec};

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

fn month_to_number(month: &str) -> Option<i32> {
	match month {
		"janvier" | "january" => Some(1),
		"février" | "february" => Some(2),
		"mars" | "march" => Some(3),
		"avril" | "april" => Some(4),
		"mai" | "may" => Some(5),
		"juin" | "june" => Some(6),
		"juillet" | "july" => Some(7),
		"août" | "august" => Some(8),
		"septembre" | "september" => Some(9),
		"octobre" | "october" => Some(10),
		"novembre" | "november" => Some(11),
		"décembre" | "december" => Some(12),
		_ => None,
	}
}

fn parse_absolute_date(text: &str) -> Option<i64> {
	let parts: Vec<&str> = text.split_whitespace().collect();

	if parts.len() < 3 {
		return None;
	}

	let (day, month, year) = if parts[0].chars().all(|c| c.is_ascii_digit()) {
		(parts[0], parts[1], parts[2])
	} else {
		(parts[1].trim_end_matches(','), parts[0], parts[2])
	};

	let day = day.parse::<i32>().ok()?;
	let month = month_to_number(month)?;
	let year = year.parse::<i32>().ok()?;

	// Calculate leap years between 1970 and year correctly
	let mut leap_years = 0;
	for y in 1970..year {
		if (y % 4 == 0 && y % 100 != 0) || (y % 400 == 0) {
			leap_years += 1;
		}
	}

	// Check if current year is leap and we're past February
	let is_leap_year = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
	let days_in_months = if is_leap_year && month > 2 {
		[0, 31, 60, 91, 121, 152, 182, 213, 244, 274, 305, 335]
	} else {
		[0, 31, 59, 90, 120, 151, 181, 212, 243, 273, 304, 334]
	};

	let days_since_epoch = (year - 1970) * 365 + leap_years;
	let month_days = days_in_months[(month - 1) as usize];
	let total_days = days_since_epoch + month_days + day - 1;

	Some((total_days as i64) * 86400)
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
