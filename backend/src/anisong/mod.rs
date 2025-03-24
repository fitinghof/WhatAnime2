use models::{AnisongAnime, ArtistAnnId};

pub mod anisong_api;
pub mod models;

pub trait AnisongAPI {
    fn artist_id_search(ids: Vec<ArtistAnnId>) -> Result<Vec<AnisongAnime>>;
}
