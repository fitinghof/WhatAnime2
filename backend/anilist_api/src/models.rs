use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, Row, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgRow, PgTypeInfo, PgValueRef},
};
use what_anime_shared::{AnilistAnimeID, ImageURL, ReleaseSeason, URL};

#[derive(Deserialize, Serialize, FromRow, Debug, Clone)]
pub struct MediaTitle {
    pub romaji: Option<String>,
    pub english: Option<String>,
    pub native: Option<String>,
}

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct HexColor(String);

#[derive(Deserialize, Serialize, FromRow, Clone, Debug, Default)]
pub struct CoverImage {
    #[sqlx(rename = "anime_cover_image_color")]
    pub color: Option<HexColor>,
    #[sqlx(rename = "anime_cover_image_medium")]
    pub medium: Option<ImageURL>,
    #[sqlx(rename = "anime_cover_image_large")]
    pub large: Option<ImageURL>,
    #[serde(rename = "extraLarge")]
    #[sqlx(rename = "anime_cover_image_extra_large")]
    pub extra_large: Option<ImageURL>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[repr(i16)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaFormat {
    Tv,
    TvShort,
    Movie,
    Special,
    Ova,
    Ona,
    Music,
    Manga,
    Novel,
    OneShot,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MediaFormat {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "tv" => Ok(Self::Tv),
            "tv_short" => Ok(Self::TvShort),
            "movie" => Ok(Self::Movie),
            "special" => Ok(Self::Special),
            "ova" => Ok(Self::Ova),
            "ona" => Ok(Self::Ona),
            "music" => Ok(Self::Music),
            "manga" => Ok(Self::Manga),
            "novel" => Ok(Self::Novel),
            "one_shot" => Ok(Self::OneShot),
            _ => Err(format!("Error Parsing: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for MediaFormat {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Tv => "tv",
            Self::TvShort => "tv_short",
            Self::Movie => "movie",
            Self::Special => "special",
            Self::Ova => "ova",
            Self::Ona => "ona",
            Self::Music => "music",
            Self::Manga => "manga",
            Self::Novel => "novel",
            Self::OneShot => "one_shot",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for MediaFormat {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("media_format")
    }
}

//#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize, Clone)]
//pub struct Genre(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
#[repr(i16)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MediaSource {
    Original,
    Manga,
    LightNovel,
    VisualNovel,
    VideoGame,
    Other,
    Novel,
    Doujinshi,
    Anime,
    WebNovel,
    LiveAction,
    Game,
    Comic,
    MultimediaProject,
    PictureBook,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for MediaSource {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "original" => Ok(Self::Original),
            "manga" => Ok(Self::Manga),
            "light_novel" => Ok(Self::LightNovel),
            "visual_novel" => Ok(Self::VisualNovel),
            "video_game" => Ok(Self::VideoGame),
            "other" => Ok(Self::Other),
            "novel" => Ok(Self::Novel),
            "doujinshi" => Ok(Self::Doujinshi),
            "anime" => Ok(Self::Anime),
            "web_novel" => Ok(Self::WebNovel),
            "live_action" => Ok(Self::LiveAction),
            "game" => Ok(Self::Game),
            "comic" => Ok(Self::Comic),
            "multi_media_project" => Ok(Self::MultimediaProject),
            "picture_book" => Ok(Self::PictureBook),
            _ => Err(format!("Error Parsing: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for MediaSource {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Original => "original",
            Self::Manga => "manga",
            Self::LightNovel => "light_novel",
            Self::VisualNovel => "visual_novel",
            Self::VideoGame => "video_game",
            Self::Other => "other",
            Self::Novel => "novel",
            Self::Doujinshi => "doujinshi",
            Self::Anime => "anime",
            Self::WebNovel => "web_novel",
            Self::LiveAction => "live_action",
            Self::Game => "game",
            Self::Comic => "comic",
            Self::MultimediaProject => "multi_media_project",
            Self::PictureBook => "picture_book",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for MediaSource {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("media_source")
    }
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct Studio {
    pub id: i32,
    pub name: String,
    #[serde(rename = "siteUrl")]
    pub site_url: Option<URL>,
}

#[derive(Deserialize, Serialize, Debug, Clone, Default)]
#[serde(default)]
pub struct StudioConnection {
    // edges: StudioEdge
    pub nodes: Vec<Studio>, // pageInfo: PageInfo
}
impl FromRow<'_, PgRow> for StudioConnection {
    fn from_row(row: &PgRow) -> Result<Self, sqlx::Error> {
        let ids: Vec<i32> = row.try_get("anime_studios_id")?;
        let names: Vec<String> = row.try_get("anime_studios_name")?;
        let urls: Vec<Option<URL>> = row.try_get("anime_studios_url")?;
        let mut studios = Vec::with_capacity(ids.len());
        for (id, name, url) in ids
            .into_iter()
            .zip(names.into_iter())
            .zip(urls.into_iter())
            .map(|((i, n), u)| (i, n, u))
        {
            studios.push(Studio {
                id,
                name,
                site_url: url,
            });
        }
        Ok(StudioConnection { nodes: studios })
    }
}
#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct TagID(i32);
#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct MediaTag {
    pub id: TagID,
    pub name: String,
}

#[derive(Deserialize, Serialize, FromRow, Clone, Debug)]
pub struct MediaTrailer {
    #[sqlx(rename = "anime_trailer_id")]
    pub id: String,
    #[sqlx(rename = "anime_trailer_site")]
    pub site: String,
    #[sqlx(rename = "anime_trailer_thumbnail")]
    pub thumbnail: ImageURL,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Media {
    pub id: AnilistAnimeID,
    // pub title: MediaTitle,
    #[serde(rename = "meanScore")]
    pub mean_score: Option<i32>,
    #[serde(rename = "bannerImage")]
    pub banner_image: Option<ImageURL>,
    #[serde(rename = "coverImage", default)]
    pub cover_image: CoverImage,
    pub format: Option<MediaFormat>,
    #[serde(default)]
    pub genres: Vec<String>,
    pub source: Option<MediaSource>,
    #[serde(default)]
    pub studios: StudioConnection,
    #[serde(default)]
    pub tags: Vec<MediaTag>,
    pub trailer: Option<MediaTrailer>,
    pub episodes: Option<i32>,
    pub season: Option<ReleaseSeason>,
    #[serde(rename = "seasonYear")]
    pub season_year: Option<i32>,
}

impl FromRow<'_, PgRow> for Media {
    fn from_row(row: &'_ PgRow) -> Result<Self, sqlx::Error> {
        let tag_ids: Vec<TagID> = row.try_get("tag_ids")?;
        let tag_names: Vec<String> = row.try_get("tag_names")?;
        let tags = tag_ids
            .into_iter()
            .zip(tag_names.into_iter())
            .map(|(id, name)| MediaTag { id, name })
            .collect();
        Ok(Self {
            id: row.try_get("anilist_id")?,
            mean_score: row.try_get("mean_score")?,
            banner_image: row.try_get("banner_image")?,
            cover_image: CoverImage::from_row(row)?,
            format: row.try_get("media_format")?,
            genres: row.try_get("genres")?,
            source: row.try_get("media_source")?,
            studios: StudioConnection::from_row(row)?,
            tags,
            trailer: MediaTrailer::from_row(row).ok(),
            episodes: row.try_get("episodes")?,
            season: row.try_get("release_season")?,
            season_year: row.try_get("index")?,
        })
    }
}
