use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use what_anime_shared::{ImageURL, SpotifyArtistID, SpotifyTrackID};

pub enum CurrentlyPlaying {
    Track(TrackObject),
    Episode,
    Nothing,
}
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash)]
pub struct State(pub String);
impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash)]
pub struct SpotifyToken(String);
impl std::fmt::Display for SpotifyToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash)]
pub struct ClientID(pub String);
impl std::fmt::Display for ClientID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Serialize, Deserialize, Hash)]
pub struct ClientSecret(pub String);
impl std::fmt::Display for ClientSecret {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

#[derive(Deserialize)]
pub struct Response {
    pub item: Item,
}

#[derive(Deserialize, Serialize)]
pub struct TokenResponse {
    pub access_token: SpotifyToken,
    pub refresh_token: Option<SpotifyToken>,
    pub expires_in: u64,
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Item {
    TrackObject(TrackObject),
    EpisodeObject,
}

#[derive(Deserialize)]
pub struct TrackObject {
    pub album: Album,
    pub artists: Vec<SimplifiedArtist>,
    pub id: SpotifyTrackID,
    pub name: String,
}

pub struct Album {
    pub name: String,
    pub images: Vec<ImageURL>,
}

impl<'de> Deserialize<'de> for Album {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Image {
            url: ImageURL,
        }
        #[derive(Deserialize)]
        pub struct Helper {
            pub name: String,
            pub images: Vec<Image>,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Self {
            name: h.name,
            images: h.images.into_iter().map(|a| a.url).collect(),
        })
    }
}

#[derive(Deserialize, Clone, Debug)]
pub struct SimplifiedArtist {
    pub id: SpotifyArtistID,
    pub name: String,
}

#[derive(Debug)]
pub struct SpotifyError {
    pub status: StatusCode,
    pub message: String,
}
impl<'de> Deserialize<'de> for SpotifyError {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct Helper {
            pub status: u16,
            pub message: String,
        }
        let h = Helper::deserialize(deserializer)?;
        Ok(Self {
            status: StatusCode::from_u16(h.status)
                .map_err(|a| serde::de::Error::custom(format!("invalid Status Code: {}", a)))?,
            message: h.message,
        })
    }
}

#[derive(Debug)]
pub enum Error {
    SpotifyError(SpotifyError),
    ParseError(String),
    ReqwestError(reqwest::Error),
    RateLimited,
    Forbidden,
    UnAuthorized,
    UnrecognisedSuccess,
}

impl From<reqwest::Error> for Error {
    fn from(value: reqwest::Error) -> Self {
        Self::ReqwestError(value)
    }
}
