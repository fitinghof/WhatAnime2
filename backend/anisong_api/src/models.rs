use std::str::FromStr;

use serde::{Deserialize, Deserializer, Serialize, de::Visitor};

use what_anime_shared::AnilistAnimeID;

use sqlx::{
    FromRow, Row, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgRow, PgTypeInfo, PgValueRef},
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
    #[serde(default, deserialize_with = "empty_vec_if_null")]
    #[serde(rename = "animeAltName")]
    pub alt_name: Vec<String>,
    #[serde(rename = "animeVintage")]
    pub vintage: Option<Release>,
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
    #[serde(deserialize_with = "bool_from_int")]
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
    // Song id is internal so keep that in mind
    #[serde(rename = "annSongId")]
    pub song_ann_id: SongAnnId,
    #[serde(rename = "annId")]
    pub anime_ann_id: AnnAnimeID,
    #[serde(rename = "songDifficulty")]
    pub difficulty: Option<f64>,
    pub song_type: SongIndex,
    #[serde(deserialize_with = "bool_from_int")]
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
            #[serde(rename = "isRebroadcast", deserialize_with = "bool_from_int")]
            pub is_rebroadcast: bool,
            #[serde(rename = "annSongId")]
            pub song_ann_id: SongAnnId,
        }

        let data = Helper::deserialize(deserializer)?;
        let anime_ann_id = data.anime.ann_id;

        Ok(Self {
            song: data.song,
            anime: data.anime,
            anisong_bind: AnisongBind {
                song_ann_id: data.song_ann_id,
                anime_ann_id,
                difficulty: data.difficulty,
                song_type: data.song_type,
                is_rebroadcast: data.is_rebroadcast,
            },
        })
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Artist {
    pub id: AnisongArtistID,
    pub names: Vec<String>,
    pub line_up_id: Option<i32>,
    #[serde(default, deserialize_with = "empty_vec_if_null")]
    pub groups: Vec<Artist>,
    #[serde(default, deserialize_with = "empty_vec_if_null")]
    pub members: Vec<Artist>,
}

impl FromRow<'_, PgRow> for Artist {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let id: AnisongArtistID = row.get("artist_id");
        let names = row.get("names");
        let line_up_id = row.get("line_up_id");
        let group_ids: Vec<AnisongArtistID> = row.get("group_ids");
        let groups_names_json: Vec<sqlx::types::Json<Vec<String>>> = row.get("groups_names");
        let groups_names: Vec<Vec<String>> = groups_names_json.into_iter().map(|n| n.0).collect();
        let group_line_up_ids: Vec<Option<i32>> = row.get("group_line_up_ids");

        let groups = group_ids
            .into_iter()
            .zip(groups_names.into_iter())
            .zip(group_line_up_ids.into_iter())
            .map(|((id, names), line_up_id)| Artist {
                id,
                names,
                line_up_id,
                groups: vec![],
                members: vec![],
            })
            .collect();

        let member_ids: Vec<AnisongArtistID> = row.get("member_ids");
        let members_names_json: Vec<sqlx::types::Json<Vec<String>>> = row.get("members_names");
        let members_names: Vec<Vec<String>> = members_names_json.into_iter().map(|n| n.0).collect();
        let member_line_up_ids: Vec<Option<i32>> = row.get("member_line_up_ids");

        let members = member_ids
            .into_iter()
            .zip(members_names.into_iter())
            .zip(member_line_up_ids.into_iter())
            .map(|((id, names), line_up_id)| Artist {
                id,
                names,
                line_up_id,
                groups: vec![],
                members: vec![],
            })
            .collect();

        Ok(Self {
            id,
            names,
            line_up_id,
            groups,
            members,
        })
    }
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

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct AnimeIndex {
    #[sqlx(rename = "anime_index_type")]
    pub index_type: AnimeIndexType,
    #[sqlx(rename = "anime_index_number")]
    pub number: i32,
    #[sqlx(rename = "anime_index_part")]
    pub part: i16,
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
            index_type: anime_index_type,
            number: anime_index_number,
            part: anime_index_part,
        })
    }
}

#[derive(Serialize, Debug, Clone, FromRow)]
pub struct SongIndex {
    #[sqlx(rename = "song_index_type")]
    pub index_type: SongIndexType,
    #[sqlx(rename = "song_index_number")]
    pub number: i32,
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
            index_type: song_index_type,
            number: song_index_number,
        })
    }
}

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
pub struct AnisongArtistID(#[serde(deserialize_with = "int_from_str_or_int")] pub i32);
impl FromStr for AnisongArtistID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse::<i32>()
            .map(AnisongArtistID)
            .map_err(|_| format!("Failed to parse '{}' as AnisongArtistID", s))
    }
}

fn int_from_str_or_int<'de, D>(deserializer: D) -> Result<i32, D::Error>
where
    D: Deserializer<'de>,
{
    struct IntOrStringVisitor;

    impl<'de> Visitor<'de> for IntOrStringVisitor {
        type Value = i32;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("int or number string")
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v as i32)
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v as i32)
        }
        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match i32::from_str(v) {
                Ok(v) => Ok(v),
                Err(_) => Err(serde::de::Error::custom(format!(
                    "Failed to parse {} to i32",
                    v
                ))),
            }
        }
        fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match i32::from_str(&v) {
                Ok(v) => Ok(v),
                Err(_) => Err(serde::de::Error::custom(format!(
                    "Failed to parse {} to i32",
                    v
                ))),
            }
        }
    }

    deserializer.deserialize_any(IntOrStringVisitor)
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SongAnnId(pub i32);

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

impl FromRow<'_, PgRow> for AnimeType {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        Ok(row.get("anime_type"))
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

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum SongCategory {
    Standard,
    Character,
    Chanting,
    Instrumental,
    NoCategory,
}

impl<'de> Deserialize<'de> for SongCategory {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = &String::deserialize(deserializer).unwrap_or("no_category".to_string());
        match s {
            "Standard" => Ok(Self::Standard),
            "Character" => Ok(Self::Character),
            "Chanting" => Ok(Self::Chanting),
            "Instrumental" => Ok(Self::Instrumental),
            _ => Ok(Self::NoCategory),
        }
    }
}

impl SongCategory {
    pub fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "standard" => Ok(Self::Standard),
            "character" => Ok(Self::Character),
            "chanting" => Ok(Self::Chanting),
            "instrumental" => Ok(Self::Instrumental),
            _ => Ok(Self::NoCategory),
        }
    }
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for SongCategory {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "standard" => Ok(Self::Standard),
            "character" => Ok(Self::Character),
            "chanting" => Ok(Self::Chanting),
            "instrumental" => Ok(Self::Instrumental),
            _ => Ok(Self::NoCategory),
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
            Self::NoCategory => "no_category",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for SongCategory {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("song_category")
    }
}

impl FromRow<'_, PgRow> for SongCategory {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let s: String = row.get("song_category");
        match Self::from_str(&s) {
            Ok(value) => Ok(value),
            Err(_) => Err(sqlx::Error::Decode(s.into())),
        }
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

fn empty_vec_if_null<'de, T, D>(deserializer: D) -> Result<Vec<T>, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    let option: Option<Vec<T>> = Option::<Vec<T>>::deserialize(deserializer)?;

    match option {
        Some(value) => Ok(value),
        None => Ok(Vec::new()),
    }
}

fn bool_from_int<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    struct BoolOrIntVisitor;

    impl<'de> Visitor<'de> for BoolOrIntVisitor {
        type Value = bool;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            formatter.write_str("a boolean or an integer 0 or 1")
        }

        fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            Ok(v)
        }

        fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(serde::de::Error::custom(format!(
                    "expected 0 or 1, got {}",
                    v
                ))),
            }
        }
        fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
        where
            E: serde::de::Error,
        {
            match v {
                0 => Ok(false),
                1 => Ok(true),
                _ => Err(serde::de::Error::custom(format!(
                    "expected 0 or 1, got {}",
                    v
                ))),
            }
        }
    }

    deserializer.deserialize_any(BoolOrIntVisitor)
}

#[derive(Debug, Serialize, Clone)]
pub struct Release {
    pub season: ReleaseSeason,
    pub year: i32,
}
impl std::fmt::Display for Release {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {}", self.season, self.year)
    }
}

impl<'de> Deserialize<'de> for Release {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let (season_str, year_op) = split_string(&s);
        if let (Some(year), Ok(season)) = (year_op, ReleaseSeason::from_str(&season_str)) {
            Ok(Self {
                season: season,
                year,
            })
        } else {
            Err(serde::de::Error::custom(format!(
                "Failed to parse Release from string: {}",
                s
            )))
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum ReleaseSeason {
    Summer,
    Fall,
    Winter,
    Spring,
}

impl std::fmt::Display for ReleaseSeason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Summer => write!(f, "Summer"),
            Self::Fall => write!(f, "Fall"),
            Self::Winter => write!(f, "Winter"),
            Self::Spring => write!(f, "Spring"),
        }
    }
}

impl FromStr for ReleaseSeason {
    fn from_str(s: &str) -> Result<Self, Error> {
        match s {
            "Summer" => Ok(Self::Summer),
            "Fall" => Ok(Self::Fall),
            "Winter" => Ok(Self::Winter),
            "Spring" => Ok(Self::Spring),
            _ => Err(Error::ParseError(s.to_string())),
        }
    }

    type Err = Error;
}
