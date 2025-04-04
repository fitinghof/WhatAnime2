use anilist_api::{AnilistAPI, AnilistAPIR, AnilistAnimeID};
use database_api::{Database, DatabaseR, models::DBAnime};
use sqlx::Postgres;
use std::collections::{HashMap, HashSet};
use tokio::time::interval;

fn match_by<T, U, K, F, FB, M, O>(a: &[T], b: &[U], key_a: F, key_b: FB, merge: M) -> Vec<O>
where
    F: Fn(&T) -> K,
    FB: Fn(&U) -> K,
    K: std::hash::Hash + Eq,
    M: Fn(&T, &U) -> O,
{
    let map_b: HashMap<K, &U> = b.iter().map(|item| (key_b(item), item)).collect();
    let mut matches = Vec::new();

    for item_a in a {
        if let Some(item_b) = map_b.get(&key_a(item_a)) {
            matches.push(merge(item_a, *item_b));
        }
    }
    matches
}

#[tokio::main]
async fn main() {
    dotenvy::from_path("../../dev.env").expect("This needs to be set");

    let db = DatabaseR::new(1).await;
    let anilist = AnilistAPIR::new();
    let animes = sqlx::query_as::<Postgres, DBAnime>(
        "SELECT 
            ann_id AS anime_ann_id,
            eng_name AS anime_eng_name,
            jpn_name AS anime_jpn_name,
            alt_names AS anime_alt_names,
            vintage_release_season AS anime_vintage_season,
            vintage_release_year AS anime_vintage_year,
            myanimelist_id,
            anidb_id,
            anilist_id,
            kitsu_id,
            anime_type,
            index_type AS anime_index_type,
            index_number AS anime_index_number,
            index_part AS anime_index_part,
            mean_score AS anime_mean_score,
            banner_image AS anime_banner_image,
            cover_image_color AS anime_cover_image_color,
            cover_image_medium AS anime_cover_image_medium,
            cover_image_large AS anime_cover_image_large,
            cover_image_extra_large AS anime_cover_image_extra_large,
            format AS anime_format,
            genres AS anime_genres,
            source AS anime_source,
            studios_id AS anime_studios_id,
            studios_name AS anime_studios_name,
            studios_url AS anime_studios_url,
            tags_id AS anime_tags_id,
            tags_name AS anime_tags_name,
            trailer_id AS anime_trailer_id,
            trailer_site AS anime_trailer_site,
            trailer_thumbnail AS anime_trailer_thumbnail,
            episodes AS anime_episodes,
            season AS anime_season,
            season_year AS anime_season_year
         FROM animes WHERE anilist_id IS NOT NULL AND banner_image IS NULL AND cover_image_medium IS NULL;",
    ).fetch_all(&db.pool).await.expect("We can't do anything without this");

    let ids: HashSet<AnilistAnimeID> = animes.iter().filter_map(|a| a.linked_ids.anilist).collect();
    let ids = Vec::from_iter(ids);

    let mut ticker = interval(tokio::time::Duration::from_secs(1));
    let mut media = Vec::with_capacity(ids.len());
    for id_chunk in ids.chunks(50) {
        ticker.tick().await;
        media.append(
            &mut anilist
                .fetch_many(id_chunk.to_vec())
                .await
                .expect("Wouldn't be able to do anything without this"),
        )
    }

    let updated_animes = match_by(
        &animes,
        &media,
        |a| a.linked_ids.anilist,
        |b| Some(b.id),
        |a, b| DBAnime {
            ann_id: a.ann_id,
            eng_name: a.eng_name.clone(),
            jpn_name: a.jpn_name.clone(),
            alt_name: a.alt_name.clone(),
            vintage: a.vintage.clone(),
            linked_ids: a.linked_ids.clone(),
            anime_type: a.anime_type.clone(),
            anime_index: a.anime_index.clone(),
            mean_score: b.mean_score.clone(),
            banner_image: b.banner_image.clone(),
            cover_image: b.cover_image.clone(),
            format: b.format.clone(),
            genres: b.genres.clone(),
            source: b.source.clone(),
            studios: b.studios.clone(),
            tags: b.tags.clone(),
            trailer: b.trailer.clone(),
            episodes: b.episodes.clone(),
            season: b.season.clone(),
            season_year: b.season_year,
        },
    );
    let amount = updated_animes.len();
    db.add_animes(updated_animes).await;
    println!("updated {} animes", amount);
}
