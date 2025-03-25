use anisong_api::models::*;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, Type};

#[derive(Serialize, Deserialize, Debug, Clone, FromRow)]
pub struct DBAnime {
    #[sqlx(flatten)]
    anisongdb_anime: AnisongAnime,
}
