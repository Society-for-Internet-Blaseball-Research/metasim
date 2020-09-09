#![warn(clippy::pedantic, rust_2018_idioms)]

mod database;
mod game;
mod history;
mod pitch;
mod read_dir;
mod stats;
mod time;
mod util;

use crate::database::Database;
use crate::game::{Game, Playable};
use crate::util::Accuracy;
use anyhow::Result;
use rayon::prelude::*;
use std::fs::File;
use tracing_subscriber::{fmt, EnvFilter};
use walkdir::WalkDir;

fn monte_carlo(game: &Playable) -> f64 {
    let simulations = 1_000_u32;
    let away_wins: u32 = (0..simulations)
        .into_par_iter()
        .map(|i| {
            let score = game.simulate(u64::from(i));
            if score.score.away > score.score.home {
                1
            } else {
                0
            }
        })
        .sum();
    f64::from(away_wins) / f64::from(simulations)
}

fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();
    let database = Database::load("team-data")?;

    let mut official_accuracy = Accuracy::default();
    let mut model_accuracy = Accuracy::default();

    for entry in WalkDir::new("game-data") {
        let entry = entry?;
        if entry.file_type().is_file() {
            let games: Vec<Game> = serde_json::from_reader(File::open(entry.path())?)?;
            for game in games {
                let actual = u8::from(game.away_score > game.home_score);
                official_accuracy.record(game.away_odds, actual);
                if let Some(playable) = game.playable(&database) {
                    model_accuracy.record(monte_carlo(&playable), actual);
                }
            }
        }
    }

    println!("official: {}", official_accuracy);
    println!("    ours: {}", model_accuracy);
    Ok(())
}
