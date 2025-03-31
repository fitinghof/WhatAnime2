use std::collections::HashMap;
use std::env;

use anilist_api::models::TagID;
use anilist_api::{AnilistAPI, AnilistAPIR};

use anisong_api::models::{Anisong, AnisongAnime, AnisongArtistID, AnisongBind, AnisongSong};

use models::{DBAnime, DBAnisong, DBAnisongBind, Report, SimplifiedAnisongSong, SimplifiedArtist};

use sqlx::QueryBuilder;
use sqlx::migrate;
use sqlx::{self, Postgres, postgres::PgPoolOptions};
use what_anime_shared::{AnilistAnimeID, SongID, SpotifyArtistID, SpotifyTrackID, URL};

pub mod models;
pub mod regex;
pub trait Database {
    fn get_anisongs_by_song_id(
        &self,
        song_id: SpotifyTrackID,
    ) -> impl std::future::Future<Output = Vec<DBAnisong>> + Send;
    fn get_anisongs_by_artist_ids(
        &self,
        artist_ids: Vec<SpotifyArtistID>,
    ) -> impl std::future::Future<Output = Vec<DBAnisong>> + Send;
    fn get_anisongs_by_ani_artist_ids(
        &self,
        artist_ids: Vec<AnisongArtistID>,
    ) -> impl std::future::Future<Output = Vec<DBAnisong>> + Send;
    fn get_artists(
        &self,
        artist_ids: Vec<AnisongArtistID>,
    ) -> impl std::future::Future<Output = Vec<SimplifiedArtist>> + Send;
    fn bind_artists(
        &self,
        binds: Vec<(AnisongArtistID, SpotifyArtistID)>,
    ) -> impl std::future::Future<Output = u64> + Send;
    fn bind_songs(
        &self,
        binds: Vec<(SongID, SpotifyTrackID)>,
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
    fn add_from_anisongs(
        &self,
        anisongs: Vec<Anisong>,
        anilist_con: &AnilistAPIR,
    ) -> impl std::future::Future<Output = ()> + Send;
    fn add_report(&self, report: Report) -> impl std::future::Future<Output = ()> + Send;
    fn full_search(
        &self,
        song_name: String,
        artist_names: Vec<String>,
        whole_word_match: bool,
        case_sensitive: bool,
    ) -> impl std::future::Future<Output = Vec<DBAnisong>> + Send;
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
    async fn get_anisongs_by_artist_ids(&self, artist_ids: Vec<SpotifyArtistID>) -> Vec<DBAnisong> {
        if artist_ids.is_empty() {
            return vec![];
        }
        sqlx::query_as::<Postgres, DBAnisong>(ANI_SONGS_FROM_SPOTIFY_ARTISTS)
            .bind(artist_ids)
            .fetch_all(&self.pool)
            .await
            .unwrap()
    }
    async fn get_anisongs_by_song_id(&self, song_id: SpotifyTrackID) -> Vec<DBAnisong> {
        sqlx::query_as::<Postgres, DBAnisong>(ANI_SONGS_FROM_SPOTIFY_SONG)
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
    async fn bind_songs(&self, binds: Vec<(SongID, SpotifyTrackID)>) -> u64 {
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
    async fn bind_artists(&self, binds: Vec<(AnisongArtistID, SpotifyArtistID)>) -> u64 {
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
            season, season_year, vintage_release_season, vintage_release_year
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
                .push_bind(anime.season_year)
                .push_bind(anime.vintage.as_ref().map(|v| v.season.clone()))
                .push_bind(anime.vintage.map(|v| v.year));
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
    async fn add_songs(&self, songs: Vec<SimplifiedAnisongSong>) -> Vec<SongID> {
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
                .push_bind(
                    song.1
                        .artists
                        .iter()
                        .map(|a| a.id)
                        .collect::<Vec<AnisongArtistID>>(),
                )
                .push_bind(
                    song.1
                        .composers
                        .iter()
                        .map(|a| a.id)
                        .collect::<Vec<AnisongArtistID>>(),
                )
                .push_bind(
                    song.1
                        .arrangers
                        .iter()
                        .map(|a| a.id)
                        .collect::<Vec<AnisongArtistID>>(),
                );
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
    async fn add_from_anisongs(&self, anisongs: Vec<Anisong>, anilist_con: &AnilistAPIR) {
        let anilist_ids: Vec<AnilistAnimeID> = anisongs
            .iter()
            .filter_map(|a| a.anime.linked_ids.anilist)
            .collect();
        let (anime, (bind, song)): (Vec<AnisongAnime>, (Vec<AnisongBind>, Vec<AnisongSong>)) =
            anisongs
                .into_iter()
                .map(|a| (a.anime, (a.anisong_bind, a.song)))
                .unzip();

        let (simplified_song, artists) = SimplifiedAnisongSong::decompose_all(song);

        let mut song_set = HashMap::new();
        let mut binds: Vec<Vec<AnisongBind>> = Vec::new();
        let mut songs = Vec::new();
        let mut index = 0;
        simplified_song
            .into_iter()
            .zip(bind.into_iter())
            .for_each(|esb| {
                let k = (
                    esb.0.name.clone(),
                    esb.0
                        .artists
                        .iter()
                        .map(|a| a.id)
                        .collect::<Vec<AnisongArtistID>>(),
                );
                match song_set.entry(k) {
                    std::collections::hash_map::Entry::Vacant(entry) => {
                        entry.insert(index);
                        index += 1;
                        binds.push(vec![esb.1]);
                        songs.push(esb.0);
                    }
                    std::collections::hash_map::Entry::Occupied(e) => {
                        binds[*e.get()].push(esb.1);
                    }
                };
            });

        let media = anilist_con.fetch_many(anilist_ids).await;
        let db_animes = DBAnime::combine(anime, media);

        self.add_animes(db_animes).await;
        let bind_data = self.add_songs(songs).await;
        assert_eq!(bind_data.len(), binds.len());

        let mut binds2 = Vec::new();
        bind_data.into_iter().zip(binds.into_iter()).for_each(|a| {
            let (id, anisong_binds) = a;
            anisong_binds.into_iter().for_each(|a| {
                binds2.push(DBAnisongBind {
                    song_id: Some(id),
                    anime_ann_id: Some(a.anime_ann_id),
                    song_ann_id: a.song_ann_id,
                    difficulty: a.difficulty,
                    song_index: a.song_type,
                    is_rebroadcast: a.is_rebroadcast,
                })
            })
        });
        self.add_anisong_bind(binds2).await;
        self.add_artists(artists).await;
    }
    async fn add_report(&self, report: Report) {
        println!("User: {:?}\nSpotify: {:?}", report.user.id, report.track_id);
        sqlx::query::<Postgres>(
            "INSERT INTO reports (track_id, ann_song_id, message, user_name, user_mail, user_id) VALUES ($1, $2, $3, $4, $5, $6)",
        )
        .bind(report.track_id)
        .bind(report.song_ann_id)
        .bind(report.message)
        .bind(report.user.display_name)
        .bind(report.user.email)
        .bind(report.user.id)
        .execute(&self.pool)
        .await
        .expect("This would be sad");
    }
    async fn full_search(
        &self,
        song_name: String,
        artist_names: Vec<String>,
        whole_word_match: bool,
        case_sensitive: bool,
    ) -> Vec<DBAnisong> {
        let song_regex = regex::create_regex(&song_name, whole_word_match);
        let artist_regex =
            regex::create_artist_regex(artist_names.iter().collect(), whole_word_match);
        let regex_type = if case_sensitive { "~" } else { "~*" };
        sqlx::query_as::<Postgres, DBAnisong>(&format!(
           " WITH related_artist_ids AS (
                SELECT ARRAY_AGG(DISTINCT ids) AS ids
                    FROM (
                        SELECT UNNEST(ARRAY[a.id] || a.group_ids || a.member_ids) AS ids
                        FROM artists a
                            WHERE EXISTS (
                                SELECT 1
                                FROM unnest(a.names) AS name  -- Unnest the `names` array into individual rows
                                WHERE name {0} $1  -- Regex match against each name
                                LIMIT 1  -- Only need to find at least one match
                            )
                    ) subq
                )
                SELECT * FROM anisong_view s, related_artist_ids
                WHERE 
                    s.artist_ids && related_artist_ids.ids OR 
                    s.composer_ids && related_artist_ids.ids OR
                    s.song_name {0} $2;", regex_type
        ))
        .bind(artist_regex)
        .bind(song_regex)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
    async fn get_anisongs_by_ani_artist_ids(
        &self,
        artist_ids: Vec<AnisongArtistID>,
    ) -> Vec<DBAnisong> {
        if artist_ids.is_empty() {
            return vec![];
        }
        sqlx::query_as::<Postgres, DBAnisong>(
            r#"
            WITH related_artist_ids AS (
                -- Get all related artist IDs including groups and members
                SELECT ARRAY_AGG(DISTINCT ids) AS ids
                FROM (
                    SELECT UNNEST(ARRAY[a.id] || a.group_ids || a.member_ids) AS ids
                    FROM artists a
                    WHERE a.id = ANY($1)
                ) subq
            )
            SELECT DISTINCT *
            FROM related_artist_ids, anisong_view s
            WHERE 
                s.artist_ids && related_artist_ids.ids OR 
                s.composer_ids && related_artist_ids.ids
            ORDER BY s.song_id;
            "#,
        )
        .bind(artist_ids)
        .fetch_all(&self.pool)
        .await
        .unwrap()
    }
}

const ANI_SONGS_FROM_SPOTIFY_SONG: &str = r#"
WITH link AS (
    SELECT song_id 
    FROM spotify_song_links 
    WHERE spotify_id = $1
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
SELECT *
FROM (
    SELECT DISTINCT 
           s.*,
           CASE 
               WHEN s.song_id = (SELECT song_id FROM link) THEN 0 
               ELSE 1 
           END AS order_priority
    FROM related_artist_ids, anisong_view s
    WHERE 
        s.artist_ids && related_artist_ids.ids OR 
        s.composer_ids && related_artist_ids.ids
) sub
ORDER BY order_priority;
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
SELECT DISTINCT *
FROM related_artist_ids, anisong_view s
WHERE 
    s.artist_ids && related_artist_ids.ids OR 
    s.composer_ids && related_artist_ids.ids
ORDER BY s.song_id;
"#;

#[cfg(test)]
mod tests {
    use crate::{Database, DatabaseR};
    use anisong_api::models::AnisongArtistID;
    use dotenvy;
    use what_anime_shared::{SpotifyArtistID, SpotifyTrackID};

    #[tokio::test]
    async fn test_parse() {
        dotenvy::from_path("../../dev.env").expect("Failed to load env file");

        let db = DatabaseR::new(1).await;
        let artists = vec![AnisongArtistID(1)];
        let artist_ids = vec![
            SpotifyArtistID("2nvl0N9GwyX69RRBMEZ4OD".to_string()),
            SpotifyArtistID("1tofuk7dTZwb6ZKsr7XRKB".to_string()),
            SpotifyArtistID("3D73KNJRMbV45N59E8IN0F".to_string()),
        ];
        let a = db.get_anisongs_by_artist_ids(artist_ids).await;
        let b = db.get_artists(artists).await;
        let song = "idol".to_string();
        let artists = vec!["LiSA".to_string(), "Sumire Uesaka".to_string()];
        let c = db
            .full_search(song.clone(), artists.clone(), false, false)
            .await;
        let d = db
            .full_search(song.clone(), artists.clone(), true, false)
            .await;
        let e = db
            .full_search(song.clone(), artists.clone(), false, true)
            .await;
        let f = db
            .full_search(song.clone(), artists.clone(), true, true)
            .await;
        let g = db
            .get_anisongs_by_song_id(SpotifyTrackID("4svcLG3SimzCbxH0RT7Omb".to_string()))
            .await;
        assert!(!a.is_empty());
        assert!(!b.is_empty());
        assert!(!c.is_empty());
        assert!(!d.is_empty());
        assert!(!e.is_empty());
        assert!(!f.is_empty());
        assert!(!g.is_empty());
        eprintln!("{:#?}", a);
        // assert!(false);
    }
}
