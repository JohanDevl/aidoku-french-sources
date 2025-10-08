use aidoku::{
	alloc::{String, Vec, format, string::ToString},
};

pub fn urlencode(text: &str) -> String {
	let mut result = String::new();
	
	for byte in text.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
				result.push(byte as char);
			}
			b' ' => {
				result.push_str("+"); // Utiliser + au lieu de %20 pour les espaces dans les paramètres de recherche
			}
			_ => {
				result.push_str(&format!("%{:02X}", byte));
			}
		}
	}
	
	result
}

pub fn urlencode_path(text: &str) -> String {
	let mut result = String::new();
	
	for byte in text.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
				result.push(byte as char);
			}
			b' ' => {
				result.push_str("%20"); // Utiliser %20 pour les espaces dans les chemins d'URL (CDN)
			}
			_ => {
				result.push_str(&format!("%{:02X}", byte));
			}
		}
	}
	
	result
}

// Version alternative d'URL encoding spécialement pour les recherches
// Utilise %20 au lieu de + pour les espaces (au cas où + ne marche pas)
pub fn urlencode_search_fallback(text: &str) -> String {
	let mut result = String::new();
	
	for byte in text.bytes() {
		match byte {
			b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
				result.push(byte as char);
			}
			b' ' => {
				result.push_str("%20"); // Utiliser %20 au lieu de +
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

pub fn _extract_number_from_url(url: &str, param: &str) -> Option<i32> {
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

pub fn _clean_text(text: &str) -> String {
	String::from(text.trim().replace('\n', " ").replace('\t', " ")
		.replace("  ", " "))
}

pub fn _parse_date_string(_date_str: &str) -> f64 {
	// Pour l'instant, retourner une valeur par défaut
	// Une implémentation plus sophistiquée serait nécessaire pour parser les dates françaises
	-1.0
}

pub fn _urldecode(text: &str) -> String {
	let mut result = String::new();
	let mut chars = text.chars();

	while let Some(ch) = chars.next() {
		match ch {
			'%' => {
				// Récupérer les deux caractères suivants
				if let (Some(c1), Some(c2)) = (chars.next(), chars.next()) {
					if let Ok(byte) = u8::from_str_radix(&format!("{}{}", c1, c2), 16) {
						result.push(byte as char);
					} else {
						// Si le décodage échoue, garder les caractères originaux
						result.push('%');
						result.push(c1);
						result.push(c2);
					}
				} else {
					result.push('%');
				}
			}
			'+' => result.push(' '),
			_ => result.push(ch),
		}
	}

	result
}

pub fn clean_url(url: &str) -> String {
	if url.contains("/scan/vf/") {
		url.replace("/scan/vf/", "")
	} else if url.contains("/scan_noir-et-blanc/vf/") {
		url.replace("/scan_noir-et-blanc/vf/", "")
	} else {
		url.to_string()
	}
}

pub fn is_one_piece_manga(manga_key: &str) -> bool {
	manga_key.contains("one-piece") || manga_key.contains("one_piece")
}
