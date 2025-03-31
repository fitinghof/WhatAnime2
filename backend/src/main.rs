mod error;
mod utility;
mod what_anime;

use anisong_api::AnisongAPIR;

use database_api::DatabaseR;
use dotenvy;
use spotify_api::SpotifyAPIR;
use what_anime::WhatAnime;

#[tokio::main]
async fn main() {
    dotenvy::from_path("../dev.env").expect("Environment load must succed");
    let database = DatabaseR::new(4).await;
    let anisong = AnisongAPIR::new();
    let spotify: SpotifyAPIR<20> = SpotifyAPIR::new();
    let what_anime = WhatAnime::new(database, spotify, anisong);

    what_anime.run().await;
}
