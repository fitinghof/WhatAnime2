use database_api::models::DBAnisong;
use serde::{Deserialize, Serialize};
use spotify_api::models::TrackObject;
use what_anime_shared::{ImageURL, SpotifyTrackID};

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Update {
    NoUpdates,
    LoginRequired,
    UnAuthorized,
    NotPlaying,
    NewSong(SongUpdate),
}

#[derive(Serialize, Deserialize)]
pub struct SongUpdate {
    pub song_info: SongInfo,
    pub anisongs: Anisongs,
}

#[derive(Serialize, Deserialize)]
pub struct SongInfo {
    pub song_name: String,
    pub song_artists: Vec<String>,
    pub album_image: ImageURL,
    pub spotify_song_id: SpotifyTrackID,
}

impl SongInfo {
    pub fn from_track(track: &TrackObject) -> Self {
        Self {
            song_name: track.name.clone(),
            song_artists: track.artists.iter().map(|a| a.name.clone()).collect(),
            album_image: track.album.images[0].clone(),
            spotify_song_id: track.id.clone(),
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Anisongs {
    Hit(NewSongHit),
    Miss(NewSongMiss),
}

#[derive(Serialize, Deserialize)]
pub struct NewSongHit {
    pub hits: Vec<DBAnisong>,
    pub more_by_artists: Vec<DBAnisong>,
    pub certainty: i32,
}

#[derive(Serialize, Deserialize)]
pub struct NewSongMiss {
    pub possible: Vec<DBAnisong>,
}
