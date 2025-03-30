mod error;
mod utility;
mod what_anime;

use anilist_api::{self, AnilistAPI};
use anisong_api::{self, AnisongAPI, AnisongAPIR, models::AnisongArtistID};

use database_api::DatabaseR;
use spotify_api::SpotifyAPIR;
use what_anime::WhatAnime;
use what_anime_shared::AnilistAnimeID;

#[tokio::main]
async fn main() {
    let database = DatabaseR::new(4).await;
    let anisong = AnisongAPIR::new();
    let spotify: SpotifyAPIR<20> = SpotifyAPIR::new();
    let what_anime = WhatAnime::new(database, spotify, anisong);

    what_anime.run().await;
}
