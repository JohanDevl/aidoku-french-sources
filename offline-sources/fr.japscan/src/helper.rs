use aidoku::alloc::{format, String, Vec};

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

pub fn extract_images_from_script(html: &str) -> Option<Vec<String>> {
	// Try multiple patterns for image URL arrays
	let patterns = ["imagesLink", "pages", "pagesLink", "imageUrls", "images"];

	for pattern in &patterns {
		if let Some(start) = html.find(pattern) {
			let after = &html[start..];
			if let Some(bracket_start) = after.find('[') {
				if let Some(bracket_end) = after.find(']') {
					let content = &after[bracket_start + 1..bracket_end];
					let urls: Vec<String> = content
						.split(',')
						.filter_map(|s| {
							let cleaned = s.trim().trim_matches('"').trim_matches('\'').trim();
							if cleaned.starts_with("http") {
								Some(String::from(cleaned))
							} else {
								None
							}
						})
						.collect();

					if !urls.is_empty() {
						return Some(urls);
					}
				}
			}
		}
	}

	None
}

pub fn extract_images_from_data_atad(html: &str) -> Option<Vec<String>> {
	// Extract the data-atad attribute containing encoded image URLs
	let data_atad_pattern = r#"data-atad=""#;
	let start_idx = html.find(data_atad_pattern)?;
	let after_start = &html[start_idx + data_atad_pattern.len()..];
	let end_idx = after_start.find('"')?;
	let encoded_data = &after_start[..end_idx];

	// Extract decryption keys from JavaScript variables
	let key_a = extract_js_variable(html, "kDoa")?;
	let key_b = extract_js_variable(html, "kDob")?;
	let key_c = extract_js_variable(html, "kDoc")?;

	// Decode using the keys
	decode_japscan_urls(encoded_data, &key_a, &key_b, &key_c)
}

fn extract_js_variable(html: &str, var_name: &str) -> Option<String> {
	let pattern = format!("var {}='", var_name);
	let start = html.find(&pattern)?;
	let after = &html[start + pattern.len()..];
	let end = after.find('\'')?;
	Some(String::from(&after[..end]))
}

fn decode_japscan_urls(encoded: &str, key_a: &str, key_b: &str, key_c: &str) -> Option<Vec<String>> {
	// JapScan uses a character substitution cipher
	// The keys determine the character mapping
	// Based on analysis: encoded string uses specific patterns that map to URL components

	let mut result = String::from(encoded);

	// Common separators in the encoded format
	result = result.replace("GaAaK", ":"); // Colon
	result = result.replace("z9w", "/");   // Forward slash
	result = result.replace("uMC7", ".");  // Dot
	result = result.replace("GdLo", "-");  // Hyphen
	result = result.replace("Gh4o", "?");  // Question mark
	result = result.replace("cbdl", "=");  // Equals
	result = result.replace("08Cd", "&");  // Ampersand

	// Decode based on keys - this is a simplified character mapping
	// The actual algorithm likely involves more complex substitution
	result = decode_with_keys(&result, key_a, key_b, key_c);

	// Split and filter to extract valid CDN URLs
	let mut urls = Vec::new();
	let cdn_base = "c4.japscan.vip";

	// Look for patterns that resemble the CDN URL structure
	// Format: /segment1/segment2/segment3/filename.ext?o=1
	for segment in result.split(&['\n', ' ', '\t'][..]) {
		if segment.contains(cdn_base) {
			// Reconstruct full URL if we found a CDN reference
			if segment.starts_with("http") {
				urls.push(String::from(segment));
			} else {
				urls.push(format!("https://{}", segment));
			}
		}
	}

	if !urls.is_empty() {
		Some(urls)
	} else {
		None
	}
}

fn decode_with_keys(encoded: &str, _key_a: &str, _key_b: &str, _key_c: &str) -> String {
	// Character substitution based on the keys
	// This is a simplified version - the real algorithm is more complex
	// TODO: Implement proper key-based decoding algorithm
	let mut result = String::from(encoded);

	// Build a character map based on patterns observed in the data
	// Each key influences different parts of the decoding

	// Key patterns mapping (observed from the encoded data)
	let substitutions = [
		("GE", "c"), ("vU", "4"), ("Nj", "1"), ("Fj", "2"),
		("nj", "3"), ("fj", "5"), ("vr", "6"), ("Fr", "7"),
		("nr", "8"), ("Vr", "9"), ("F0", "a"), ("n0", "b"),
		("f0", "c"), ("v0", "d"), ("N0", "e"), ("nM", "f"),
	];

	for (pattern, replacement) in &substitutions {
		result = result.replace(pattern, replacement);
	}

	result
}
