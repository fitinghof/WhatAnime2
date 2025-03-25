pub mod models;
use models::*;

async fn dosom() {
    println!("{:?}", Media::fetch_one(AnilistID(1)).await);
}
