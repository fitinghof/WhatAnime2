pub mod models;
use log::error;
use log::warn;
pub use models::Media;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
pub use what_anime_shared::AnilistAnimeID;
pub use what_anime_shared::error;

pub trait AnilistAPI {
    fn fetch_one(
        &self,
        id: AnilistAnimeID,
    ) -> impl std::future::Future<Output = Option<Media>> + Send;
    fn fetch_many(
        &self,
        ids: Vec<AnilistAnimeID>,
    ) -> impl std::future::Future<Output = Result<Vec<Media>, what_anime_shared::error::Error>> + Send;
}
pub struct AnilistAPIR {
    client: Client,
}

impl AnilistAPIR {
    const QUERY_STRING: &str = include_str!("query.graphql");
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }
}

impl AnilistAPI for AnilistAPIR {
    async fn fetch_one(&self, id: AnilistAnimeID) -> Option<Media> {
        let anime = self.fetch_many(vec![id]).await;
        match anime {
            Ok(a) => a.into_iter().next(),
            Err(_) => None,
        }
    }
    async fn fetch_many(
        &self,
        ids: Vec<AnilistAnimeID>,
    ) -> Result<Vec<Media>, what_anime_shared::error::Error> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        if ids.len() > 50 {
            warn!(
                "It is unrecommended to pass more than 50 ids as this will lead to multiple instant fetches from anilist"
            );
        }

        let mut all_media: Vec<Media> = Vec::new();
        let mut page = 1;
        let per_page = 50;

        loop {
            let json_body = json!({
                "query": Self::QUERY_STRING,
                "variables": {
                    "ids": &ids,
                    "isMain": false,
                    "page": page,
                    "perPage": per_page,
                }
            });

            let response = match self
                .client
                .post("https://graphql.anilist.co")
                .json(&json_body)
                .send()
                .await
            {
                Ok(r) => r,
                Err(e) => {
                    error!("Request Failed! {:?}", e);
                    return Err(error::Error::from(e));
                }
            };

            if response.status().is_success() {
                let data: AnilistResponse = response.json().await.unwrap();
                all_media.extend(data.data.page.media);

                if data.data.page.page_info.is_none_or(|a| !a.has_next_page) {
                    break;
                }
                page += 1;
            } else {
                error!("{}", response.text().await.unwrap());
                break;
            }
        }
        all_media.sort_by(|a, b| a.id.cmp(&b.id));
        Ok(all_media)
    }
}

#[derive(Deserialize, Serialize)]
struct PageInfo {
    #[serde(rename = "hasNextPage")]
    has_next_page: bool,
}

#[derive(Deserialize, Serialize)]
struct MediaList {
    media: Vec<Media>,
    #[serde(rename = "pageInfo")]
    page_info: Option<PageInfo>,
}

#[derive(Deserialize, Serialize)]
struct PageData {
    #[serde(rename = "Page")]
    page: MediaList,
}

#[derive(Deserialize, Serialize)]
struct AnilistResponse {
    pub data: PageData,
}

#[cfg(test)]
mod tests {
    use super::*;
    const PARSE_STRING: &str = include_str!("testParse.json");
    const PARSE_STRING2: &str = include_str!("testParse2.json");

    #[tokio::test]
    async fn test_parse() {
        let animes: Vec<Media> = serde_json::from_str(PARSE_STRING).expect("This should work");
        let anime: Media = serde_json::from_str(PARSE_STRING2).expect("This should work");

        assert!(anime.banner_image.is_some());
        assert!(anime.format.is_some());
        assert!(!anime.genres.is_empty());
        assert!(anime.source.is_some());
        assert!(!anime.studios.nodes.is_empty());
        assert!(!anime.tags.is_empty());
        assert!(anime.trailer.is_some());
        assert!(anime.episodes.is_some());
        assert!(anime.season.is_some());
        assert!(anime.season_year.is_some());
    }

    #[tokio::test]
    async fn test_fetch() {
        let api = AnilistAPIR::new();
        let animes = api
            .fetch_many(vec![
                AnilistAnimeID(20997),
                AnilistAnimeID(20651),
                AnilistAnimeID(14653),
            ])
            .await
            .unwrap();
        assert!(animes.len() == 3);
    }
}
