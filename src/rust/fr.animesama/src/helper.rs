use aidoku::{prelude::*, std::{String, Vec, current_date}};

pub fn urlencode(text: &str) -> String {
	let mut result = String::new();
	
	for byte in text.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
				result.push(byte as char);
			}
			b' ' => {
				result.push_str("%20");
			}
			_ => {
				result.push_str(&format!("%{:02X}", byte));
			}
		}
	}
	
	result
}

pub fn i32_to_string(num: i32) -> String {
	let mut result = String::new();
	let mut n = num;
	
	if n == 0 {
		return String::from("0");
	}
	
	if n < 0 {
		result.push('-');
		n = -n;
	}
	
	let mut digits = Vec::new();
	while n > 0 {
		digits.push((n % 10) as u8 + b'0');
		n /= 10;
	}
	
	for digit in digits.iter().rev() {
		result.push(*digit as char);
	}
	
	result
}

pub fn extract_number_from_url(url: &str, param: &str) -> Option<i32> {
	if let Some(start) = url.find(&format!("{}=", param)) {
		let start = start + param.len() + 1;
		if let Some(end) = url[start..].find('&') {
			let number_str = &url[start..start + end];
			return number_str.parse().ok();
		} else {
			let number_str = &url[start..];
			return number_str.parse().ok();
		}
	}
	None
}

pub fn clean_text(text: &str) -> String {
	String::from(text.trim().replace('\n', " ").replace('\t', " ")
		.replace("  ", " "))
}

pub fn parse_date_string(date_str: &str) -> f64 {
	// Pour l'instant, retourner la date actuelle
	// Une implémentation plus sophistiquée serait nécessaire pour parser les dates françaises
	current_date()
}