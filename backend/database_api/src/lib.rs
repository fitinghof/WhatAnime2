use std::env;

use anisong_api::models::AnisongArtistID;
use anisong_api::models::SongID;
use models::DBAnime;
use models::SimplifiedArtist;
use models::SpotifyArtistId;
use models::SpotifySongId;
use sqlx::QueryBuilder;
use sqlx::{self, Postgres, postgres::PgPoolOptions};

mod models;
pub trait Database {
    fn get_anisongs_by_song_id(
        &self,
        song_id: SpotifySongId,
    ) -> impl std::future::Future<Output = Vec<DBAnime>> + Send;
    fn get_anisongs_by_artist_ids(
        &self,
        artist_ids: Vec<SpotifyArtistId>,
    ) -> impl std::future::Future<Output = Vec<DBAnime>> + Send;
    fn get_artists(
        &self,
        artist_ids: Vec<AnisongArtistID>,
    ) -> impl std::future::Future<Output = Vec<SimplifiedArtist>> + Send;
    fn bind_artist(
        &self,
        binds: Vec<(AnisongArtistID, SpotifyArtistId)>,
    ) -> impl std::future::Future<Output = u64> + Send;
    fn bind_song(
        &self,
        binds: Vec<(SongID, SpotifySongId)>,
    ) -> impl std::future::Future<Output = u64> + Send;
}

pub struct DatabaseR {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl DatabaseR {
    async fn new(num_connections: u32) -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set.");

        //println!("{}", &database_url);

        // Create the connection pool.
        let pool = PgPoolOptions::new()
            .max_connections(num_connections)
            .connect(&database_url)
            .await
            .expect("Failed to create the pool");

        Self { pool }
    }
}

impl Database for DatabaseR {
    async fn get_anisongs_by_artist_ids(&self, artist_ids: Vec<SpotifyArtistId>) -> Vec<DBAnime> {
        sqlx::query_as::<Postgres, DBAnime>(ANI_SONGS_FROM_SPOTIFY_ARTIST)
            .bind(artist_ids)
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }
    async fn get_anisongs_by_song_id(&self, song_id: SpotifySongId) -> Vec<DBAnime> {
        sqlx::query_as::<Postgres, DBAnime>(ANI_SONGS_FROM_SPOTIFY_SONG)
            .bind(song_id)
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }

    async fn get_artists(&self, artist_ids: Vec<AnisongArtistID>) -> Vec<SimplifiedArtist> {
        sqlx::query_as::<Postgres, SimplifiedArtist>(
            r#"
                SELECT * 
                FROM artists
                WHERE id = ANY($1)
                "#,
        )
        .bind(artist_ids)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
    async fn bind_song(&self, binds: Vec<(SongID, SpotifySongId)>) -> u64 {
        let mut query_builder: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("INSERT INTO spotify_song_links (song_id, spotify_id) ");
        query_builder.push_values(binds, |mut builder, value| {
            builder.push_bind(value.0).push_bind(value.1);
        });
        query_builder.push(" ON CONFLICT DO NOTHING");
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .unwrap()
            .rows_affected()
    }
    async fn bind_artist(&self, binds: Vec<(AnisongArtistID, SpotifyArtistId)>) -> u64 {
        let mut query_builder: QueryBuilder<'_, Postgres> =
            QueryBuilder::new("INSERT INTO spotify_artist_links (artist_id, spotify_id) ");
        query_builder.push_values(binds, |mut builder, value| {
            builder.push_bind(value.0).push_bind(value.1);
        });
        query_builder.push(" ON CONFLICT DO NOTHING");
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .unwrap()
            .rows_affected()
    }
}

const SONGS_FROM_ARTISTS: &str = r#"
SELECT DISTINCT s.* FROM songs s,
(
SELECT ARRAY[id] || group_ids || member_ids AS all_ids
  FROM artists 
  WHERE id = ANY($1)
) as sub
WHERE s.artists && sub.all_ids;
"#;

const ANI_SONGS_FROM_ARTISTS: &str = r#"
WITH artist_songs AS (
    SELECT DISTINCT s.*
    FROM songs s, (
        SELECT ARRAY[id] || group_ids || member_ids AS all_ids
        FROM artists 
        WHERE id = ANY(ARRAY[$1])  -- Parameterized array of artist IDs
    ) as sub
    WHERE s.artists && sub.all_ids
)
SELECT DISTINCT
    a.*, 
    s.*, 
    asl.difficulty,
    asl.song_index_type,
    asl.song_index_number,
    asl.is_rebroadcast
FROM artist_songs s
INNER JOIN anime_song_links asl ON s.id = asl.song_id
INNER JOIN animes a ON asl.anime_ann_id = a.ann_id
ORDER BY s.id;
"#;

const ANI_SONGS_FROM_SPOTIFY_SONG: &str = r#"
WITH link AS (
    SELECT song_id 
    FROM spotify_song_links 
    WHERE spotify_id = 'spotify_id3'
),
song_artists AS (
    SELECT artists, composers
    FROM songs 
    WHERE id = (SELECT song_id FROM link)
),
related_artist_ids AS (
    SELECT ARRAY_AGG(DISTINCT ids) AS ids
    FROM (
        SELECT UNNEST(ARRAY[a.id] || a.group_ids || a.member_ids) AS ids
        FROM artists a, song_artists sa
        WHERE 
            a.id = ANY(sa.artists || sa.composers)
    ) subq
)
SELECT DISTINCT
    s.id AS song_id,
    s.name AS song_name,
    s.artist_name,
    s.composer_name,
    s.arranger_name,
    s.category AS song_category,
    s.length AS song_length,
    s.is_dub AS song_is_dub,
    s.hq,
    s.mq,
    s.audio,
    s.artists,
    s.composers,
    s.arrangers,
    a.ann_id AS anime_ann_id,
    a.eng_name AS anime_eng_name,
    a.jpn_name AS anime_jpn_name,
    a.alt_names AS anime_alt_names,
    a.vintage AS anime_vintage,
    a.myanimelist_id,
    a.anidb_id,
    a.anilist_id,
    a.kitsu_id,
    a.anime_type,
    a.index_type AS anime_index_type,
    a.index_number AS anime_index_number,
    a.index_part AS anime_index_part,
    a.mean_score AS anime_mean_score,
    a.banner_image AS anime_banner_image,
    a.cover_image_color AS anime_cover_image_color,
    a.cover_image_medium AS anime_cover_image_medium,
    a.cover_image_large AS anime_cover_image_large,
    a.cover_image_extra_large AS anime_cover_image_extra_large,
    a.format AS anime_format,
    a.genres AS anime_genres,
    a.source AS anime_source,
    a.studios_id AS anime_studios_id,
    a.studios_name AS anime_studios_name,
    a.studios_url AS anime_studios_url,
    a.tags_id AS anime_tags_id,
    a.tags_name AS anime_tags_name,
    a.trailer_id AS anime_trailer_id,
    a.trailer_site AS anime_trailer_site,
    a.trailer_thumbnail AS anime_trailer_thumbnail,
    a.episodes AS anime_episodes,
    a.season AS anime_season,
    a.season_year AS anime_season_year,    
    asl.difficulty,
    asl.song_ann_id,
    asl.song_index_type,
    asl.song_index_number,
    asl.is_rebroadcast
FROM related_artist_ids, songs s
INNER JOIN anime_song_links asl ON s.id = asl.song_id
INNER JOIN animes a ON asl.anime_ann_id = a.ann_id
WHERE 
    s.artists && related_artist_ids.ids OR 
    s.composers && related_artist_ids.ids
ORDER BY s.id;
"#;

const ANI_SONGS_FROM_SPOTIFY_ARTIST: &str = r#"
WITH artist_link AS (
    -- Get the artist_id(s) linked to the given Spotify ID
    SELECT artist_id 
    FROM spotify_artist_links 
    WHERE spotify_id = 'spotigy_artist_id4'
),
related_artist_ids AS (
    -- Get all related artist IDs including groups and members
    SELECT ARRAY_AGG(DISTINCT ids) AS ids
    FROM (
        SELECT UNNEST(ARRAY[a.id] || a.group_ids || a.member_ids) AS ids
        FROM artists a
        WHERE a.id IN (SELECT artist_id FROM artist_link)
    ) subq
)
SELECT DISTINCT
    s.id AS song_id,
    s.name AS song_name,
    s.artist_name,
    s.composer_name,
    s.arranger_name,
    s.category AS song_category,
    s.length AS song_length,
    s.is_dub AS song_is_dub,
    s.hq,
    s.mq,
    s.audio,
    s.artists,
    s.composers,
    s.arrangers,
    a.ann_id AS anime_ann_id,
    a.eng_name AS anime_eng_name,
    a.jpn_name AS anime_jpn_name,
    a.alt_names AS anime_alt_names,
    a.vintage AS anime_vintage,
    a.myanimelist_id,
    a.anidb_id,
    a.anilist_id,
    a.kitsu_id,
    a.anime_type,
    a.index_type AS anime_index_type,
    a.index_number AS anime_index_number,
    a.index_part AS anime_index_part,
    a.mean_score AS anime_mean_score,
    a.banner_image AS anime_banner_image,
    a.cover_image_color AS anime_cover_image_color,
    a.cover_image_medium AS anime_cover_image_medium,
    a.cover_image_large AS anime_cover_image_large,
    a.cover_image_extra_large AS anime_cover_image_extra_large,
    a.format AS anime_format,
    a.genres AS anime_genres,
    a.source AS anime_source,
    a.studios_id AS anime_studios_id,
    a.studios_name AS anime_studios_name,
    a.studios_url AS anime_studios_url,
    a.tags_id AS anime_tags_id,
    a.tags_name AS anime_tags_name,
    a.trailer_id AS anime_trailer_id,
    a.trailer_site AS anime_trailer_site,
    a.trailer_thumbnail AS anime_trailer_thumbnail,
    a.episodes AS anime_episodes,
    a.season AS anime_season,
    a.season_year AS anime_season_year,    
    asl.difficulty,
    asl.song_ann_id,
    asl.song_index_type,
    asl.song_index_number,
    asl.is_rebroadcast
FROM related_artist_ids, songs s
INNER JOIN anime_song_links asl ON s.id = asl.song_id
INNER JOIN animes a ON asl.anime_ann_id = a.ann_id
WHERE 
    s.artists && related_artist_ids.ids OR 
    s.composers && related_artist_ids.ids
ORDER BY s.id;
"#;

#[cfg(test)]
mod tests {
    use crate::DatabaseR;

    #[tokio::test]
    async fn song_view() {
        let db = DatabaseR::new(1).await;
        assert!(1 == 1);
    }
}
