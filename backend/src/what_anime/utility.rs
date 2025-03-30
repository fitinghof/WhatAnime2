use std::f32;

use database_api::models::DBAnisong;
use fuzzywuzzy;
use kakasi;
use spotify_api::models::SimplifiedArtist;

use database_api::regex::{
    normalize_text, process_artist_name, process_possible_japanese, process_similarity,
};

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
            let num_pairs = artist_pairs.len();

            let mut artist_score = 0.0;
            artist_pairs.iter().for_each(|a| artist_score += a.2);
            artist_score /= num_pairs as f32;

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
