use std::{collections::HashSet, f32};

use anilist_api::AnilistAPI;
use anisong_api::{AnisongAPI, models::Release};
use chrono::Datelike;
use database_api::{Database, models::DBAnisong};
use fuzzywuzzy;
use kakasi;
use log::error;
use spotify_api::models::SimplifiedArtist;

use database_api::regex::{
    normalize_text, process_artist_name, process_possible_japanese, process_similarity,
};
use what_anime_shared::ReleaseSeason;

use super::models::NewSongHit;

pub fn pair_artists(
    artists: Vec<SimplifiedArtist>,
    artists2: Vec<database_api::models::SimplifiedArtist>,
) -> Vec<(
    SimplifiedArtist,
    database_api::models::SimplifiedArtist,
    f32,
)> {
    if artists.is_empty() || artists2.is_empty() {
        return vec![];
    }
    let mut pairs = Vec::new();
    artists.into_iter().for_each(|artist| {
        let eval = artists2
            .iter()
            .map(|artist2| {
                artist2
                    .names
                    .iter()
                    .map(|artist2_name| {
                        let artist_name = process_artist_name(&artist.name);
                        let artist2_name = process_artist_name(&artist2_name);
                        if kakasi::is_japanese(&artist_name) != kakasi::IsJapanese::False
                            || kakasi::is_japanese(&artist2_name) != kakasi::IsJapanese::False
                        {
                            let artist_name = process_possible_japanese(&artist_name);
                            let artist_name = normalize_text(&artist_name);

                            let artist2_name = process_possible_japanese(&artist2_name);
                            let artist2_name = normalize_text(&artist2_name);

                            let value = fuzzywuzzy::fuzz::token_set_ratio(
                                &artist_name,
                                &artist2_name,
                                true,
                                true,
                            );

                            // This is here mainly to allow possibly more advanced processing of japanese input, for example, before  I did a comparison pass with just consonants
                            // something like that could be implemented again if I can make it reliable enough.

                            (value as f32, artist2)
                        } else {
                            let value = fuzzywuzzy::fuzz::token_set_ratio(
                                &normalize_text(&artist_name),
                                &normalize_text(&artist2_name),
                                true,
                                true,
                            ) as f32;
                            (value, artist2)
                        }
                    })
                    .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
                    .unwrap()
            })
            .max_by(|a, b| a.0.partial_cmp(&b.0).unwrap())
            .unwrap();
        pairs.push((artist, eval.1.to_owned(), eval.0));
    });
    let mut artist_set = HashSet::new();
    let mut artist_set2 = HashSet::new();
    pairs.sort_by(|a, b| {
        b.2.partial_cmp(&a.2)
            .expect("There should only be values [0, 100] here which can be compared")
    });
    pairs.retain(|p| artist_set.insert(p.0.id.clone()) && artist_set2.insert(p.1.id.clone()));
    pairs
}

pub fn select_best(
    anisongs: Vec<DBAnisong>,
    song_name: String,
    artists: Vec<SimplifiedArtist>,
) -> (
    NewSongHit,
    Vec<(
        SimplifiedArtist,
        database_api::models::SimplifiedArtist,
        f32,
    )>,
) {
    if anisongs.is_empty() {
        return (
            NewSongHit {
                hits: vec![],
                more_by_artists: vec![],
                certainty: 0,
            },
            vec![],
        );
    }
    let mut best_artist_pairs = Vec::new();
    let mut certainty = 0.0;
    let best = anisongs
        .into_iter()
        .map(|a| {
            let name_score = process_similarity(&song_name, &a.song.name);
            let artist_pairs = pair_artists(artists.clone(), a.song.artists.clone());
            let num_artists = std::cmp::max(artists.len(), a.song.artists.len());

            let mut artist_score = 0.0;
            artist_pairs.iter().for_each(|a| artist_score += a.2);
            artist_score /= num_artists as f32;

            let score = (name_score + artist_score) / 2.0;
            if score > certainty {
                best_artist_pairs = artist_pairs;
                certainty = score;
            }

            (score, a)
        })
        .collect::<Vec<(f32, DBAnisong)>>();

    let (hits, more_by_artists): (Vec<(f32, DBAnisong)>, Vec<(f32, DBAnisong)>) =
        best.into_iter().partition(|a| a.0 == certainty);

    let hits = hits.into_iter().map(|h| h.1).collect();
    let more_by_artists = more_by_artists.into_iter().map(|m| m.1).collect();
    let certainty = certainty as i32;
    (
        NewSongHit {
            hits,
            more_by_artists,
            certainty,
        },
        best_artist_pairs,
    )
}

pub async fn update_current_season<D, A, B>(db: &D, anisong: &A, anilist: &B) -> u64
where
    D: Database + 'static + Send + Sync,
    A: AnisongAPI + 'static + Send + Sync,
    B: AnilistAPI + 'static + Send + Sync,
{
    let now = chrono::Local::now();
    let year = now.year();
    let month = now.month();
    let season = match ReleaseSeason::try_from((month - 1) / 4) {
        Ok(s) => s,
        Err(e) => {
            error!("How? Error: {:?}", e);
            return 0;
        }
    };
    let release = Release { season, year };
    let anisongs = match anisong.get_anime_season(release).await {
        Ok(a) => a,
        Err(e) => {
            error!("Failed anisong fetch! Error: {:?}", e);
            return 0;
        }
    };
    let ids: HashSet<what_anime_shared::AnilistAnimeID> = anisongs
        .iter()
        .filter_map(|a| a.anime.linked_ids.anilist)
        .collect();

    let mut media: Vec<anilist_api::Media> = Vec::with_capacity(ids.len());
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    let ids: Vec<_> = ids.into_iter().collect();
    for chunk in ids.chunks(50) {
        ticker.tick().await;
        let mut new = match anilist.fetch_many(chunk.to_vec()).await {
            Ok(m) => m,
            Err(e) => {
                error!("Got error from anilist_api, Error {:?}", e);
                vec![]
            }
        };
        media.append(&mut new);
    }

    let numof = anisongs.len();
    db.add_from_anisongs(anisongs, media).await;
    numof as u64
}
