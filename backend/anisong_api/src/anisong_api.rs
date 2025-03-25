use std::vec;

use super::AnisongAPI;
use super::models::{Anisong, AnisongArtistID};
use crate::error::{Error, Result};
use futures::future::join_all;
use log::{error, warn};
use reqwest::Client;
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
pub struct AnisongAPIR {
    client: Client,
}

impl AnisongAPI for AnisongAPIR {
    async fn artist_id_search(&self, ids: Vec<AnisongArtistID>) -> Result<Vec<Anisong>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let search = ArtistIDSearchRequest {
            artist_ids: ids,
            group_granularity: 0,
            max_other_artist: 99,
            ignore_duplicate: false,
            opening_filter: true,
            ending_filter: true,
            insert_filter: true,
            normal_broadcast: true,
            dub: true,
            rebroadcast: true,
            standard: true,
            instrumental: true,
            chanting: true,
            character: true,
        };

        let response = self
            .client
            .post(Self::ARTIST_ID_SEARCH_REQUEST_URL)
            .json(&search)
            .send()
            .await?;

        match response.status() {
            value if value.is_success() => Ok(response.json().await.unwrap()),
            StatusCode::SERVICE_UNAVAILABLE | StatusCode::INTERNAL_SERVER_ERROR => {
                warn!(
                    "Non-successfull response from anisong, status: {} Response:\n{}",
                    response.status(),
                    response.text().await.unwrap(),
                );
                Ok(vec![])
            }
            _ => {
                error!(
                    "Unrecognised non-successfull response from anisong, treated as empty response, status: {} Response:\n{}",
                    response.status(),
                    response.text().await.unwrap(),
                );
                Ok(vec![])
            }
        }
    }
    async fn full_search(&self, song_title: String, mut artist_names: Vec<String>) -> Vec<Anisong> {
        let mut song_filter = Some(SearchFilter {
            search: song_title,
            partial_match: false,
            max_other_artist: Some(99),
            group_granularity: Some(0),
            arrangement: Some(true),
        });

        let mut futures = Vec::new();

        loop {
            let artist_filter = if let Some(artist_name) = artist_names.pop() {
                Some(SearchFilter {
                    search: artist_name,
                    partial_match: false,
                    group_granularity: Some(0),
                    max_other_artist: Some(99),
                    arrangement: Some(true),
                })
            } else {
                None
            };

            let search = SearchRequest {
                anime_search_filter: None,
                song_name_search_filter: song_filter,
                artist_search_filter: artist_filter.as_ref().map(|a| a.clone()),
                composer_search_filter: artist_filter,
                and_logic: Some(false),
                ignore_duplicate: Some(false),
                opening_filter: Some(true),
                insert_filter: Some(true),
                ending_filter: Some(true),
                normal_broadcast: Some(true),
                dub: Some(true),
                rebroadcast: Some(true),
                standard: Some(true),
                instrumental: Some(true),
                chanting: Some(true),
                character: Some(true),
            };

            futures.push(
                self.client
                    .post(Self::SEARCH_REQUEST_URL)
                    .json(&search)
                    .send(),
            );

            song_filter = None;
            if artist_names.is_empty() {
                break;
            }
        }

        let responses = join_all(futures).await;

        let mut anisongs: Vec<Anisong> = Vec::new();

        for result in responses {
            match result {
                Ok(response) => {
                    let mut animes = match response.status() {
                        status if status.is_success() => match response.json().await {
                            Ok(animes) => animes,
                            Err(error) => {
                                error!("Parsing of animes failed! Error:\n{}", error);
                                vec![]
                            }
                        },
                        StatusCode::SERVICE_UNAVAILABLE | StatusCode::INTERNAL_SERVER_ERROR => {
                            warn!(
                                "Non-successfull response from anisong, status: {} Response:\n{}",
                                response.status(),
                                response.text().await.unwrap_or("No text".to_string()),
                            );
                            vec![]
                        }
                        _ => {
                            error!(
                                "Unrecognised non-successfull response from anisong, treated as empty response, status: {} Response:\n{}",
                                response.status(),
                                response.text().await.unwrap_or("No text".to_string()),
                            );
                            vec![]
                        }
                    };
                    anisongs.append(&mut animes);
                }
                Err(error) => {
                    error!("Anisong fetch failed! error:\n{}", error);
                }
            }
        }
        anisongs
    }
    async fn get_exact_song(
        &self,
        song_title: String,
        artist_ids: Vec<AnisongArtistID>,
    ) -> Result<Vec<Anisong>> {
        if artist_ids.is_empty() {
            return Ok(vec![]);
        }

        let search = ArtistIDSearchRequest {
            artist_ids: artist_ids,
            group_granularity: 99,
            max_other_artist: 0,
            ignore_duplicate: false,
            opening_filter: true,
            ending_filter: true,
            insert_filter: true,
            normal_broadcast: true,
            dub: true,
            rebroadcast: true,
            standard: true,
            instrumental: true,
            chanting: true,
            character: true,
        };

        let response = self
            .client
            .post(Self::ARTIST_ID_SEARCH_REQUEST_URL)
            .json(&search)
            .send()
            .await?;

        let animes: Vec<Anisong> = match response.status() {
            status if status.is_success() => response.json().await?,
            status => {
                return Err(Error::UnsuccessfulResponse {
                    status,
                    text: response.text().await.unwrap_or("No text".to_string()),
                });
            }
        };

        Ok(animes
            .into_iter()
            .filter(|a| a.song.name == song_title)
            .collect())
    }
}

impl AnisongAPIR {
    const SEARCH_REQUEST_URL: &str = "https://anisongdb.com/api/search_request";
    const ARTIST_ID_SEARCH_REQUEST_URL: &str = "https://anisongdb.com/api/artist_ids_request";
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct SearchRequest {
    anime_search_filter: Option<SearchFilter>,
    song_name_search_filter: Option<SearchFilter>,
    artist_search_filter: Option<SearchFilter>,
    composer_search_filter: Option<SearchFilter>,

    and_logic: Option<bool>,

    ignore_duplicate: Option<bool>,

    opening_filter: Option<bool>,
    ending_filter: Option<bool>,
    insert_filter: Option<bool>,

    normal_broadcast: Option<bool>,
    dub: Option<bool>,
    rebroadcast: Option<bool>,

    standard: Option<bool>,
    instrumental: Option<bool>,
    chanting: Option<bool>,
    character: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtistIDSearchRequest {
    pub artist_ids: Vec<AnisongArtistID>,
    pub group_granularity: i32,
    pub max_other_artist: i32,
    pub ignore_duplicate: bool,
    pub opening_filter: bool,
    pub ending_filter: bool,
    pub insert_filter: bool,
    pub normal_broadcast: bool,
    pub dub: bool,
    pub rebroadcast: bool,
    pub standard: bool,
    pub instrumental: bool,
    pub chanting: bool,
    pub character: bool,
}
#[derive(Serialize, Deserialize, Debug, Default, Clone)]
pub struct SearchFilter {
    search: String,
    partial_match: bool,
    group_granularity: Option<i32>,
    max_other_artist: Option<i32>,
    arrangement: Option<bool>,
}
