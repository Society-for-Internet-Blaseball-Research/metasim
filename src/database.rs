use crate::history::History;
use crate::read_dir::{read_dir, Entries};
use anyhow::{Context, Result};
use flate2::{read::GzDecoder, write::GzEncoder, Compression};
use rustc_hash::FxHasher;
use serde::{de::IgnoredAny, Deserialize, Serialize};
use serde_json::Deserializer;
use std::collections::HashMap;
use std::fmt;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use uuid::Uuid;

const DATABASE_VERSION: u64 = 1;

#[derive(Debug, Deserialize, Serialize)]
pub struct Database {
    pub teams: HashMap<Uuid, History<Team>>,
    pub players: HashMap<Uuid, History<Player>>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize)]
pub struct Team {
    #[serde(alias = "_id")]
    pub id: Uuid,
    pub nickname: String,
    pub lineup: [Uuid; 9],
    pub rotation: [Uuid; 5],
}

#[derive(Default, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Player {
    #[serde(alias = "_id")]
    pub id: Uuid,
    pub name: String,
    pub anticapitalism: f64,
    pub base_thirst: f64,
    pub buoyancy: f64,
    pub chasiness: f64,
    pub cinnamon: f64,
    pub coldness: f64,
    pub continuation: f64,
    pub divinity: f64,
    pub ground_friction: f64,
    pub indulgence: f64,
    pub laserlikeness: f64,
    pub martyrdom: f64,
    pub moxie: f64,
    pub musclitude: f64,
    pub omniscience: f64,
    pub overpowerment: f64,
    pub patheticism: f64,
    pub pressurization: f64,
    pub ruthlessness: f64,
    pub shakespearianism: f64,
    pub tenaciousness: f64,
    pub thwackability: f64,
    pub tragicness: f64,
    pub unthwackability: f64,
    pub watchfulness: f64,
}

impl fmt::Debug for Player {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Player")
            .field("id", &self.id)
            .field("name", &self.name)
            .finish()
    }
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
