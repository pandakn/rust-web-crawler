use std::{
    error::Error,
    fs::{create_dir_all, File},
    io::Write,
};

use spider::{page::Page, tokio, url::Url, Client};
use spider_transformations::{
    transform_content,
    transformation::content::{transform_markdown, ReturnFormat, TransformConfig},
};

/// Processes a list of URLs in parallel using Tokio.
pub async fn from_urls_and_save_files(urls: Vec<String>) {
    let mut handles = Vec::new();

    for url in urls {
        let url_clone = url.clone();

        // Spawn a new async task for each URL
        let handle = tokio::spawn(async move {
            let client = Client::default();
            if let Err(e) = process_url(&url_clone, &client).await {
                eprintln!("Failed to process {}: {}", url_clone, e);
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        let _ = handle.await;
    }
}

async fn process_url(url: &str, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    let page = Page::new_page(url, client).await;

    let mut conf = TransformConfig::default();
    conf.return_format = ReturnFormat::Markdown;

    let markdown = transform_content(&page, &conf, &None, &None, &None);
    save_markdown_and_save_file(url, &markdown)?;

    Ok(())
}

pub fn from_html_content_and_save_file(url: &str, content: &str) {
    println!("{}", content);

    let markup = transform_markdown(content, false);

    println!("{}", markup);
    if let Err(e) = save_markdown_and_save_file(url, &markup) {
        eprintln!("Failed to save markdown file: {}", e);
    }
}

fn save_markdown_and_save_file(url: &str, markdown: &str) -> Result<(), Box<dyn Error>> {
    create_dir_all("output")?;

    let parsed_url = Url::parse(url)?;
    let segments: Vec<&str> = parsed_url
        .path_segments()
        .ok_or("Cannot extract path segments")?
        .filter(|s| !s.is_empty())
        .collect();

    let slug = segments.last().unwrap_or(&"index");
    let filename = format!("output/{}.md", slug);

    File::create(&filename)?.write_all(markdown.as_bytes())?;
    Ok(())
}
