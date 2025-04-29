mod fetch_anisong;
mod load_links;
mod parse_reports;

use std::io::Read;

use fetch_anisong::fetch_anisong;
use load_links::load_links;
use log::{error, info, warn};
use parse_reports::parse_reports;
const OPTIONS: &'static [&str] = &["Run anisong fetch", "Load links", "Parse Reports"];

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("tracing", log::LevelFilter::Warn)
        .target(env_logger::Target::Stdout)
        .init();

    for o in OPTIONS.iter().enumerate() {
        println!("{}: {}", o.0 + 1, o.1);
    }
    let mut inp = String::new();
    std::io::stdin()
        .read_line(&mut inp)
        .expect("Failed to parse input");
    if inp.len() > 10 {
        error!("Failed to get input or invalid input");
    }
    match inp.trim() {
        "1" => {
            if fetch_anisong().await {
                info!("Successfully fetched anisong")
            } else {
                warn!("Failed to fetch anisong")
            }
        }
        "2" => {
            if load_links().await {
                info!("Successfully loaded links")
            } else {
                warn!("Failed to load links")
            }
        }
        "3" => {
            if parse_reports().await {
                info!("Finished parsing reports")
            } else {
                warn!("Something went wrong while parsing reports")
            }
        }
        _ => {
            error!("invalid input");
        }
    }
}
