use core::str;

use rust_web_crawler::utils::fetch;
use spider::{
    hashbrown::HashSet,
    quick_xml::{events::Event, Reader},
    Client,
};

fn parse_sitemap_xml_locs(xml: &str) -> (Vec<String>, Vec<String>) {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut buf = Vec::new();
    let mut urls = Vec::new();
    let mut sub_sitemaps = Vec::new();
    let mut reading_loc = false;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) if e.name().as_ref() == b"loc" => reading_loc = true,
            Ok(Event::Text(e)) if reading_loc => {
                if let Ok(url) = e.unescape() {
                    let url = url.into_owned();
                    if url.ends_with(".xml") {
                        sub_sitemaps.push(url);
                    } else {
                        urls.push(url);
                    }
                }
                reading_loc = false;
            }
            Ok(Event::Eof) => break,
            Err(e) => {
                eprintln!("XML parsing error: {:?}", e);
                break;
            }
            _ => {}
        }
        buf.clear();
    }

    (urls, sub_sitemaps)
}

/// Parses `sitemap.xml` and returns the top-level URLs.
pub async fn fetch_sitemap_urls(domain: &str) -> Option<Vec<String>> {
    let url = format!("{}/sitemap.xml", domain);
    let client = Client::default();

    match fetch::fetch_content(&client, &url).await {
        Ok(xml) => {
            let (urls, _) = parse_sitemap_xml_locs(&xml);
            Some(urls)
        }
        Err(e) => {
            eprintln!("Failed to fetch or parse sitemap.xml from {}: {}", url, e);
            None
        }
    }
}

/// Extracts all URLs from multiple sitemap URLs, recursively.
pub async fn extract_all_urls_from_sitemaps(sitemap_urls: Vec<String>) -> Vec<String> {
    let client = Client::default();
    let mut visited_sitemaps = HashSet::new();
    let mut collected_urls = HashSet::new();

    for sitemap_url in sitemap_urls {
        extract_from_single_sitemap(
            &sitemap_url,
            &client,
            &mut visited_sitemaps,
            &mut collected_urls,
        )
        .await;
    }

    collected_urls.into_iter().collect()
}

async fn extract_from_single_sitemap(
    sitemap_url: &str,
    client: &Client,
    visited_sitemaps: &mut HashSet<String>,
    collected_urls: &mut HashSet<String>,
) {
    let clean_url = sitemap_url.trim_end_matches('/').to_string();

    if !visited_sitemaps.insert(clean_url.clone()) {
        println!("Skipping duplicate sitemap: {}", clean_url);
        return;
    }

    let xml = match fetch::fetch_content(client, &clean_url).await {
        Ok(content) => content,
        Err(_) => return,
    };

    let (urls, submaps) = parse_sitemap_xml_locs(&xml);

    collected_urls.extend(urls);

    for sub_url in submaps {
        Box::pin(extract_from_single_sitemap(
            &sub_url,
            client,
            visited_sitemaps,
            collected_urls,
        ))
        .await;
    }
}
