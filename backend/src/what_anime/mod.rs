mod models;
mod routes;
mod utility;

use anisong_api::AnisongAPI;
use axum::Router;
use axum::http::HeaderValue;
use axum::routing::get;
use axum::routing::post;
use database_api::Database;
use reqwest::Method;
use reqwest::Url;
use reqwest::header::ACCEPT;
use reqwest::header::AUTHORIZATION;
use routes::AppState;
use routes::confirm_anime;
use routes::report;
use routes::{callback, login, update};
use spotify_api::SpotifyAPI;
use spotify_api::models::ClientID;
use spotify_api::models::ClientSecret;
use std::str::FromStr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_sessions::MemoryStore;
use tower_sessions::SessionManagerLayer;
use tower_sessions::cookie;

pub struct WhatAnime<D, S, A>
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    app_state: Arc<AppState<D, S, A>>,
}

impl<D, S, A> WhatAnime<D, S, A>
where
    D: Database + Send + Sync + 'static,
    S: SpotifyAPI + Send + Sync + 'static,
    A: AnisongAPI + Send + Sync + 'static,
{
    pub fn new(database: D, spotify_api: S, anisong_api: A) -> Self {
        let client_id =
            ClientID(std::env::var("client_id").expect("Environment variable client_id not set"));
        let client_secret = ClientSecret(
            std::env::var("client_secret").expect("Environment variable client_secret not set"),
        );

        Self {
            app_state: Arc::new(AppState {
                database,
                spotify_api,
                _anisong_api: anisong_api,
                client_id,
                client_secret,
                redirect_uri: Url::from_str("http://whatanime.ddns.net:8000/callback")
                    .expect("redirect must be valid str"),
            }),
        }
    }

    pub async fn run(&self) {
        let session_store = MemoryStore::default();
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_same_site(cookie::SameSite::Lax)
            .with_always_save(true);
        //.with_expiry(Expiry::OnInactivity(Duration::seconds(10)));

        // migrate_database(&shared_state.database).await;

        let allowed_origins = [
            "http://localhost:5173".parse::<HeaderValue>().unwrap(),
            "http://whatanime.ddns.net:5173"
                .parse::<HeaderValue>()
                .unwrap(),
        ];

        let app = Router::new()
            .route("/api/update", get(update))
            .route("/api/login", get(login))
            .route("/callback", get(callback))
            .route("/api/confirm_anime", post(confirm_anime))
            .route("/api/report", post(report))
            .layer(session_layer)
            .layer(
                CorsLayer::new()
                    .allow_origin(allowed_origins)
                    .allow_credentials(true)
                    .allow_methods([Method::GET, Method::POST])
                    .allow_headers([AUTHORIZATION, ACCEPT]),
            )
            .with_state(self.app_state.clone());

        let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
        axum::serve(listener, app).await.unwrap()
    }
}
