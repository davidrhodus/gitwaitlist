use clap::Parser;
use config::Config;
use csv::Writer;
use reqwest::header::{HeaderMap, HeaderValue};
use serde::Deserialize;
use std::error::Error;
use std::fs::File;

#[derive(Deserialize, Debug)]
struct Signup {
    email: String,
    name: Option<String>,
    signed_up_at: Option<String>,
}

/// CLI arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Output format: json or csv
    #[arg(short, long, default_value = "json")]
    format: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // Load configuration
    let settings = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()?;

    let api_key: String = settings.get("api.key")?;
    let waitlist_id: String = settings.get("api.waitlist_id")?;
    let base_url = format!("https://api.getwaitlist.com/api/v1/signup/waitlist/{}", waitlist_id);

    let client = reqwest::Client::new();
    let mut headers = HeaderMap::new();
    headers.insert("api-key", HeaderValue::from_str(&api_key)?);

    let mut signups = Vec::new();
    let mut offset = 0;
    let limit = 1000; // Adjust based on API documentation

    loop {
        println!("Fetching records starting from offset: {}", offset);
        let response = client
            .get(&base_url)
            .query(&[("offset", offset.to_string()), ("limit", limit.to_string())])
            .headers(headers.clone())
            .send()
            .await?;

        if response.status().is_success() {
            let mut page_signups: Vec<Signup> = response.json().await?;
            if page_signups.is_empty() {
                break; // No more records
            }
            signups.append(&mut page_signups);
            offset += limit; // Move to the next set
        } else {
            eprintln!("Failed to fetch records at offset {}: {}", offset, response.status());
            break;
        }
    }

    println!("Total signups fetched: {}", signups.len());

    match args.format.as_str() {
        "json" => {
            let emails: Vec<String> = signups.iter().map(|s| s.email.clone()).collect();
            std::fs::write("emails.json", serde_json::to_string_pretty(&emails)?)?;
            println!("Emails saved to emails.json");
        }
        "csv" => {
            let mut wtr = Writer::from_writer(File::create("emails.csv")?);
            wtr.write_record(&["Email", "Name", "Signed Up At"])?;
            for signup in &signups {
                wtr.write_record(&[
                    &signup.email,
                    &signup.name.clone().unwrap_or_else(|| "".to_string()),
                    &signup.signed_up_at.clone().unwrap_or_else(|| "".to_string()),
                ])?;
            }
            wtr.flush()?;
            println!("Emails saved to emails.csv");
        }
        _ => {
            eprintln!("Invalid format specified. Use 'json' or 'csv'.");
        }
    }

    Ok(())
}
