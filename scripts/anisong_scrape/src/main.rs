use anilist_api::{AnilistAPI, AnilistAPIR};
use anisong_api::{
    AnisongAPI, AnisongAPIR,
    models::{AnisongAnime, AnisongArtistID, AnisongBind, AnisongSong, Release},
};
use chrono::Datelike;
use database_api::{
    Database, DatabaseR,
    models::{DBAnime, DBAnisongBind, SimplifiedAnisongSong},
};
use dotenvy;
use std::{collections::HashMap, io};
use what_anime_shared::{AnilistAnimeID, ReleaseSeason};

async fn scrape_season(
    release: Release,
    anisong: &AnisongAPIR,
    anilist: &AnilistAPIR,
    db: &DatabaseR,
) -> usize {
    let anisongs = anisong.get_anime_season(release).await.unwrap();
    let song_amount = anisongs.len();

    db.add_from_anisongs(anisongs, anilist).await;
    song_amount
}

#[tokio::main]
async fn main() {
    env_logger::Builder::new()
        .filter_level(log::LevelFilter::Info)
        .filter_module("tracing", log::LevelFilter::Warn)
        .target(env_logger::Target::Stdout)
        .init();

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
    if end_year < start_year || ((end_year == start_year) && end_season < start_season) {
        eprintln!(
            "End cannot be before start",
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
    let mut ticker = tokio::time::interval(tokio::time::Duration::from_secs(6));
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
