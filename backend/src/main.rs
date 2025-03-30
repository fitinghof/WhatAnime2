mod error;
mod utility;
mod what_anime;

use anisong_api::AnisongAPIR;

use database_api::DatabaseR;
use spotify_api::SpotifyAPIR;
use what_anime::WhatAnime;

#[tokio::main]
async fn main() {
    let database = DatabaseR::new(4).await;
    let anisong = AnisongAPIR::new();
    let spotify: SpotifyAPIR<20> = SpotifyAPIR::new();
    let what_anime = WhatAnime::new(database, spotify, anisong);

    what_anime.run().await;
}
