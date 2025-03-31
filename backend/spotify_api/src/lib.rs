pub mod models;
use std::str::FromStr;

use base64::{Engine, engine};
use models::{
    ClientID, ClientSecret, CurrentlyPlaying, Item, Response, SpotifyError, SpotifyToken, State,
    TokenResponse, TrackObject,
};
use rand::Rng;
use reqwest::{
    StatusCode, Url,
    header::{HeaderMap, HeaderValue},
};
use serde::Serialize;
use what_anime_shared::{SpotifyTrackID, SpotifyUser};

// use tokio::time::{Duration, Interval, interval};
pub trait SpotifyAPI {
    fn get_current(
        &self,
        token: SpotifyToken,
    ) -> impl std::future::Future<Output = Result<CurrentlyPlaying, models::Error>> + Send;
    fn get_user(
        &self,
        token: SpotifyToken,
    ) -> impl std::future::Future<Output = Result<SpotifyUser, models::Error>> + Send;

    fn get_song(
        &self,
        token: SpotifyToken,
        song_id: SpotifyTrackID,
    ) -> impl std::future::Future<Output = Result<TrackObject, models::Error>> + Send;
    fn refresh_token(
        &self,
        refresh_token: SpotifyToken,
        client_id: ClientID,
        client_secret: ClientSecret,
    ) -> impl std::future::Future<Output = Result<TokenResponse, models::Error>> + Send;
    fn generate_login_link(&self, client_id: ClientID, redirect_uri: Url) -> (State, Url);
    fn handle_callback(
        &self,
        client_id: ClientID,
        client_secret: ClientSecret,
        code: String,
        redirect_uri: Url,
    ) -> impl std::future::Future<Output = Result<TokenResponse, models::Error>> + Send;
}

pub struct SpotifyAPIR<const ALLOWED_FETCH_PER_SEC: u64> {
    client: reqwest::Client,
    //ticker: Interval,
}

impl<const ALLOWED_FETCH_PER_SEC: u64> SpotifyAPIR<ALLOWED_FETCH_PER_SEC> {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            //ticker: interval(Duration::from_millis(1000 / ALLOWED_FETCH_PER_SEC)),
        }
    }

    async fn handle_error_status(response: reqwest::Response) -> models::Error {
        match response.status() {
            StatusCode::TOO_MANY_REQUESTS => models::Error::RateLimited,
            StatusCode::FORBIDDEN => models::Error::Forbidden,
            StatusCode::UNAUTHORIZED => models::Error::UnAuthorized,
            code if code.is_success() => models::Error::UnrecognisedSuccess,
            code => models::Error::SpotifyError(response.json::<SpotifyError>().await.unwrap_or(
                SpotifyError {
                    status: code,
                    message: "Couldn't parse spotify Error".to_string(),
                },
            )),
        }
    }
}

impl<const ALLOWED_FETCH_PER_SEC: u64> SpotifyAPI for SpotifyAPIR<ALLOWED_FETCH_PER_SEC> {
    async fn get_current(&self, token: SpotifyToken) -> Result<CurrentlyPlaying, models::Error> {
        //self.ticker.tick().await;
        let mut headers = HeaderMap::new();
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Bearer {}", token)).unwrap(),
        );

        let url = Url::from_str("https://api.spotify.com/v1/me/player/currently-playing")
            .expect("Invalid URL");

        let response = self
            .client
            .get(url.clone())
            .headers(headers)
            .send()
            .await
            .unwrap();

        match response.status() {
            StatusCode::NO_CONTENT => Ok(CurrentlyPlaying::Nothing),
            StatusCode::OK => {
                let t: Response = response.json().await.unwrap();
                match t.item {
                    Item::TrackObject(t) => Ok(CurrentlyPlaying::Track(t)),
                    Item::EpisodeObject => Ok(CurrentlyPlaying::Episode),
                }
            }
            _ => Err(Self::handle_error_status(response).await),
        }
    }

    async fn get_user(&self, token: SpotifyToken) -> Result<SpotifyUser, models::Error> {
        let url = reqwest::Url::from_str("https://api.spotify.com/v1/me")
            .expect("Api endpoint must be valid");

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await
            .unwrap();

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(Self::handle_error_status(response).await),
        }
    }

    async fn get_song(
        &self,
        token: SpotifyToken,
        song_id: SpotifyTrackID,
    ) -> Result<TrackObject, models::Error> {
        let url = reqwest::Url::from_str(&format!("https://api.spotify.com/v1/tracks/{}", song_id))
            .expect("Url must be valid");

        let response = self
            .client
            .get(url)
            .header("Authorization", format!("Bearer {}", token))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(response.json().await?),
            _ => Err(Self::handle_error_status(response).await),
        }
    }

    async fn refresh_token(
        &self,
        refresh_token: SpotifyToken,
        client_id: ClientID,
        client_secret: ClientSecret,
    ) -> Result<TokenResponse, models::Error> {
        #[derive(Serialize)]
        struct TokenData {
            grant_type: &'static str,
            refresh_token: SpotifyToken,
        }

        let token_data = TokenData {
            grant_type: "refresh_token",
            refresh_token,
        };

        let refresh_url = "https://accounts.spotify.com/api/token";
        let mut headers = HeaderMap::new();

        let client_creds = format!("{}:{}", client_id, client_secret);
        let client_creds_b64 = engine::general_purpose::STANDARD.encode(client_creds);

        headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );
        headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Basic {}", client_creds_b64)).unwrap(),
        );

        let token_response = self
            .client
            .post(refresh_url)
            .headers(headers)
            .form(&token_data)
            .send()
            .await
            .unwrap();

        match token_response.status() {
            StatusCode::OK => Ok(token_response.json().await?),
            _ => Err(Self::handle_error_status(token_response).await),
        }
    }
    fn generate_login_link(&self, client_id: ClientID, redirect_uri: Url) -> (State, Url) {
        let random_bytes: [u8; 16] = rand::rng().random();
        let scope = "user-read-private user-read-email user-read-playback-state user-read-currently-playing";
        let state: State = State(hex::encode(random_bytes));

        let auth_params = [
            ("client_id", client_id.to_string()),
            ("response_type", "code".to_string()),
            ("redirect_uri", redirect_uri.to_string()),
            ("state", state.to_string()),
            ("scope", scope.to_string()),
        ];

        let url = Url::parse_with_params("https://accounts.spotify.com/authorize?", auth_params)
            .expect("Url must be valid");
        (state, url)
    }
    async fn handle_callback(
        &self,
        client_id: ClientID,
        client_secret: ClientSecret,
        code: String,
        redirect_uri: Url,
    ) -> Result<TokenResponse, models::Error> {
        let client_creds = format!("{}:{}", client_id, client_secret);
        let client_creds_b64 = engine::general_purpose::STANDARD.encode(client_creds);

        let token_data = [
            ("code", code),
            ("redirect_uri", redirect_uri.to_string()),
            ("grant_type", "authorization_code".to_string()),
        ];

        let mut token_headers = HeaderMap::new();
        token_headers.insert(
            "Authorization",
            HeaderValue::from_str(&format!("Basic {client_creds_b64}")).unwrap(),
        );
        token_headers.insert(
            "Content-Type",
            HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let token_response = self
            .client
            .post("https://accounts.spotify.com/api/token")
            .headers(token_headers)
            .form(&token_data)
            .send()
            .await
            .unwrap();

        match token_response.status() {
            StatusCode::OK => Ok(token_response.json().await?),
            _ => Err(Self::handle_error_status(token_response).await),
        }
    }
}
