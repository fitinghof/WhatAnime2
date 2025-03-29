use std::collections::HashSet;
use std::env;

use anilist_api::models::TagID;
use anilist_api::models::URL;
pub use anisong_api::models::AnisongArtistID;
use anisong_api::models::AnisongSong;
use anisong_api::models::SongID;
use models::DBAnime;
use models::DBAnisongBind;
use models::SimplifiedAnisongSong;
use models::SimplifiedArtist;
use models::SpotifyArtistId;
use models::SpotifySongId;
use serde::de::value;
use sqlx::QueryBuilder;
use sqlx::migrate;
use sqlx::query_builder;
use sqlx::{self, Postgres, postgres::PgPoolOptions};

pub mod models;
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
    fn bind_artists(
        &self,
        binds: Vec<(AnisongArtistID, SpotifyArtistId)>,
    ) -> impl std::future::Future<Output = u64> + Send;
    fn bind_songs(
        &self,
        binds: Vec<(SongID, SpotifySongId)>,
    ) -> impl std::future::Future<Output = u64> + Send;
    fn add_artists(
        &self,
        artist: Vec<SimplifiedArtist>,
    ) -> impl std::future::Future<Output = u64> + Send;
    fn add_songs(
        &self,
        songs: Vec<SimplifiedAnisongSong>,
    ) -> impl std::future::Future<Output = Vec<SongID>> + Send;
    fn add_animes(&self, animes: Vec<DBAnime>) -> impl std::future::Future<Output = u64> + Send;
    fn add_anisong_bind(
        &self,
        bind: Vec<DBAnisongBind>,
    ) -> impl std::future::Future<Output = u64> + Send;
}

pub struct DatabaseR {
    pool: sqlx::Pool<sqlx::Postgres>,
}

impl DatabaseR {
    pub async fn new(num_connections: u32) -> Self {
        let database_url =
            env::var("DATABASE_URL").expect("DATABASE_URL environment variable must be set.");

        //println!("{}", &database_url);

        // Create the connection pool.
        let pool = PgPoolOptions::new()
            .max_connections(num_connections)
            .connect(&database_url)
            .await
            .expect("Failed to create the pool");

        migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Migrations failed");

        Self { pool }
    }
}

impl Database for DatabaseR {
    async fn get_anisongs_by_artist_ids(&self, artist_ids: Vec<SpotifyArtistId>) -> Vec<DBAnime> {
        if artist_ids.is_empty() {
            return vec![];
        }
        sqlx::query_as::<Postgres, DBAnime>(ANI_SONGS_FROM_SPOTIFY_ARTISTS)
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
        if artist_ids.is_empty() {
            return vec![];
        }
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
    async fn bind_songs(&self, binds: Vec<(SongID, SpotifySongId)>) -> u64 {
        if binds.is_empty() {
            return 0;
        }
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
    async fn bind_artists(&self, binds: Vec<(AnisongArtistID, SpotifyArtistId)>) -> u64 {
        if binds.is_empty() {
            return 0;
        }
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
    async fn add_animes(&self, animes: Vec<DBAnime>) -> u64 {
        if animes.is_empty() {
            return 0;
        }
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"INSERT INTO animes (
            ann_id, eng_name, jpn_name, alt_names, myanimelist_id, anidb_id, anilist_id, kitsu_id, anime_type, index_type, index_number,
            index_part, mean_score, banner_image, cover_image_color, cover_image_medium, cover_image_large, cover_image_extra_large, format,
            genres, source, studios_id, studios_name, studios_url, tags_id, tags_name, trailer_id, trailer_site, trailer_thumbnail, episodes,
            season, season_year 
            )
            "#,
        );
        query_builder.push_values(animes, |mut builder, anime| {
            let (tag_ids, tag_names): (Vec<TagID>, Vec<String>) =
                anime.tags.into_iter().map(|t| (t.id, t.name)).unzip();
            let (studio_ids, studio_info): (Vec<i32>, Vec<(String, Option<URL>)>) = anime
                .studios
                .nodes
                .into_iter()
                .map(|a| (a.id, (a.name, a.site_url)))
                .unzip();

            let (studio_names, studio_urls): (Vec<String>, Vec<Option<URL>>) =
                studio_info.into_iter().unzip();

            builder
                .push_bind(anime.ann_id)
                .push_bind(anime.eng_name)
                .push_bind(anime.jpn_name)
                .push_bind(anime.alt_name)
                .push_bind(anime.linked_ids.myanimelist)
                .push_bind(anime.linked_ids.anidb)
                .push_bind(anime.linked_ids.anilist)
                .push_bind(anime.linked_ids.kitsu)
                .push_bind(anime.anime_type)
                .push_bind(anime.anime_index.index_type)
                .push_bind(anime.anime_index.number)
                .push_bind(anime.anime_index.part)
                .push_bind(anime.mean_score)
                .push_bind(anime.banner_image)
                .push_bind(anime.cover_image.color)
                .push_bind(anime.cover_image.medium)
                .push_bind(anime.cover_image.large)
                .push_bind(anime.cover_image.extra_large)
                .push_bind(anime.format)
                .push_bind(anime.genres)
                .push_bind(anime.source)
                .push_bind(studio_ids)
                .push_bind(studio_names)
                .push_bind(studio_urls)
                .push_bind(tag_ids)
                .push_bind(tag_names)
                .push_bind(anime.trailer.as_ref().map(|t| t.id.clone()))
                .push_bind(anime.trailer.as_ref().map(|t| t.site.clone()))
                .push_bind(anime.trailer.as_ref().map(|t| t.thumbnail.clone()))
                .push_bind(anime.episodes)
                .push_bind(anime.season)
                .push_bind(anime.season_year);
        });

        query_builder.push(" ON CONFLICT ( ann_id ) DO NOTHING");

        query_builder
            .build()
            .execute(&self.pool)
            .await
            .unwrap()
            .rows_affected()
    }
    async fn add_artists(&self, artists: Vec<SimplifiedArtist>) -> u64 {
        if artists.is_empty() {
            return 0;
        }
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            "INSERT INTO artists (id, names, line_up_id, group_ids, member_ids) ",
        );
        query_builder.push_values(artists, |mut builder, artist| {
            builder
                .push_bind(artist.id)
                .push_bind(artist.names)
                .push_bind(artist.line_up_id)
                .push_bind(artist.group_ids)
                .push_bind(artist.member_ids);
        });
        query_builder.push(" ON CONFLICT DO NOTHING");
        query_builder
            .build()
            .execute(&self.pool)
            .await
            .unwrap()
            .rows_affected()
    }
    async fn add_songs(&self, mut songs: Vec<SimplifiedAnisongSong>) -> Vec<SongID> {
        if songs.is_empty() {
            return vec![];
        }
        // let mut song_set = HashSet::new();
        // songs.retain(|a| song_set.insert((a.name.clone(), a.artists.clone())));
        let mut query_builder: QueryBuilder<Postgres> = QueryBuilder::new(
            r#"WITH data (temp_order, name, artist_name, composer_name, arranger_name, category, length, is_dub, hq, mq, audio, artists, composers, arrangers) AS ("#,
        );
        query_builder.push_values(songs.into_iter().enumerate(), |mut builder, song| {
            builder
                .push_bind(song.0 as i64)
                .push_bind(song.1.name)
                .push_bind(song.1.artist_name)
                .push_bind(song.1.composer_name)
                .push_bind(song.1.arranger_name)
                .push_bind(song.1.category)
                .push_bind(song.1.length)
                .push_bind(song.1.is_dub)
                .push_bind(song.1.hq)
                .push_bind(song.1.mq)
                .push_bind(song.1.audio)
                .push_bind(song.1.artists)
                .push_bind(song.1.composers)
                .push_bind(song.1.arrangers);
        });
        query_builder.push(
            r#") INSERT INTO songs (name, artist_name, composer_name, arranger_name, category, length, is_dub, hq, mq, audio, artists, composers, arrangers)
                    SELECT name, artist_name, composer_name, arranger_name, category, length, is_dub, hq, mq, audio, artists, composers, arrangers FROM data
                    ON CONFLICT (name, sort_int_array(artists)) DO UPDATE
                        SET name = EXCLUDED.name
                    RETURNING id;
        "#);

        let pairs: Vec<SongID> = query_builder
            .build_query_as()
            .fetch_all(&self.pool)
            .await
            .unwrap();
        pairs
    }
    async fn add_anisong_bind(&self, binds: Vec<DBAnisongBind>) -> u64 {
        if binds.is_empty() {
            return 0;
        }
        let mut query_builder: QueryBuilder<'_, Postgres> = QueryBuilder::new(
            "INSERT INTO anime_song_links (song_id, anime_ann_id, song_ann_id, difficulty, song_index_type, song_index_number, is_rebroadcast) ",
        );

        query_builder.push_values(binds, |mut builder, bind| {
            assert!(bind.song_id.is_some());
            assert!(bind.anime_ann_id.is_some());

            builder
                .push_bind(bind.song_id)
                .push_bind(bind.anime_ann_id)
                .push_bind(bind.song_ann_id)
                .push_bind(bind.difficulty)
                .push_bind(bind.song_index.index_type)
                .push_bind(bind.song_index.number)
                .push_bind(bind.is_rebroadcast);
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
    -- a.vintage AS anime_vintage,
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

const ANI_SONGS_FROM_SPOTIFY_ARTISTS: &str = r#"
WITH artist_link AS (
    -- Get the artist_id(s) linked to the given Spotify ID
    SELECT artist_id 
    FROM spotify_artist_links 
    WHERE spotify_id = ANY($1)
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
    use crate::{
        Database, DatabaseR,
        models::{SpotifyArtistId, SpotifySongId},
    };
    use anisong_api::models::AnisongArtistID;
    use dotenvy;

    #[tokio::test]
    async fn test_parse() {
        dotenvy::from_path("../../dev.env").expect("Failed to load env file");

        let db = DatabaseR::new(1).await;
        let artists = vec![AnisongArtistID(1)];
        let artist_ids = vec![
            SpotifyArtistId("2nvl0N9GwyX69RRBMEZ4OD".to_string()),
            SpotifyArtistId("1tofuk7dTZwb6ZKsr7XRKB".to_string()),
            SpotifyArtistId("3D73KNJRMbV45N59E8IN0F".to_string()),
        ];
        let a = db.get_anisongs_by_artist_ids(artist_ids).await;
        let b = db.get_artists(artists).await;
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        eprintln!("{:?}", a);
    }
}
