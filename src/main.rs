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
use anyhow::Result;
use rayon::prelude::*;
use std::fs::File;
use tracing_subscriber::{fmt, EnvFilter};

fn monte_carlo(game: &Playable) -> f64 {
    let simulations = 10_000_u32;
    let away_wins: u32 = (0..simulations)
        .into_par_iter()
        .map(|_| {
            let score = game.simulate();
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
    let games: Vec<Game> = serde_json::from_reader(File::open("game-data/4/001.json")?)?;
    println!("{}", monte_carlo(&games[0].playable(&database).unwrap()));
    Ok(())
}
