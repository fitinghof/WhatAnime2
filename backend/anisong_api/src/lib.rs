pub mod anisong_api;
pub mod error;
pub mod models;

use crate::error::Result;
pub use anisong_api::AnisongAPIR;
use models::{Anisong, AnisongArtistID};

pub trait AnisongAPI {
    async fn artist_id_search(&self, ids: Vec<AnisongArtistID>) -> Result<Vec<Anisong>>;
    async fn full_search(&self, song_title: String, artist_names: Vec<String>) -> Vec<Anisong>;
    async fn get_exact_song(
        &self,
        song_title: String,
        artist_ids: Vec<AnisongArtistID>,
    ) -> Result<Vec<Anisong>>;
}
