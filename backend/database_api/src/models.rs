use anilist_api::models::*;
use anisong_api::models::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SpotifySongId(String);

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SpotifyArtistId(String);

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnime {
    #[sqlx(flatten)]
    anisongdb_anime: AnisongAnime,
    #[sqlx(flatten)]
    anilist_anime: Media,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SimplifiedAnisongSong {
    #[sqlx(rename = "song_id")]
    pub id: SongID,
    #[sqlx(rename = "song_name")]
    pub name: String,
    pub artist_name: String,
    pub composer_name: String,
    pub arranger_name: String,
    #[sqlx(rename = "song_category")]
    pub category: SongCategory,
    #[sqlx(rename = "song_length")]
    pub length: Option<f64>,
    #[sqlx(rename = "song_is_dub")]
    pub is_dub: bool,
    pub hq: Option<String>,
    pub mq: Option<String>,
    pub audio: Option<String>,
    pub artists: Vec<AnisongArtistID>,
    pub composers: Vec<AnisongArtistID>,
    pub arrangers: Vec<AnisongArtistID>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnisong {
    #[sqlx(flatten)]
    anisongdb_anime: AnisongAnime,
    #[sqlx(flatten)]
    anilist_anime: Media,
    #[sqlx(flatten)]
    song: SimplifiedAnisongSong,
    #[sqlx(flatten)]
    bind: DBAnisongBind,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct SimplifiedArtist {
    names: Vec<String>,
    id: AnisongArtistID,
    line_up_id: i32,
    group_ids: Vec<AnisongArtistID>,
    member_ids: Vec<AnisongArtistID>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnisongBind {
    pub song_id: Option<SongID>,
    pub anime_ann_id: Option<AnnAnimeID>,
    pub song_ann_id: SongAnnId,

    pub difficulty: Option<f64>,
    pub song_type: SongIndex,
    pub is_rebroadcast: bool,
}
