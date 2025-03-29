pub mod models;
use anisong_api::models::AnilistAnimeID;
use log::error;
pub use models::Media;
use models::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;

pub trait AnilistAPI {
    fn fetch_one(
        &self,
        id: AnilistAnimeID,
    ) -> impl std::future::Future<Output = Option<Media>> + Send;
    fn fetch_many(
        &self,
        ids: Vec<AnilistAnimeID>,
    ) -> impl std::future::Future<Output = Vec<Media>> + Send;
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
        if anime.len() == 1 {
            Some(anime.into_iter().next().expect("len is 1, so how???"))
        } else {
            None
        }
    }
    async fn fetch_many(&self, ids: Vec<AnilistAnimeID>) -> Vec<Media> {
        if ids.is_empty() {
            return vec![];
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

            let response = self
                .client
                .post("https://graphql.anilist.co")
                .json(&json_body)
                .send()
                .await
                .unwrap();

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
        all_media
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
            .await;
        assert!(animes.len() == 3);
    }
}
