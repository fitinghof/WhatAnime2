use serde::{Deserialize, Serialize};
use sqlx::{
    FromRow, Type,
    encode::IsNull,
    error::BoxDynError,
    postgres::{PgTypeInfo, PgValueRef},
};

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct AnilistAnimeID(pub i32);

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SpotifyTrackID(String);
impl std::fmt::Display for SpotifyTrackID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

#[derive(
    Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SongID(i32);

#[derive(
    Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, FromRow, Type,
)]
#[sqlx(transparent)]
pub struct SpotifyArtistID(pub String);

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct URL(String);

#[derive(
    Debug, PartialEq, Eq, PartialOrd, Ord, Hash, FromRow, Deserialize, Serialize, Type, Clone,
)]
#[sqlx(transparent)]
pub struct ImageURL(URL);

#[derive(Deserialize, Serialize)]
pub struct SpotifyUser {
    pub display_name: Option<String>,
    pub email: Option<String>,
    pub id: SpotifyUserID,
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash, Type)]
#[sqlx(transparent)]
pub struct SpotifyUserID(String);

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ReleaseSeason {
    Winter,
    Spring,
    Summer,
    Fall,
}

impl<'r> sqlx::Decode<'r, sqlx::Postgres> for ReleaseSeason {
    fn decode(value: PgValueRef<'r>) -> Result<Self, BoxDynError> {
        let s = <&str as sqlx::Decode<sqlx::Postgres>>::decode(value)?;
        match s {
            "winter" => Ok(Self::Winter),
            "spring" => Ok(Self::Spring),
            "summer" => Ok(Self::Summer),
            "fall" => Ok(Self::Fall),
            _ => Err(format!("Error Parsing: {}", s).into()),
        }
    }
}

impl<'q> sqlx::Encode<'q, sqlx::Postgres> for ReleaseSeason {
    fn encode_by_ref(
        &self,
        buf: &mut <sqlx::Postgres as sqlx::Database>::ArgumentBuffer<'q>,
    ) -> Result<IsNull, BoxDynError> {
        let s = match self {
            Self::Winter => "winter",
            Self::Spring => "spring",
            Self::Summer => "summer",
            Self::Fall => "fall",
        };
        <&str as sqlx::Encode<sqlx::Postgres>>::encode(&s, buf)
    }
}

impl sqlx::Type<sqlx::Postgres> for ReleaseSeason {
    fn type_info() -> PgTypeInfo {
        PgTypeInfo::with_name("release_season")
    }
}
