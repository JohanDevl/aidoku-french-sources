use aidoku::alloc::{String, Vec};

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
    let _patterns = [
        r#"imagesLink\s*=\s*\[(.*?)\]"#,
        r#"var\s+imagesLink\s*=\s*\[(.*?)\]"#,
        r#"let\s+imagesLink\s*=\s*\[(.*?)\]"#,
        r#"const\s+imagesLink\s*=\s*\[(.*?)\]"#,
    ];

    for _pattern in &_patterns {
        if let Some(start) = html.find("imagesLink") {
            let after = &html[start..];
            if let Some(bracket_start) = after.find('[') {
                if let Some(bracket_end) = after.find(']') {
                    let content = &after[bracket_start + 1..bracket_end];
                    let urls: Vec<String> = content
                        .split(',')
                        .filter_map(|s| {
                            let cleaned = s
                                .trim()
                                .trim_matches('"')
                                .trim_matches('\'')
                                .trim();
                            if cleaned.starts_with("http") {
                                Some(String::from(cleaned) + "?y=1")
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
