use models::DBAnime;

mod models;
pub trait Database {
    async fn get_animes() -> Vec<DBAnime>;
}
