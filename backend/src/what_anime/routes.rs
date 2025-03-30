use std::sync::Arc;

use anisong_api::{AnisongAPI, models::SongAnnId};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use database_api::{
    Database,
    models::{DBAnisong, Report},
};
use log::error;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use spotify_api::{
    SpotifyAPI,
    models::{ClientID, ClientSecret, CurrentlyPlaying, TokenResponse},
};
use tower_sessions::Session;
use what_anime_shared::SpotifyTrackID;

use crate::what_anime::utility::select_best;

use super::{
    models::{self, NewSongHit, NewSongMiss, SongInfo, SongUpdate},
    utility::pair_artists,
};

pub struct AppState<D, S, A>
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    pub database: D,
    pub spotify_api: S,
    pub _anisong_api: A,
    pub client_id: ClientID,
    pub client_secret: ClientSecret,
    pub redirect_uri: Url,
}

pub async fn login<D, S, A>(
    State(app_state): State<Arc<AppState<D, S, A>>>,
    session: Session,
) -> impl IntoResponse
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    let (state, url) = app_state
        .spotify_api
        .generate_login_link(app_state.client_id.clone(), app_state.redirect_uri.clone());

    insert_state(session, state).await.expect("Sad if failure");

    Redirect::to(url.as_str())
}
pub async fn update<D, S, A>(
    State(app_state): State<Arc<AppState<D, S, A>>>,
    session: Session,
) -> impl IntoResponse
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    let token = match get_token_data(session).await.unwrap() {
        Some(t) => t,
        None => return axum::Json(models::Update::LoginRequired),
    };

    // Add expiry check

    match app_state.spotify_api.get_current(token.access_token).await {
        Ok(p) => match p {
            CurrentlyPlaying::Track(t) => {
                let anisongs = app_state
                    .database
                    .get_anisongs_by_song_id(t.id.clone())
                    .await;
                if !anisongs.is_empty() {
                    let hit_id = anisongs[0]
                        .song
                        .id
                        .expect("anisong from database should always contain an id");
                    let (hits, more_by_artists): (Vec<DBAnisong>, Vec<DBAnisong>) = anisongs
                        .into_iter()
                        .partition(|a| a.song.id == Some(hit_id));

                    let artist_pairs =
                        pair_artists(t.artists.clone(), hits[0].song.artists.clone());
                    let artist_binds = artist_pairs
                        .into_iter()
                        .filter_map(|a| {
                            if a.2 > 80.0 {
                                Some((a.1.id, a.0.id))
                            } else {
                                None
                            }
                        })
                        .collect();
                    app_state.database.bind_artists(artist_binds).await;

                    return axum::Json(models::Update::NewSong(SongUpdate {
                        song_info: SongInfo::from_track(&t),
                        anisongs: models::Anisongs::Hit(NewSongHit {
                            hits,
                            more_by_artists,
                            certainty: 100,
                        }),
                    }));
                }
                let anisongs = app_state
                    .database
                    .get_anisongs_by_artist_ids(t.artists.iter().map(|a| a.id.clone()).collect())
                    .await;
                if !anisongs.is_empty() {
                    let (mut song, artist_pairs) =
                        select_best(anisongs, t.name.clone(), t.artists.clone());
                    if song.certainty >= 80 {
                        song.certainty = 100;
                        let artist_binds = artist_pairs
                            .into_iter()
                            .filter_map(|a| {
                                if a.2 > 80.0 {
                                    Some((a.1.id, a.0.id))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        app_state.database.bind_artists(artist_binds).await;
                        let best_id = song.hits[0].song.id.expect("From database must be Some");
                        app_state
                            .database
                            .bind_songs(vec![(best_id, t.id.clone())])
                            .await;
                    }
                    return axum::Json(models::Update::NewSong(SongUpdate {
                        song_info: SongInfo::from_track(&t),
                        anisongs: models::Anisongs::Hit(song),
                    }));
                }
                let anisongs = app_state
                    .database
                    .full_search(
                        t.name.clone(),
                        t.artists.iter().map(|a| a.name.clone()).collect(),
                        true,
                        true,
                    )
                    .await;
                if !anisongs.is_empty() {
                    let (mut song, artist_pairs) =
                        select_best(anisongs, t.name.clone(), t.artists.clone());

                    let final_search_ids = song.hits[0].song.artists.iter().map(|a| a.id).collect();
                    let hit_song_id = song.hits[0].song.id.expect("must be some");
                    if song.certainty >= 80 {
                        song.certainty = 100;
                        let artist_binds = artist_pairs
                            .into_iter()
                            .filter_map(|a| {
                                if a.2 > 80.0 {
                                    Some((a.1.id, a.0.id))
                                } else {
                                    None
                                }
                            })
                            .collect();
                        app_state.database.bind_artists(artist_binds).await;
                        let best_id = song.hits[0].song.id.expect("From database must be Some");
                        app_state
                            .database
                            .bind_songs(vec![(best_id, t.id.clone())])
                            .await;
                    }
                    let all_songs = app_state
                        .database
                        .get_anisongs_by_ani_artist_ids(final_search_ids)
                        .await;
                    let (hits, more) = all_songs
                        .into_iter()
                        .partition(|a| a.song.id == Some(hit_song_id));

                    song.hits = hits;
                    song.more_by_artists = more;
                    return axum::Json(models::Update::NewSong(SongUpdate {
                        song_info: SongInfo::from_track(&t),
                        anisongs: models::Anisongs::Hit(song),
                    }));
                }
                let possible = app_state
                    .database
                    .full_search(
                        t.name.clone(),
                        t.artists.iter().map(|a| a.name.clone()).collect(),
                        false,
                        false,
                    )
                    .await;

                return axum::Json(models::Update::NewSong(SongUpdate {
                    song_info: SongInfo::from_track(&t),
                    anisongs: models::Anisongs::Miss(NewSongMiss { possible }),
                }));
            }
            _ => axum::Json(models::Update::NotPlaying),
        },
        Err(_) => axum::Json(models::Update::NotPlaying),
    }
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: String,
    state: spotify_api::models::State,
}

pub async fn callback<D, S, A>(
    Query(params): Query<CallbackParams>,
    State(app_state): State<Arc<AppState<D, S, A>>>,
    session: Session,
) -> impl IntoResponse
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    let session_state = match remove_state(session.clone()).await {
        Ok(v) => v,
        Err(e) => {
            error!("Couldn't fetch State, error: {}", e);
            return Err(axum::http::StatusCode::BAD_REQUEST);
        }
    };
    if session_state.as_ref() != Some(&params.state) {
        error!(
            "Sate missmatch occured, probably\n{:?}, {:?}",
            params.state, session_state
        );
        return Err(axum::http::StatusCode::BAD_REQUEST);
    }

    let res = app_state
        .spotify_api
        .handle_callback(
            app_state.client_id.clone(),
            app_state.client_secret.clone(),
            params.code,
            app_state.redirect_uri.clone(),
        )
        .await;

    match res {
        Err(e) => match e {
            _ => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
        },
        Ok(v) => match insert_token_data(session.clone(), v).await {
            Ok(_) => {}
            Err(e) => {
                error!("Token insertion failed: {}", e);
                return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR);
            }
        },
    }

    session.save().await.unwrap();

    return Ok(Redirect::to("http://whatanime.ddns.net:5173/"));
}

#[derive(Deserialize)]
pub struct ConfirmationParams {
    pub song_id: what_anime_shared::SongID,
    pub spotify_song_id: what_anime_shared::SpotifyTrackID,
}

pub async fn confirm_anime<D, S, A>(
    Query(params): Query<ConfirmationParams>,
    State(app_state): State<Arc<AppState<D, S, A>>>,
    // session: Session,
) -> impl IntoResponse
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    app_state
        .database
        .bind_songs(vec![(params.song_id, params.spotify_song_id)])
        .await;
}

#[derive(Deserialize, Serialize)]
pub struct ReportParams {
    pub track_id: Option<SpotifyTrackID>,
    pub ann_song_id: Option<SongAnnId>,
    pub message: String,
}

pub async fn report<D, S, A>(
    State(app_state): State<Arc<AppState<D, S, A>>>,
    session: Session,
    axum::Json(params): axum::Json<ReportParams>,
) -> Result<impl IntoResponse, axum::http::StatusCode>
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    let token_data = match get_token_data(session.clone()).await {
        Ok(Some(v)) => v,
        _ => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    };
    let user = match app_state
        .spotify_api
        .get_user(token_data.access_token)
        .await
    {
        Ok(u) => u,
        Err(_) => return Err(axum::http::StatusCode::INTERNAL_SERVER_ERROR),
    };
    let report = Report {
        track_id: params.track_id,
        song_ann_id: params.ann_song_id,
        message: params.message,
        user,
    };
    app_state.database.add_report(report).await;
    Ok(())
}

async fn insert_state(
    session: Session,
    state: spotify_api::models::State,
) -> Result<(), tower_sessions::session::Error> {
    session.insert("state", state).await
}
async fn remove_state(
    session: Session,
) -> Result<Option<spotify_api::models::State>, tower_sessions::session::Error> {
    session.remove("state").await
}
async fn insert_token_data(
    session: Session,
    token_data: TokenResponse,
) -> Result<(), tower_sessions::session::Error> {
    session.insert("state", token_data).await
}
async fn get_token_data(
    session: Session,
) -> Result<Option<TokenResponse>, tower_sessions::session::Error> {
    session.get("token").await
}
