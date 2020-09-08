use crate::history::History;
use crate::read_dir::{read_dir, Entries};
use anyhow::{Context, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rustc_hash::FxHasher;
use serde::{de::IgnoredAny, Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DATABASE_VERSION: u64 = 1;

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    teams: HashMap<Uuid, History<Team>>,
    players: HashMap<Uuid, History<Player>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Team {
    #[serde(alias = "_id")]
    id: Uuid,
    nickname: String,
    lineup: Vec<Uuid>,
    rotation: Vec<Uuid>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[serde(alias = "_id")]
    id: Uuid,
    name: String,
    anticapitalism: f64,
    base_thirst: f64,
    buoyancy: f64,
    chasiness: f64,
    cinnamon: f64,
    coldness: f64,
    continuation: f64,
    divinity: f64,
    ground_friction: f64,
    indulgence: f64,
    laserlikeness: f64,
    martyrdom: f64,
    moxie: f64,
    musclitude: f64,
    omniscience: f64,
    overpowerment: f64,
    patheticism: f64,
    pressurization: f64,
    ruthlessness: f64,
    shakespearianism: f64,
    suppression: f64,
    tenaciousness: f64,
    thwackability: f64,
    tragicness: f64,
    unthwackability: f64,
    watchfulness: f64,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "endpoint")]
#[serde(rename_all = "camelCase")]
enum InputLine {
    AllTeams {
        data: Vec<Team>,
        #[serde(rename = "clientMeta")]
        meta: Meta,
    },
    Players {
        data: Vec<Player>,
        #[serde(rename = "clientMeta")]
        meta: Meta,
    },
    GlobalEvents(IgnoredAny),
    OffseasonSetup(IgnoredAny),
}

#[derive(Debug, Deserialize)]
struct Meta {
    timestamp: u64,
}

#[derive(Debug, Hash)]
struct CacheKey<'a> {
    version: u64,
    entries: &'a Entries,
}

impl Database {
    pub fn load<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let dir = dir.as_ref();
        let entries = read_dir(dir)?;
        if let Ok(cache) = Database::load_from_cache(&entries) {
            return Ok(cache);
        }

        let mut database = Database {
            teams: HashMap::new(),
            players: HashMap::new(),
        };

        for entry in &entries {
            let path = dir.join(&entry.file_name);
            let reader = GzDecoder::new(File::open(&path)?);
            for line in Deserializer::from_reader(reader).into_iter::<InputLine>() {
                match line? {
                    InputLine::AllTeams { data, meta } => {
                        for team in data {
                            let history = database.teams.entry(team.id).or_default();
                            history.insert(meta.timestamp, team);
                        }
                    }
                    InputLine::Players { data, meta } => {
                        for player in data {
                            let history = database.players.entry(player.id).or_default();
                            history.insert(meta.timestamp, player);
                        }
                    }
                    _ => {}
                };
            }
        }

        for history in database.teams.values_mut() {
            history.dedup();
        }
        for history in database.players.values_mut() {
            history.dedup();
        }

        database.save_to_cache(&entries).ok();
        Ok(database)
    }

    fn load_from_cache(entries: &Entries) -> Result<Self> {
        let mut reader = GzDecoder::new(File::open(get_cache_path(entries)?)?);
        Ok(bincode::deserialize_from(&mut reader)?)
    }

    fn save_to_cache(&self, entries: &Entries) -> Result<()> {
        let cache_path = get_cache_path(entries)?;
        let mut writer = GzEncoder::new(Vec::new(), Compression::default());
        bincode::serialize_into(&mut writer, self)?;
        let data = writer.finish()?;
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        std::fs::write(cache_path, data)?;
        Ok(())
    }
}

fn get_cache_path(entries: &Entries) -> Result<PathBuf> {
    let mut hasher = FxHasher::default();
    let key = CacheKey {
        version: DATABASE_VERSION,
        entries,
    };
    key.hash(&mut hasher);
    Ok(dirs::cache_dir()
        .context("unable to find cache dir")?
        .join(env!("CARGO_PKG_NAME"))
        .join(format!("db-{:x}.bincode.gz", hasher.finish())))
}
