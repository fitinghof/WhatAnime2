use database_api::Database;
use database_api::DatabaseR;
use database_api::models::AnisongArtistID;
use dotenvy;
use std::{fs::File, io::Read};
use what_anime_shared::SpotifyArtistID;

#[tokio::main]
async fn main() {
    dotenvy::from_path("../../dev.env").ok();

    let mut file = File::open("data.csv").expect("Failed to open file");
    let mut s = String::new();
    let n = file.read_to_string(&mut s).expect("Failed to read file");

    let lines: Vec<(AnisongArtistID, SpotifyArtistID)> = s
        .lines()
        .map(|l| {
            let line: Vec<&str> = l.split(",").collect();
            let anime_id = line[0]
                .parse::<AnisongArtistID>()
                .expect("if this failes we cannot do anything");
            let spotify_id = line[1];
            (
                anime_id,
                SpotifyArtistID(spotify_id.trim_matches('\"').to_string()),
            )
        })
        .collect();

    println!("{:?}", lines.first());

    let db = DatabaseR::new(1).await;
    println!("Inserted rows: {}", db.bind_artists(lines).await);
}
