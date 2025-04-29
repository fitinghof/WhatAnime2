use std::{io::Read, iter::Enumerate};

use database_api::{DatabaseR, models::Report};
use log::{error, warn};
use sqlx::Postgres;

const REPORT_OPTIONS: &[&str] = &[
    "Dismiss",
    "Mark as resolved",
    "Remove bind",
    "Alter anime",
    "Alter song",
];

pub async fn parse_reports() -> bool {
    match dotenvy::from_path("../../dev.env") {
        Ok(_) => {}
        Err(e) => {
            eprintln!("{}", e);
            panic!();
        }
    };
    let db = DatabaseR::new(1).await;
    let reports =
        sqlx::query_as::<Postgres, Report>("SELECT * FROM reports WHERE status = pending")
            .fetch_all(&db.pool)
            .await
            .expect("Report fetch must work");
    for report in reports {
        let anisong = match report.song_ann_id {
            Some(id) => sqlx::query_as::<Postgres, database_api::models::DBAnisong>(
                "SELECT * FROM anisong_view WHERE ann_song_id = $1",
            )
            .bind(report.song_ann_id)
            .fetch_optional(&db.pool)
            .await
            .ok()
            .flatten(),
            None => None,
        };

        loop {
            for o in REPORT_OPTIONS.iter().enumerate() {
                println!("{}: {}", o.0, o.1);
            }
            let mut inp = String::new();
            let res = std::io::stdin().read_to_string(&mut inp);
            if res.is_err() {
                error!("Failed to read input");
                continue;
            }
            match inp.trim() {
                // "1" => {sqlx::query("DO UPDATE reports SET status = dismissed WHERE report_id = $1").bind(report.);},
                _ => {
                    warn!("invalid input");
                    continue;
                }
            }
        }
    }
    todo!();
}
