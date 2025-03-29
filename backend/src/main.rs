mod error;
mod utility;
mod what_anime;

use anilist_api::{self, AnilistAPI};
use anisong_api::{
    self, AnisongAPI,
    models::{AnilistAnimeID, AnisongArtistID},
};

#[tokio::main]
async fn main() {
    let a = anisong_api::AnisongAPIR::new();
    let b = anilist_api::AnilistAPIR::new();
    println!(
        "{:?}",
        a.artist_id_search(vec![AnisongArtistID(1)]).await.unwrap()
    );

    println!("{:?}", b.fetch_one(AnilistAnimeID(1)).await.unwrap())
    //let what_anime = WhatAnime::new();
}
