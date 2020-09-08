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
use crate::game::Game;
use anyhow::Result;
use std::fs::File;
use tracing_subscriber::{fmt, EnvFilter};

fn main() -> Result<()> {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    let database = Database::load("team-data")?;
    let games: Vec<Game> = serde_json::from_reader(File::open("game-data/4/001.json")?)?;
    println!("{:?}", games[0].simulate(&database));
    Ok(())
}
