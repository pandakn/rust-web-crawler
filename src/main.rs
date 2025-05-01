use std::error::Error;

use spider::{hashbrown::HashSet, tokio, website::Website, Client};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let target_domain = "https://www.heygoody.com";

    println!("Fetching robots.txt from: {}", target_domain);
    match fetch_robots_txt(target_domain).await {
        Ok(Some(robots_content)) => {
            // println!("robots.txt content:\n{}", robots_content);
            let sitemap_urls = extract_sitemap_urls(&robots_content);
            if !sitemap_urls.is_empty() {
                // println!("\nFound Sitemap URLs:");
                for url in sitemap_urls.clone() {
                    let extracted_urls = extract_urls_from_sitemap(&url).await;
                    println!("\n\n- {}", url);
                    println!(
                        "Extracting URLs from sitemap length: {}",
                        extracted_urls.len()
                    );
                    println!("Extracted URLs from sitemap: {:?}\n\n\n\n", extracted_urls);

                    for url in extracted_urls {
                        match fetch_html(&url).await {
                            Ok(Some(html)) => {
                                println!("Fetched HTML:\n{}", html)
                            }
                            Ok(None) => println!("Page not found in website object."),
                            Err(e) => eprintln!("Error: {}", e),
                        }
                    }

                    // match fetch_html(target_domain).await {
                    //     Ok(Some(html)) => println!("Fetched HTML:\n{}", html),
                    //     Ok(None) => println!("Page not found in website object."),
                    //     Err(e) => eprintln!("Error: {}", e),
                    // }
                }
            } else {
                println!("\nNo Sitemap URLs found in robots.txt.");
            }
        }
        Ok(None) => {
            println!("\nrobots.txt not found on {}", target_domain);
        }
        Err(e) => {
            eprintln!("\nError fetching robots.txt: {}", e);
        }
    }

    Ok(())
}

async fn fetch_robots_txt(domain: &str) -> Result<Option<String>, Box<dyn Error>> {
    let robots_url = format!("{}/robots.txt", domain.trim_end_matches('/'));
    let client = Client::builder().build()?;
    let response = client.get(robots_url.clone()).send().await?;

    if response.status().is_success() {
        Ok(Some(response.text().await?))
    } else if response.status().as_u16() == 404 {
        Ok(None)
    } else {
        Err(format!(
            "Failed to fetch robots.txt from {}: {}",
            domain,
            response.status()
        )
        .into())
    }
}

fn extract_sitemap_urls(robots_content: &str) -> Vec<String> {
    let mut sitemap_urls = Vec::new();
    for line in robots_content.lines() {
        if line.to_lowercase().starts_with("sitemap:") {
            if let Some(url) = line.split_whitespace().nth(1) {
                sitemap_urls.push(url.to_string());
            }
        }
    }
    sitemap_urls
}

async fn extract_urls_from_sitemap(sitemap_url: &str) -> Vec<String> {
    let mut result = Vec::new();
    let mut visited = HashSet::new();
    let mut stack = vec![sitemap_url.to_string()];

    while let Some(url) = stack.pop() {
        if visited.contains(&url) {
            continue;
        }
        visited.insert(url.clone());

        let mut website = Website::new(&url);
        website.scrape_raw().await;

        for page in website.get_pages().unwrap().iter() {
            let html = page.get_html() else { continue };
            // Extract <loc> tags from HTML using string match (because regex is not allowed)
            for loc_tag in html.match_indices("<loc>") {
                if let Some(end) = html[loc_tag.0..].find("</loc>") {
                    let loc = html[loc_tag.0 + 5..loc_tag.0 + end].trim().to_string();
                    if loc.ends_with(".xml") {
                        stack.push(loc);
                    } else {
                        result.push(loc);
                    }
                }
            }
        }
    }

    result
}

pub async fn fetch_html(url: &str) -> Result<Option<String>, Box<dyn std::error::Error>> {
    let mut website = Website::new(url);
    website.configuration.depth = 1;

    // Perform scraping
    website.scrape_smart().await;

    // Check if the page exists and return the HTML
    if let Some(pages) = website.get_pages() {
        for page in pages.iter() {
            println!("{:?}", page);
            if page.get_url() == url {
                return Ok(Some(page.get_html()));
            }
        }
    }

    Ok(None)
}
// Process all sitemaps recursively and return a vector of all URLs
// async fn process_sitemaps_recursively(
//     initial_sitemaps: &[String],
// ) -> Result<Vec<String>, Box<dyn Error>> {
//     let client = Client::builder().build()?;
//     let mut all_urls = HashSet::new();
//     let mut sitemap_queue = VecDeque::new();
//     let mut processed_sitemaps = HashSet::new();

//     // Add initial sitemaps to queue
//     for sitemap in initial_sitemaps {
//         sitemap_queue.push_back(sitemap.clone());
//     }

//     // Process queue until empty
//     while let Some(sitemap_url) = sitemap_queue.pop_front() {
//         // Skip if we've already processed this sitemap
//         if processed_sitemaps.contains(&sitemap_url) {
//             continue;
//         }

//         println!("Processing sitemap: {}", sitemap_url);
//         processed_sitemaps.insert(sitemap_url.clone());

//         // Download sitemap content
//         match client.get(&sitemap_url).send().await {
//             Ok(response) => {
//                 let mut content = String::new();
//                 if let Err(e) = response.text().await {
//                     eprintln!("Error reading sitemap content: {}", e);
//                     continue;
//                 }

//                 // Process the sitemap XML
//                 // let (urls, nested_sitemaps) = parse_sitemap_xml(&content);

//                 // // Process the sitemap XML
//                 // let (urls, nested_sitemaps) = parse_sitemap_xml(&content);

//                 // // Add page URLs to result
//                 // for url in urls {
//                 //     all_urls.insert(url);
//                 // }

//                 // // Add nested sitemaps to the queue
//                 // for nested_sitemap in nested_sitemaps {
//                 //     if !processed_sitemaps.contains(&nested_sitemap) {
//                 //         sitemap_queue.push_back(nested_sitemap);
//                 //     }
//                 // }
//             }
//             Err(e) => {
//                 eprintln!("Error downloading sitemap {}: {}", sitemap_url, e);
//             }
//         }
//     }

//     Ok(all_urls.into_iter().collect())
// }
