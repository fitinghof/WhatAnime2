use crate::anisong::AnisongAPI;
use crate::database::Database;
use crate::spotify::{self, SpotifyAPI};

pub struct WhatAnime<D, S, A>
where
    D: Database,
    S: SpotifyAPI,
    A: AnisongAPI,
{
    database: D,
    spotify_api: S,
    anisong_api: A,
}

impl<D, S, A> WhatAnime<D, S, A>
where
    D: Database,
    S: SpotifyAPI,
    A: AnisongAPI,
{
    pub fn new(database: D, spotify_api: S, anisong_api: A) -> Self {
        Self {
            database,
            spotify_api,
            anisong_api,
        }
    }

    pub async fn run(&self) {
        println!("hello");
    }
}
