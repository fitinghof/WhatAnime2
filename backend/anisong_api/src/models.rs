use std::str::FromStr;

use serde::{Deserialize, Serialize};

use sqlx::{
    FromRow, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgTypeInfo, PgValueRef},
};

use crate::error::Error;

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnisongAnime {
    pub ann_id: AnnAnimeID,
    #[serde(rename = "animeENName")]
    pub eng_name: String,
    #[serde(rename = "animeJPName")]
    pub jpn_name: String,
    #[serde(rename = "animeAltName")]
    pub alt_name: Option<Vec<String>>,
    #[serde(rename = "animeVintage")]
    pub vintage: Option<String>,
    #[serde(rename = "linked_ids")]
    #[sqlx(flatten)]
    pub linked_ids: AnimeListLinks,
    pub anime_type: Option<AnimeType>,
    #[serde(rename = "animeCategory")]
    #[sqlx(flatten)]
    pub anime_index: AnimeIndex,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnisongSong {
    #[serde(rename = "annSongId")]
    pub ann_id: SongAnnID,
    #[serde(rename = "songName")]
    pub name: String,
    #[serde(rename = "songArtist")]
    pub artist_name: String,
    #[serde(rename = "songComposer")]
    pub composer_name: String,
    #[serde(rename = "songArranger")]
    pub arranger_name: String,
    #[serde(rename = "songCategory")]
    pub category: SongCategory,
    #[serde(rename = "songLength")]
    pub length: Option<f64>,
    pub is_dub: bool,
    #[serde(rename = "HQ")]
    pub hq: Option<String>,
    #[serde(rename = "MQ")]
    pub mq: Option<String>,
    pub audio: Option<String>,
    pub artists: Vec<Artist>,
    pub composers: Vec<Artist>,
    pub arrangers: Vec<Artist>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
#[serde(rename_all = "camelCase")]
pub struct AnisongBind {
    #[serde(rename = "annSongId")]
    pub song_ann_id: SongAnnID,
    #[serde(rename = "annId")]
    pub anime_ann_id: AnnAnimeID,
    #[serde(rename = "songDifficulty")]
    pub difficulty: Option<f64>,
    pub song_type: SongIndex,
    pub is_rebroadcast: bool,
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct Anisong {
    #[serde(flatten)]
    pub anime: AnisongAnime,
    #[serde(flatten)]
    pub song: AnisongSong,
    #[serde(flatten)]
    pub anisong_bind: AnisongBind,
}

impl<'de> Deserialize<'de> for Anisong {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            #[serde(flatten)]
            song: AnisongSong,
            #[serde(flatten)]
            anime: AnisongAnime,
            // For bind
            #[serde(rename = "songDifficulty")]
            pub difficulty: Option<f64>,
            #[serde(rename = "songType")]
            pub song_type: SongIndex,
            #[serde(rename = "isRebroadcast")]
            pub is_rebroadcast: bool,
        }

        let data = Helper::deserialize(deserializer)?;
        let song_ann_id = data.song.ann_id;
        let anime_ann_id = data.anime.ann_id;

        Ok(Self {
            song: data.song,
            anime: data.anime,
            anisong_bind: AnisongBind {
                song_ann_id,
                anime_ann_id,
                difficulty: data.difficulty,
                song_type: data.song_type,
                is_rebroadcast: data.is_rebroadcast,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct Artist {
    pub id: AnisongArtistID,
    pub names: Vec<String>,
    pub line_up_id: Option<i32>,
    pub groups: Option<Vec<Artist>>,
    pub members: Option<Vec<Artist>>,
}

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct AnimeListLinks {
    #[sqlx(rename = "myanimelist_id")]
    pub myanimelist: Option<MyAnimeListAnimeID>,
    #[sqlx(rename = "anidb_id")]
    pub anidb: Option<AniDBAnimeID>,
    #[sqlx(rename = "anilist_id")]
    pub anilist: Option<AnilistAnimeID>,
    #[sqlx(rename = "kitsu_id")]
    pub kitsu: Option<KitsuAnimeID>,
}

#[derive(Serialize, Debug, Clone)]
pub struct AnimeIndex {
    pub anime_index_type: AnimeIndexType,
    pub anime_index_number: i32,
    pub anime_index_part: i16,
}

impl<'de> Deserialize<'de> for AnimeIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (type_string, index_number): (String, Option<f32>) = split_string(&s);
        let anime_index_type = AnimeIndexType::from_str(&type_string);

        let temp = index_number.unwrap_or(1.0);

        let anime_index_number = temp as i32;
        let anime_index_part = if temp.fract() > 0.1 { 2 } else { 1 };

        Ok(AnimeIndex {
            anime_index_type,
            anime_index_number,
            anime_index_part,
        })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct SongIndex {
    song_index_type: SongIndexType,
    song_index_number: i32,
}

impl<'de> Deserialize<'de> for SongIndex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (type_string, index_number): (String, Option<i32>) = split_string(&s);
        let song_index_type =
            SongIndexType::from_str(&type_string).expect("We should never get bad string :(");

        let song_index_number = if song_index_type == SongIndexType::Insert {
            index_number.unwrap_or(0)
        } else {
            index_number.unwrap_or(1)
        };

        Ok(SongIndex {
            song_index_type,
            song_index_number,
        })
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct AnilistAnimeID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct MyAnimeListAnimeID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct AniDBAnimeID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct KitsuAnimeID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct AnnAnimeID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SongAnnID(i32);

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct AnisongArtistID(pub i32);

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimeType {
    TV,
    Movie,
    OVA,
    ONA,
    Special,
    Unknown,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AnimeType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(match s {
            "tv" => Self::TV,
            "movie" => Self::Movie,
            "ova" => Self::OVA,
            "ona" => Self::ONA,
            "special" => Self::Special,
            _ => Self::Unknown,
        })
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AnimeType {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::TV => "tv",
            Self::Movie => "movie",
            Self::OVA => "ova",
            Self::ONA => "ona",
            Self::Special => "special",
            Self::Unknown => "unknown",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for AnimeType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("anime_type")
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnimeIndexType {
    Season,
    Movie,
    ONA,
    OVA,
    TVSpecial,
    Special,
    MusicVideo,
    Unknown,
}

impl AnimeIndexType {
    pub fn from_str(s: &str) -> Self {
        match s {
            "TV" => Self::Season,
            "Season" => Self::Season,
            "Movie" => Self::Movie,
            "ONA" => Self::ONA,
            "OVA" => Self::OVA,
            "TV Special" => Self::TVSpecial,
            "Special" => Self::Special,
            "Music Video" => Self::MusicVideo,
            _ => {
                println!("Found weird track type: {}", s);
                Self::Unknown
            }
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for AnimeIndexType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        Ok(match s {
            "season" => Self::Season,
            "movie" => Self::Movie,
            "ona" => Self::OVA,
            "ova" => Self::ONA,
            "tv_special" => Self::TVSpecial,
            "special" => Self::Special,
            "music_video" => Self::MusicVideo,
            _ => Self::Unknown,
        })
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for AnimeIndexType {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Season => "season",
            Self::Movie => "movie",
            Self::OVA => "ona",
            Self::ONA => "ova",
            Self::TVSpecial => "tv_special",
            Self::Special => "special",
            Self::MusicVideo => "music_video",
            Self::Unknown => "unknown",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for AnimeIndexType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("anime_index_type")
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SongIndexType {
    Opening,
    Insert,
    Ending,
}

impl SongIndexType {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "Opening" => Ok(Self::Opening),
            "Insert Song" => Ok(Self::Insert),
            "Ending" => Ok(Self::Ending),
            _ => Err(Error::ParseError(s.to_string())),
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SongIndexType {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "opening" => Ok(Self::Opening),
            "insert" => Ok(Self::Insert),
            "ending" => Ok(Self::Ending),
            _ => Err(format!("Error Parsing: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SongIndexType {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Opening => "opening",
            Self::Insert => "insert",
            Self::Ending => "ending",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for SongIndexType {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("song_index_type")
    }
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SongCategory {
    Standard,
    Instrumental,
    Character,
    Chanting,
}

impl SongCategory {
    // pub fn from_str(s: &str) -> Result<Self, Error> {
    //     match s {
    //         "standard" => Ok(Self::Standard),
    //         "sharacter" => Ok(Self::Character),
    //         "shanting" => Ok(Self::Chanting),
    //         _ => Err(Error::ParseError(s.to_string())),
    //     }
    // }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SongCategory {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "standard" => Ok(Self::Standard),
            "character" => Ok(Self::Character),
            "chanting" => Ok(Self::Chanting),
            "isntrumental" => Ok(Self::Instrumental),
            _ => Err(format!("Error Parsing: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for SongCategory {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Standard => "standard",
            Self::Character => "character",
            Self::Chanting => "chanting",
            Self::Instrumental => "instrumental",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for SongCategory {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("song_category")
    }
}

fn split_string<T: FromStr>(input: &str) -> (String, Option<T>) {
    let mut words: Vec<&str> = input.split_whitespace().collect();
    if let Some(last) = words.last() {
        if let Ok(num) = last.parse::<T>() {
            words.pop();
            let text = words.join(" ");
            return (text, Some(num));
        }
    }
    (input.to_owned(), None)
}

#[cfg(test)]
mod tests {
    use super::Anisong;
    const TEST_INPUT: &str = include_str!("testParse1.txt");
    const TEST_INPUT2: &str = include_str!("testParse2.txt");
    const TEST_INPUT3: &str = include_str!("testParse3.txt");

    #[test]
    fn test_parse() {
        // General Parsing
        let _: Vec<Anisong> = serde_json::from_str(TEST_INPUT).expect("Parsing Failed");
        let _: Vec<Anisong> = serde_json::from_str(TEST_INPUT2).expect("Parsing Failed");

        // Checks to make sure that Options aren't just ommited due to missnaming or something
        let anisong: Anisong = serde_json::from_str(TEST_INPUT3).expect("Parsing Failed");
        assert!(anisong.anime.alt_name.is_some());
        assert!(anisong.anime.anime_type.is_some());
        assert!(anisong.anime.vintage.is_some());
        assert!(anisong.song.audio.is_some());
        assert!(anisong.song.hq.is_some());
        assert!(anisong.song.mq.is_some());
        assert!(anisong.anisong_bind.difficulty.is_some());
        assert!(anisong.song.length.is_some());
    }
}
