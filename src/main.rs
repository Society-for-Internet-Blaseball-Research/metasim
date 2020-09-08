mod database;
mod history;
mod read_dir;

use crate::database::Database;
use anyhow::Result;

fn main() -> Result<()> {
    Database::load("team-data")?;
    Ok(())
}
