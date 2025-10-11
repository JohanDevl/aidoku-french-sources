use aidoku::{alloc::{String, Vec}, Result, AidokuError, imports::net::Request};

pub fn urlencode(string: &str) -> String {
	let mut result: Vec<u8> = Vec::with_capacity(string.len() * 3);
	let hex = "0123456789abcdef".as_bytes();
	let bytes = string.as_bytes();

	for byte in bytes {
		let curr = *byte;
		if (b'a'..=b'z').contains(&curr)
			|| (b'A'..=b'Z').contains(&curr)
			|| (b'0'..=b'9').contains(&curr)
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

pub fn validate_json_response(response: &str) -> Result<()> {
	if response.trim_start().starts_with('<') ||
	   response.contains("403 Forbidden") ||
	   response.contains("Access Denied") {
		return Err(AidokuError::message("Invalid API response"));
	}
	Ok(())
}

pub fn build_request(url: &str) -> Result<Request> {
	Ok(Request::get(url)?
		.header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/122.0.0.0 Safari/537.36")
		.header("Accept", "application/json, text/plain, */*")
		.header("Accept-Language", "fr-FR,fr;q=0.9,en;q=0.8")
		.header("Accept-Encoding", "gzip, deflate, br")
		.header("DNT", "1")
		.header("Connection", "keep-alive")
		.header("Upgrade-Insecure-Requests", "1")
		.header("Referer", "https://phenix-scans.com/")
		.header("Origin", "https://phenix-scans.com"))
}

