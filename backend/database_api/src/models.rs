use std::collections::HashSet;

use anilist_api::models::*;
use anisong_api::models::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Row, postgres::PgRow};

use what_anime_shared::{ImageURL, ReleaseSeason, SongID, SpotifyTrackID, SpotifyUser};
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DBAnime {
    // AnisongDB stuff
    pub ann_id: AnnAnimeID,
    pub eng_name: String,
    pub jpn_name: String,
    pub alt_name: Vec<String>,
    pub vintage: Option<Release>,
    pub linked_ids: AnimeListLinks,
    pub anime_type: Option<AnimeType>,
    pub anime_index: AnimeIndex,

    // Anilist stuff
    pub mean_score: Option<i32>,
    pub banner_image: Option<ImageURL>,
    pub cover_image: CoverImage,
    pub format: Option<MediaFormat>,
    pub genres: Vec<String>,
    pub source: Option<MediaSource>,
    pub studios: StudioConnection,
    pub tags: Vec<MediaTag>,
    pub trailer: Option<MediaTrailer>,
    pub episodes: Option<i32>,
    pub season: Option<ReleaseSeason>,
    pub season_year: Option<i32>,
}

pub struct Report {
    pub track_id: Option<SpotifyTrackID>,
    pub song_ann_id: Option<SongAnnId>,
    pub message: String,
    pub user: SpotifyUser,
}

impl From<(AnisongAnime, Option<Media>)> for DBAnime {
    fn from((anisong, anilist): (AnisongAnime, Option<Media>)) -> Self {
        match anilist {
            Some(ani) => Self {
                ann_id: anisong.ann_id,
                eng_name: anisong.eng_name,
                jpn_name: anisong.jpn_name,
                alt_name: anisong.alt_name,
                vintage: anisong.vintage,
                linked_ids: anisong.linked_ids,
                anime_type: anisong.anime_type,
                anime_index: anisong.anime_index,
                mean_score: Some(ani.mean_score),
                banner_image: ani.banner_image,
                cover_image: ani.cover_image,
                format: ani.format,
                genres: ani.genres,
                source: ani.source,
                studios: ani.studios,
                tags: ani.tags,
                trailer: ani.trailer,
                episodes: ani.episodes,
                season: ani.season,
                season_year: ani.season_year,
            },
            None => Self {
                ann_id: anisong.ann_id,
                eng_name: anisong.eng_name,
                jpn_name: anisong.jpn_name,
                alt_name: anisong.alt_name,
                vintage: anisong.vintage,
                linked_ids: anisong.linked_ids,
                anime_type: anisong.anime_type,
                anime_index: anisong.anime_index,
                mean_score: None,
                banner_image: None,
                cover_image: CoverImage::default(),
                format: None,
                genres: vec![],
                source: None,
                studios: StudioConnection::default(),
                tags: vec![],
                trailer: None,
                episodes: None,
                season: None,
                season_year: None,
            },
        }
    }
}

impl DBAnime {
    pub fn combine(mut anisongs: Vec<AnisongAnime>, mut anilists: Vec<Media>) -> Vec<DBAnime> {
        if anisongs.is_empty() {
            return vec![];
        }
        anisongs.sort_by(|a, b| a.linked_ids.anilist.cmp(&b.linked_ids.anilist));
        anilists.sort_by(|a, b| a.id.cmp(&b.id));

        let mut anilists = anilists.into_iter();
        let mut anilist_o = anilists.next();

        let mut anisongs = anisongs.into_iter();
        let mut anisong = anisongs.next().unwrap();

        let mut db_animes = Vec::with_capacity(anisongs.len());
        loop {
            if let (Some(sid), Some(anilist)) = (anisong.linked_ids.anilist, anilist_o.take()) {
                match anilist.id {
                    i if i == sid => {
                        db_animes.push(DBAnime::from((anisong, Some(anilist))));
                        match anisongs.next() {
                            Some(a) => anisong = a,
                            None => return db_animes,
                        }
                    }
                    i if i > sid => anilist_o = anilists.next(),
                    _ => match anisongs.next() {
                        Some(a) => {
                            db_animes.push(DBAnime::from((anisong, None)));
                            anisong = a;
                        }
                        None => return db_animes,
                    },
                }
            }
            match anisongs.next() {
                Some(a) => {
                    db_animes.push(DBAnime::from((anisong, None)));
                    anisong = a;
                }
                None => return db_animes,
            }
        }
    }
}

impl FromRow<'_, PgRow> for DBAnime {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let tag_ids: Vec<TagID> = row.get("anime_tags_id");
        let tag_names: Vec<String> = row.get("anime_tags_name");
        let tags = tag_ids
            .into_iter()
            .zip(tag_names.into_iter())
            .map(|(id, name)| MediaTag { id, name })
            .collect();
        let vintage_release_season: Option<ReleaseSeason> =
            row.try_get("vintage_release_season").ok();
        let vintage_release_year: Option<i32> = row.try_get("vintage_release_season").ok();
        let vintage =
            if let (Some(season), Some(year)) = (vintage_release_season, vintage_release_year) {
                Some(Release { season, year })
            } else {
                None
            };
        Ok(Self {
            ann_id: row.get("anime_ann_id"),
            eng_name: row.get("anime_eng_name"),
            jpn_name: row.get("anime_jpn_name"),
            alt_name: row.get("anime_alt_names"),
            vintage: vintage,
            linked_ids: AnimeListLinks::from_row(row)?,
            anime_type: row.get("anime_type"),
            anime_index: AnimeIndex::from_row(row)?,
            mean_score: row.get("anime_mean_score"),
            banner_image: row.get("anime_banner_image"),
            cover_image: CoverImage::from_row(row)?,
            format: row.get("anime_format"),
            genres: row.get("anime_genres"),
            source: row.get("anime_source"),
            studios: StudioConnection::from_row(row)?,
            tags,
            trailer: MediaTrailer::from_row(row).ok(),
            episodes: row.get("anime_episodes"),
            season: row.get("anime_season"),
            season_year: row.get("anime_season_year"),
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SimplifiedAnisongSong {
    //#[sqlx(rename = "song_id")]
    pub id: Option<SongID>,
    //#[sqlx(rename = "song_name")]
    pub name: String,
    pub artist_name: String,
    pub composer_name: String,
    pub arranger_name: String,
    //#[sqlx(rename = "song_category")]
    pub category: SongCategory,
    //#[sqlx(rename = "song_length")]
    pub length: Option<f64>,
    //#[sqlx(rename = "song_is_dub")]
    pub is_dub: bool,
    pub hq: Option<String>,
    pub mq: Option<String>,
    pub audio: Option<String>,
    pub artists: Vec<SimplifiedArtist>,
    pub composers: Vec<SimplifiedArtist>,
    pub arrangers: Vec<SimplifiedArtist>,
}

impl FromRow<'_, PgRow> for SimplifiedAnisongSong {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id = row.try_get("song_id")?;
        let name = row.try_get("song_name")?;
        let artist_name = row.try_get("artist_name")?;
        let composer_name = row.try_get("composer_name")?;
        let arranger_name = row.try_get("arranger_name")?;
        let category = row.try_get("song_category")?;
        let length = row.try_get("song_length")?;
        let is_dub = row.try_get("song_is_dub")?;
        let hq = row.try_get("hq")?;
        let mq = row.try_get("mq")?;
        let audio = row.try_get("audio")?;

        // For the JSONB columns, get them as serde_json::Value then deserialize.
        let artists_value: serde_json::Value = row.try_get("artists")?;
        let composers_value: serde_json::Value = row.try_get("composers")?;
        let arrangers_value: serde_json::Value = row.try_get("arrangers")?;

        let artists: Vec<SimplifiedArtist> =
            serde_json::from_value(artists_value).map_err(|e| sqlx::Error::ColumnDecode {
                index: "artists".into(),
                source: Box::new(e),
            })?;
        let composers: Vec<SimplifiedArtist> =
            serde_json::from_value(composers_value).map_err(|e| sqlx::Error::ColumnDecode {
                index: "composers".into(),
                source: Box::new(e),
            })?;
        let arrangers: Vec<SimplifiedArtist> =
            serde_json::from_value(arrangers_value).map_err(|e| sqlx::Error::ColumnDecode {
                index: "arrangers".into(),
                source: Box::new(e),
            })?;

        Ok(SimplifiedAnisongSong {
            id,
            name,
            artist_name,
            composer_name,
            arranger_name,
            category,
            length,
            is_dub,
            hq,
            mq,
            audio,
            artists,
            composers,
            arrangers,
        })
    }
}

impl SimplifiedAnisongSong {
    pub fn decompose(anisong: AnisongSong) -> (SimplifiedAnisongSong, Vec<SimplifiedArtist>) {
        let mut artist_set = HashSet::with_capacity(
            anisong.artists.len() + anisong.arrangers.len() + anisong.composers.len(),
        );
        let mut artists = Vec::with_capacity(
            anisong.artists.len() + anisong.arrangers.len() + anisong.composers.len(),
        );
        let mut song = Self {
            id: None,
            name: anisong.name,
            artist_name: anisong.artist_name,
            composer_name: anisong.composer_name,
            arranger_name: anisong.arranger_name,
            category: anisong.category,
            length: anisong.length,
            is_dub: anisong.is_dub,
            hq: anisong.hq,
            mq: anisong.mq,
            audio: anisong.audio,
            artists: vec![],
            composers: vec![],
            arrangers: vec![],
        };
        for a in anisong.artists {
            let s = SimplifiedArtist {
                id: a.id,
                names: a.names,
                line_up_id: a.line_up_id,
                group_ids: a.groups.iter().map(|a| a.id).collect(),
                member_ids: a.members.iter().map(|a| a.id).collect(),
            };
            song.artists.push(s.clone());
            if artist_set.insert(a.id) {
                artists.push(s);
            }
        }
        for a in anisong.composers {
            let s = SimplifiedArtist {
                id: a.id,
                names: a.names,
                line_up_id: a.line_up_id,
                group_ids: a.groups.iter().map(|a| a.id).collect(),
                member_ids: a.members.iter().map(|a| a.id).collect(),
            };
            song.composers.push(s.clone());
            if artist_set.insert(a.id) {
                artists.push(s);
            }
        }
        for a in anisong.arrangers {
            let s = SimplifiedArtist {
                id: a.id,
                names: a.names,
                line_up_id: a.line_up_id,
                group_ids: a.groups.iter().map(|a| a.id).collect(),
                member_ids: a.members.iter().map(|a| a.id).collect(),
            };
            song.arrangers.push(s.clone());
            if artist_set.insert(a.id) {
                artists.push(s);
            }
        }

        (song, artists)
    }
    pub fn decompose_all(
        anisongs: Vec<AnisongSong>,
    ) -> (Vec<SimplifiedAnisongSong>, Vec<SimplifiedArtist>) {
        let mut artist_set = HashSet::new();
        let mut artists = Vec::with_capacity(anisongs.len() * 2);
        let mut songs = Vec::with_capacity(anisongs.len());
        for anisong in anisongs {
            let (song, temp_artists) = Self::decompose(anisong);
            songs.push(song);
            artists.extend(temp_artists.into_iter().filter(|a| artist_set.insert(a.id)));
        }
        (songs, artists)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnisong {
    #[sqlx(flatten)]
    pub anime: DBAnime,
    #[sqlx(flatten)]
    pub song: SimplifiedAnisongSong,
    #[sqlx(flatten)]
    pub bind: DBAnisongBind,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow, sqlx::Type)]
#[sqlx(type_name = "jsonb")]
pub struct SimplifiedArtist {
    pub names: Vec<String>,
    pub id: AnisongArtistID,
    pub line_up_id: Option<i32>,
    pub group_ids: Vec<AnisongArtistID>,
    pub member_ids: Vec<AnisongArtistID>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnisongBind {
    pub song_id: Option<SongID>,
    pub anime_ann_id: Option<AnnAnimeID>,
    pub song_ann_id: SongAnnId,

    pub difficulty: Option<f64>,
    #[sqlx(flatten)]
    pub song_index: SongIndex,
    pub is_rebroadcast: bool,
}
