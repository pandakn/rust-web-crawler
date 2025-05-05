use spider::website::Website;

/// Crawls a website and returns a list of visited URLs as `Vec<String>`.
pub async fn crawl_urls(target_domain: &str) -> Vec<String> {
    let mut website: Website = Website::new(target_domain)
        .with_stealth(true)
        .with_depth(3)
        .build()
        .unwrap();

    website.crawl().await;

    let links = website.get_all_links_visited().await;

    links.into_iter().map(|s| s.to_string()).collect()
}
