use std::str::FromStr;
pub mod error;

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
pub struct SpotifyTrackID(pub String);
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

#[derive(Deserialize, Serialize, Debug)]
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

impl TryFrom<u32> for ReleaseSeason {
    fn try_from(value: u32) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(ReleaseSeason::Winter),
            1 => Ok(ReleaseSeason::Spring),
            2 => Ok(ReleaseSeason::Summer),
            3 => Ok(ReleaseSeason::Fall),
            _ => Err(()),
        }
    }
    type Error = ();
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
    fn from_str(s: &str) -> Result<Self, ()> {
        match s {
            "Summer" => Ok(Self::Summer),
            "Fall" => Ok(Self::Fall),
            "Winter" => Ok(Self::Winter),
            "Spring" => Ok(Self::Spring),
            _ => Err(()),
        }
    }

    type Err = ();
}
