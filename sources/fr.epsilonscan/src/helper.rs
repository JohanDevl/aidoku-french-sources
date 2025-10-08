use aidoku::alloc::{String, format};

pub fn urlencode(string: String) -> String {
    let mut result = String::new();
    for byte in string.as_bytes() {
        match *byte {
            b'0'..=b'9' | b'A'..=b'Z' | b'a'..=b'z' | b'-' | b'_' | b'.' | b'~' => {
                result.push(*byte as char);
            }
            b' ' => result.push_str("%20"),
            _ => {
                result.push('%');
                let hex = format!("{:02X}", byte);
                result.push_str(&hex);
            }
        }
    }
    result
}

pub fn get_image_url(node: &aidoku::imports::html::Element) -> String {
    node.attr("data-src")
        .or_else(|| node.attr("data-lazy-src"))
        .or_else(|| node.attr("src"))
        .or_else(|| node.attr("data-cfsrc"))
        .unwrap_or_default()
}
