use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fs::File;

#[derive(Serialize, Deserialize, Debug)]
struct JobPosting {
    id: String,
    title: String,
    location: String,
    description: String,
    qualifications: String,
    responsibilities: String,
    company: String,
    url: String,
    date_posted: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let base_url = "https://careers.google.com/api/v3/search/";

    let mut headers = HeaderMap::new();
    headers.insert(
        USER_AGENT,
        HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let static_params = [
        ("company", "Google"),
        ("employment_type", "FULL_TIME"),
        ("location", ""),
        ("distance", "50"),
        ("page_size", "100"),
        ("sort_by", "date"), 
    ];

    let mut all_jobs = Vec::new();
    let mut page = 1;
    let mut has_more = true;

    while has_more && page < 2{
        println!("Fetching page {}", page);

        // Create the page string outside the params vec
        let page_str = page.to_string();
        
        // Create params for this iteration
        let mut params = static_params.to_vec();
        params.push(("page", page_str.as_str()));

        let resp = client
            .get(base_url)
            .query(&params)
            .send()
            .await?;

        if !resp.status().is_success() {
            eprintln!("Request failed on page {}: {}", page, resp.status());
            break;
        }

        // Print the response body to check its structure
        let body = resp.text().await?;
        println!("Response body for page {}:\n{}", page, body);

        // Parse the body as JSON
        let response: Value = serde_json::from_str(&body)?;

        // Check if the "jobs" field exists and is an array
        if let Some(jobs) = response.get("jobs").and_then(|v| v.as_array()) {
            if jobs.is_empty() {
                has_more = false;
                continue;
            }

            for job in jobs {
                let job_posting = JobPosting {
                    id: job["id"].as_str().unwrap_or("").to_string(),
                    title: job["title"].as_str().unwrap_or("").to_string(),
                    location: job["locations"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .map(|loc| loc.as_str().unwrap_or("").to_string())
                                .collect::<Vec<_>>()
                                .join(", ")
                        })
                        .unwrap_or_default(),
                    description: job["description"]["content"].as_str().unwrap_or("").to_string(),
                    qualifications: job["qualifications"]["content"].as_str().unwrap_or("").to_string(),
                    responsibilities: job["responsibilities"]["content"].as_str().unwrap_or("").to_string(),
                    company: "Google".to_string(),
                    url: format!(
                        "https://careers.google.com/jobs/results/{}",
                        job["id"].as_str().unwrap_or("")
                    ),
                    date_posted: job["posted_date"].as_str().unwrap_or("").to_string(),
                };

                all_jobs.push(job_posting);
            }
        } else {
            println!("No jobs found in response, stopping.");
            has_more = false;
        }

        // Respect rate limits
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        page += 1;
    }

    // Write to JSON file
    let file = File::create("google_jobs.json")?;
    serde_json::to_writer_pretty(file, &all_jobs)?;

    println!("Found {} jobs", all_jobs.len());
    println!("Data has been written to google_jobs.json");

    Ok(())
}
