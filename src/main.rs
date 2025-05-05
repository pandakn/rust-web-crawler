mod crawler;
mod sitemap;

use core::str;

use rust_web_crawler::utils::{cli, fetch, markdown_transformer};
use spider::{tokio, Client};

#[tokio::main]
async fn main() {
    let target_domain_string = cli::parse_arguments().unwrap_or_else(|| {
        eprintln!("\x1b[31;1mError:\x1b[0m \x1b[31mMissing target domain.\x1b[0m");
        cli::print_usage();
        std::process::exit(1);
    });

    let target_domain: &str = target_domain_string.trim_end_matches("/").as_ref();

    println!("Fetching robots.txt from: {}", target_domain);
    let sitemap_urls_result = fetch_sitemaps_from_robots_txt(target_domain).await;

    let mut extracted_urls: Vec<String> = Vec::new();

    match sitemap_urls_result {
        Some(sitemap_urls) => {
            println!("\nFrom robots.txt len: {}", sitemap_urls.clone().len());
            println!("Sitemap URLs from robots.txt: {:?}\n\n", sitemap_urls);

            let urls = sitemap::extract_all_urls_from_sitemaps(sitemap_urls).await;
            extracted_urls.extend(urls);
        }
        None => {
            println!("\nNo sitemap URLs found.");
            println!("Fetching sitemap.xml from: {}", target_domain);
            let sitemap_urls = sitemap::fetch_sitemap_urls(target_domain).await;

            match sitemap_urls {
                Some(sitemap_urls) => {
                    println!("\nFrom sitemap.xml len: {}", sitemap_urls.clone().len());
                    println!("Sitemap URLs from sitemap.xml: {:?}\n\n", sitemap_urls);

                    let urls = sitemap::extract_all_urls_from_sitemaps(sitemap_urls).await;
                    extracted_urls.extend(urls);
                }
                None => {
                    println!("No sitemap.xml found.");
                    let urls = crawler::crawl_urls(target_domain).await;

                    extracted_urls.extend(urls);
                }
            }
        }
    }

    println!("Extracted URLs len: {}", extracted_urls.len());
    markdown_transformer::from_urls_and_save_files(extracted_urls.clone()).await;
}

/// Parses the `robots.txt` file of the given domain to extract all `Sitemap:` URLs.
///
/// Returns a list of sitemap URLs if found.
async fn fetch_sitemaps_from_robots_txt(domain: &str) -> Option<Vec<String>> {
    let robots_url = format!("{}/robots.txt", domain);
    let content_response = fetch::fetch_content(&Client::default(), &robots_url).await;

    match content_response {
        Ok(content) => match str::from_utf8(content.as_bytes()) {
            Ok(text) => {
                let sitemap: Vec<String> = text
                    .lines()
                    .filter(|line| line.starts_with("Sitemap:"))
                    .map(|line| line.replace("Sitemap:", "").trim().to_string())
                    .collect();
                Some(sitemap)
            }
            Err(e) => {
                eprintln!("Failed to parse robots.txt content as UTF-8: {}", e);
                None
            }
        },
        Err(e) => {
            eprintln!("Failed to fetch robots.txt from {}: {}", robots_url, e);
            None
        }
    }
}
