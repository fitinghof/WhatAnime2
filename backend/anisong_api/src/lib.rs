pub mod anisong_api;
pub mod error;
pub mod models;

use crate::error::Result;
pub use anisong_api::AnisongAPIR;
use models::{Anisong, AnisongArtistID};

pub trait AnisongAPI {
    fn artist_id_search(
        &self,
        ids: Vec<AnisongArtistID>,
    ) -> impl std::future::Future<Output = Result<Vec<Anisong>>> + Send;
    fn full_search(
        &self,
        song_title: String,
        artist_names: Vec<String>,
    ) -> impl std::future::Future<Output = Vec<Anisong>> + Send;
    fn get_exact_song(
        &self,
        song_title: String,
        artist_ids: Vec<AnisongArtistID>,
    ) -> impl std::future::Future<Output = Result<Vec<Anisong>>> + Send;
}
