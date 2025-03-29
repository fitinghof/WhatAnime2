use anilist_api::{AnilistAPI, AnilistAPIR};
use anisong_api::{
    AnisongAPI, AnisongAPIR,
    models::{
        AnilistAnimeID, AnisongAnime, AnisongArtistID, AnisongBind, AnisongSong, Release,
        ReleaseSeason,
    },
};
use chrono::Datelike;
use database_api::{
    Database, DatabaseR,
    models::{DBAnime, DBAnisongBind, SimplifiedAnisongSong},
};
use dotenvy;
use std::{collections::HashMap, io};

async fn scrape_season(
    release: Release,
    anisong: &AnisongAPIR,
    anilist: &AnilistAPIR,
    db: &DatabaseR,
) -> usize {
    let anisongs = anisong.get_anime_season(release).await.unwrap();
    let song_amount = anisongs.len();

    let anilist_ids: Vec<AnilistAnimeID> = anisongs
        .iter()
        .filter_map(|a| a.anime.linked_ids.anilist)
        .collect();
    let (anime, (bind, song)): (Vec<AnisongAnime>, (Vec<AnisongBind>, Vec<AnisongSong>)) = anisongs
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
            let k = (esb.0.name.clone(), esb.0.artists.clone());
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

    let media = anilist.fetch_many(anilist_ids).await;
    let db_animes = DBAnime::combine(anime, media);

    db.add_animes(db_animes).await;
    let bind_data = db.add_songs(songs).await;
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
    db.add_anisong_bind(binds2).await;
    db.add_artists(artists).await;
    song_amount
}

#[tokio::main]
async fn main() {
    let year = chrono::Local::now().year();
    match dotenvy::from_path("../../dev.env") {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            panic!();
        }
    };
    let mut inp = String::new();
    println!("First valid date is Winter 1951");
    println!(
        "input start season\n1: {}\n2: {}\n3: {}\n4: {}\n",
        ReleaseSeason::Winter,
        ReleaseSeason::Spring,
        ReleaseSeason::Summer,
        ReleaseSeason::Fall,
    );
    inp.clear();
    io::stdin()
        .read_line(&mut inp)
        .expect("Failed to parse input");

    let mut start_season: u32 = match inp.trim().parse::<u32>() {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Invalid input: {}, exiting...", e);
            return;
        }
    };
    if start_season < 1 || start_season > 4 {
        eprintln!("Invalid number, select a number between 1 and 4, including 4, exiting...");
        return;
    }
    println!("Input start year\n");
    inp.clear();
    io::stdin()
        .read_line(&mut inp)
        .expect("Failed to parse input");
    let mut start_year: u32 = match inp.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid input, exiting...");
            return;
        }
    };
    if start_year < 1951 || start_year > (year + 1) as u32 {
        eprintln!(
            "Invalid number, select a number between 1951 and {}, including {}",
            year + 1,
            year + 1
        );
        return;
    }
    // end date
    println!(
        "input end season\n1: {}\n2: {}\n3: {}\n4: {}\n",
        ReleaseSeason::Winter,
        ReleaseSeason::Spring,
        ReleaseSeason::Summer,
        ReleaseSeason::Fall,
    );
    inp.clear();
    io::stdin()
        .read_line(&mut inp)
        .expect("Failed to parse input");

    let mut end_season: u32 = match inp.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid input, exiting...");
            return;
        }
    };
    if end_season < 1 || end_season > 4 {
        eprintln!("Invalid number, select a number between 1 and 4, including 4, exiting...");
        return;
    }
    println!("Input end year\n");
    inp.clear();
    io::stdin()
        .read_line(&mut inp)
        .expect("Failed to parse input");
    let end_year: u32 = match inp.trim().parse() {
        Ok(v) => v,
        Err(_) => {
            eprintln!("Invalid input, exiting...");
            return;
        }
    };
    if end_year < 1951 || end_year > (year + 1) as u32 {
        eprintln!(
            "Invalid number, select a number between 1951 and {}, including {}",
            year + 1,
            year + 1
        );
        return;
    }

    let anisong = AnisongAPIR::new();
    let anilist = AnilistAPIR::new();
    let db = DatabaseR::new(1).await;

    let seasons = [
        ReleaseSeason::Winter,
        ReleaseSeason::Spring,
        ReleaseSeason::Summer,
        ReleaseSeason::Fall,
    ];
    start_season -= 1;
    end_season -= 1;
    // Fall 1959 has something
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(1));
    loop {
        let release = Release {
            season: seasons[start_season as usize].clone(),
            year: start_year as i32,
        };
        let found_anime = scrape_season(release.clone(), &anisong, &anilist, &db).await;
        println!(
            "Fetched {} animes from anisong release {}",
            found_anime, release
        );
        if start_season == end_season && start_year == end_year {
            println!("Data fetch done!");
            return;
        } else {
            start_season += 1;
            if start_season == 4 {
                start_season = 0;
                start_year += 1;
            }
        }
        ticker.tick().await;
    }
}
