use crate::database::{Database, Player};
use crate::pitch::Pitch;
use crate::util::AwayHome;
use serde::Deserialize;
use std::convert::TryInto;
use std::fmt;
use tracing::{instrument, trace};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(alias = "_id")]
    id: Uuid,
    season: u16,
    day: u8,
    away_pitcher: Uuid,
    away_team: Uuid,
    away_odds: f64,
    home_pitcher: Uuid,
    home_team: Uuid,
    home_odds: f64,
}

impl fmt::Debug for Game {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Game")
            .field("id", &self.id)
            .field("season", &(self.season + 1))
            .field("day", &(self.day + 1))
            .field("away_team", &self.away_team)
            .field("home_team", &self.home_team)
            .finish()
    }
}

#[derive(Debug, Default)]
pub struct State<'a> {
    score: Score,
    bases: [Option<&'a Player>; 3],
    position: AwayHome<usize>,
}

#[derive(Debug, Default)]
pub struct Score {
    inning: u8,
    bottom: bool,
    score: AwayHome<u8>,
    shame: bool,
}

impl Game {
    #[instrument(name = "Game::simulate", skip(database))]
    pub fn simulate(&self, database: &Database) -> Option<Score> {
        let timestamp = self.timestamp();
        let teams = AwayHome {
            away: self.away_team,
            home: self.home_team,
        }
        .map_opt(|id| database.teams.get(id).and_then(|h| h.get(timestamp)))?;
        let lineups = teams.map_opt(|team| {
            let lineup = team
                .lineup
                .iter()
                .map(|id| get_player(database, self, id))
                .collect::<Option<Vec<Player>>>()?;
            if lineup.len() == 9 {
                let boxed_array: Box<[Player; 9]> = lineup.into_boxed_slice().try_into().ok()?;
                Some(*boxed_array)
            } else {
                None
            }
        })?;
        let pitchers = AwayHome {
            away: get_player(database, self, &self.away_pitcher)?,
            home: get_player(database, self, &self.home_pitcher)?,
        };

        let mut state = State::default();

        while !state.is_complete() {
            let pitcher = state.fielding(&pitchers);
            let defense = state.fielding(&lineups);

            let mut outs = 0_u8;
            while outs < 3 {
                let batter = state.batter(&lineups);
                let mut balls = 0_u8;
                let mut strikes = 0_u8;

                loop {
                    trace!(
                        balls,
                        strikes,
                        outs,
                        inning = (state.score.inning + 1),
                        top = state.is_top(),
                        bottom = state.is_bottom(),
                        ?batter,
                        ?pitcher,
                        bases = ?state.bases,
                    );

                    match Pitch::simulate(pitcher, batter, defense) {
                        Pitch::Ball => {
                            balls += 1;
                            if balls == 4 {
                                unimplemented!("on base without advancing unforced runners");
                                break;
                            }
                        }
                        Pitch::Strike => {
                            strikes += 1;
                            if strikes == 3 {
                                outs += 1;
                                break;
                            }
                        }
                        Pitch::Foul => {
                            if strikes < 2 {
                                strikes += 1;
                            }
                        }
                        Pitch::Out => {
                            outs += 1;
                            unimplemented!("handle fielder's choice / double plays");
                            break;
                        }
                        Pitch::Single | Pitch::Double | Pitch::Triple => {
                            unimplemented!("on base, advance runners");
                            break;
                        }
                        Pitch::Dinger => {
                            unimplemented!("dinger");
                            break;
                        }
                    }
                }

                state.next_batter();
            }

            state.next_half_inning();
        }

        Some(state.score)
    }

    fn timestamp(&self) -> u64 {
        crate::time::game_time(self.season, self.day)
    }
}

impl<'a> State<'a> {
    fn batter<'b>(&self, lineups: &'b AwayHome<[Player; 9]>) -> &'b Player {
        let lineup = self.hitting(lineups);
        let position = *self.hitting(&self.position);
        &lineup[position]
    }

    fn next_batter(&mut self) {
        let position = if self.is_top() {
            &mut self.position.away
        } else {
            &mut self.position.home
        };
        *position = (*position + 1) % 9;
    }

    fn next_half_inning(&mut self) {
        self.bases = [None; 3];
        if self.score.bottom {
            self.score.bottom = false;
            self.score.inning += 1;
        } else {
            self.score.bottom = true;
        }
    }

    fn hitting<'b, T>(&self, x: &'b AwayHome<T>) -> &'b T {
        if self.is_top() {
            &x.away
        } else {
            &x.home
        }
    }

    fn fielding<'b, T>(&self, x: &'b AwayHome<T>) -> &'b T {
        if self.is_bottom() {
            &x.away
        } else {
            &x.home
        }
    }

    fn is_top(&self) -> bool {
        !self.score.bottom
    }

    fn is_bottom(&self) -> bool {
        self.score.bottom
    }

    fn is_complete(&self) -> bool {
        if self.score.inning >= 8 {
            if self.is_top() && self.score.score.home > self.score.score.away {
                true
            } else {
                self.score.score.away != self.score.score.home
            }
        } else {
            false
        }
    }
}

fn get_player(database: &Database, game: &Game, id: &Uuid) -> Option<Player> {
    let mut player = database.players.get(id)?.get(game.timestamp())?.clone();
    player.vibe_check(game.day);
    Some(player)
}
