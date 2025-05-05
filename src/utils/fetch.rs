use core::str;

use spider::{utils::fetch_page_html_raw_only_html, Client};

/// Fetches a page and returns it as a UTF-8 string.
pub async fn fetch_content(client: &Client, url: &str) -> Result<String, String> {
    let response = fetch_page_html_raw_only_html(url, client).await;

    println!("HTTP Status Code: {}", response.status_code);

    if response.status_code != 200 {
        return Err(format!(
            "Failed to fetch content from {}. HTTP Status: {}",
            url, response.status_code
        ));
    }

    match response.content {
        Some(bytes) => str::from_utf8(&bytes)
            .map(str::to_string)
            .map_err(|e| format!("Failed to decode response as UTF-8: {}", e)),
        None => Err(format!("Failed to fetch content from: {}", url)),
    }
}
