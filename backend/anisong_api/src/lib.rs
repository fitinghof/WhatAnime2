pub mod anisong_api;
pub mod error;
pub mod models;

use crate::error::Result;
pub use anisong_api::AnisongAPIR;
use models::{Anisong, AnisongArtistID, Release};

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
    fn get_anime_season(
        &self,
        release: Release,
    ) -> impl std::future::Future<Output = Result<Vec<Anisong>>> + Send;
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;

    use super::*;
    const TEST_INPUT: &str = include_str!("testParse1.json");
    const TEST_INPUT2: &str = include_str!("testParse2.json");
    const TEST_INPUT3: &str = include_str!("testParse3.json");

    #[test]
    fn test_parse() {
        let _: Vec<Anisong> = serde_json::from_str(TEST_INPUT).expect("Parsing Failed");
        let _: Vec<Anisong> = serde_json::from_str(TEST_INPUT2).expect("Parsing Failed");

        // Checks to make sure that Options aren't just ommited due to missnaming or something
        let anisong: Anisong = serde_json::from_str(TEST_INPUT3).expect("Parsing Failed");
        assert!(!anisong.anime.alt_name.is_empty());
        assert!(anisong.anime.alt_name[0] == "some");
        assert!(anisong.anime.anime_type.is_some());
        assert!(anisong.anime.vintage.is_some());
        assert!(anisong.song.audio.is_some());
        assert!(anisong.song.hq.is_some());
        assert!(anisong.song.mq.is_some());
        assert!(anisong.anisong_bind.difficulty.is_some());
        assert!(anisong.song.length.is_some());
    }

    #[tokio::test]
    async fn test_fetch() {
        let anisong = AnisongAPIR::new();
        let _: Vec<Anisong> = anisong
            .artist_id_search(vec![AnisongArtistID(1)])
            .await
            .unwrap();

        let release = Release {
            season: what_anime_shared::ReleaseSeason::Fall,
            year: 2000,
        };

        let a = anisong
            .get_anime_season(release)
            .await
            .expect("Fetch failed");
        assert!(!a.is_empty());
    }
}
