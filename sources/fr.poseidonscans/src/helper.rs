use aidoku::{
	alloc::{String, Vec},
	imports::net::Request,
	Result,
};
use crate::BASE_URL;

pub fn urlencode(string: String) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		// RFC 3986 unreserved characters: A-Z a-z 0-9 - _ . ~
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
			|| curr == b'-' || curr == b'_' || curr == b'.' || curr == b'~'
		{
			result.push(curr);
		} else {
			result.push(b'%');
			result.push(hex[curr as usize >> 4]);
			result.push(hex[curr as usize & 15]);
		}
	}

	String::from_utf8(result).unwrap_or_default()
}

pub fn build_api_request(url: &str) -> Result<Request> {
	Ok(Request::get(url)?
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
		.header("Accept", "application/json, text/plain, */*")
		.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
		.header("Accept-Encoding", "gzip, deflate, br")
		.header("Referer", BASE_URL)
		.header("Origin", BASE_URL)
	)
}

pub fn build_html_request(url: &str) -> Result<Request> {
	Ok(Request::get(url)?
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36")
		.header("Accept", "text/html,application/xhtml+xml,application/xml;q=0.9,image/webp,*/*;q=0.8")
		.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
		.header("Referer", BASE_URL)
	)
}

