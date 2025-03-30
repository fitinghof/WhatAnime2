use std::sync::Arc;

use anisong_api::{AnisongAPI, models::SongAnnId};
use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
};
use database_api::{Database, models::Report};
use log::error;
use reqwest::Url;
use serde::{Deserialize, Serialize};
use spotify_api::{
    SpotifyAPI,
    models::{ClientID, ClientSecret, TokenResponse},
};
use tower_sessions::Session;
use what_anime_shared::SpotifyTrackID;

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
    todo!();
    ""
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
