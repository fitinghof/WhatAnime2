mod error;
mod utility;
mod what_anime;

use anisong_api::{self, AnisongAPI, models::AnisongArtistID};

#[tokio::main]
async fn main() {
    let a = anisong_api::AnisongAPIR::new();
    println!(
        "{:?}",
        a.artist_id_search(vec![AnisongArtistID(1)]).await.unwrap()
    );
    //let what_anime = WhatAnime::new();
}
