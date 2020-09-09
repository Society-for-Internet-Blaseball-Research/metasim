use crate::database::{Database, Player};
use crate::pitch::Pitch;
use crate::util::{fix, random, AwayHome};
use rand::{thread_rng, Rng};
use serde::Deserialize;
use std::convert::TryInto;
use std::fmt;
use tracing::{instrument, trace};
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Game {
    #[serde(alias = "_id")]
    pub id: Uuid,
    pub season: u16,
    pub day: u8,
    pub away_pitcher: Uuid,
    pub away_team: Uuid,
    pub home_pitcher: Uuid,
    pub home_team: Uuid,
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
                                state.walk(batter);
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
                            if state.bases.iter().any(Option::is_some) {
                                let first_defender = &defense[thread_rng().gen_range(0, 9)];
                                let double_play = {
                                    let second_defender = &defense[thread_rng().gen_range(0, 9)];
                                    let p = fix(first_defender.defense(), 0.0, 0.075)
                                        + fix(second_defender.defense(), 0.0, 0.075);
                                    let r = random();
                                    trace!(
                                        double_play = r < p,
                                        %p,
                                        %r,
                                        first_defender.defense = %first_defender.defense(),
                                        second_defender.defense = %second_defender.defense(),
                                        ?first_defender,
                                        ?second_defender,
                                    );
                                    r < p
                                };
                                if double_play {
                                    // runner on the highest base is out, batter is out, everyone
                                    // else advances 0-1 bases
                                    outs += 1;
                                    *state.bases.iter_mut().rev().next().unwrap() = None;
                                    state.advance(0, 1);
                                    break;
                                }

                                let fielders_choice = {
                                    let p = fix(first_defender.defense(), 0.0, 0.75);
                                    let r = random();
                                    trace!(
                                        fielders_choice = r < p,
                                        %p,
                                        %r,
                                        defender.defense = %first_defender.defense(),
                                        defender = ?first_defender,
                                    );
                                    r < p
                                };
                                if fielders_choice {
                                    // runner on the highest base is out, everyone else advances
                                    // 1 base, runner on first
                                    outs += 1;
                                    *state.bases.iter_mut().rev().next().unwrap() = None;
                                    state.advance(1, 1);
                                    state.bases[0] = Some(batter);
                                    break;
                                }
                            }
                            break;
                        }
                        Pitch::Single => {
                            state.advance(1, 2);
                            state.bases[0] = Some(batter);
                            break;
                        }
                        Pitch::Double => {
                            state.advance(2, 3);
                            state.bases[1] = Some(batter);
                            break;
                        }
                        Pitch::Triple => {
                            state.advance(3, 3);
                            state.bases[2] = Some(batter);
                            break;
                        }
                        Pitch::Dinger => {
                            state.advance(3, 3);
                            trace!(player_scored = ?batter);
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

    fn score(&mut self) {
        if self.is_top() {
            self.score.score.away += 1;
        } else {
            self.score.score.home += 1;
        }
    }

    #[instrument]
    fn walk(&mut self, batter: &'a Player) {
        let mut swap = Some(batter);
        for batter in &mut self.bases {
            swap = std::mem::replace(batter, swap);
            if swap.is_none() {
                break;
            }
        }
        if let Some(player) = swap {
            trace!(player_scored = ?player);
            self.score();
        }
    }

    #[instrument]
    fn advance(&mut self, min: usize, max: usize) {
        let mut new_bases = [None; 3];
        let mut score = 0_usize;
        let mut in_front = max;
        for (i, base) in self.bases.iter_mut().enumerate().rev() {
            if let Some(runner) = base.take() {
                let extra_base = if in_front > min {
                    let p = fix(runner.baserunning(), 0.0, 0.5);
                    let r = random();
                    trace!(
                        extra_base = r < p,
                        %p,
                        %r,
                        runner.baserunning = %runner.baserunning(),
                        ?runner,
                    );
                    r < p
                } else {
                    false
                };
                in_front = if extra_base { min + 1 } else { min };
                let new_base = i + in_front;
                if new_base >= 3 {
                    trace!(player_scored = ?runner);
                    score += 1;
                } else {
                    new_bases[new_base] = Some(runner);
                }
            }
        }
        for _ in 0..score {
            self.score();
        }
        self.bases = new_bases;
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

#[cfg(test)]
mod tests {
    use super::{Player, State};
    use uuid::Uuid;

    impl Player {
        const fn test(uuid: u128) -> Player {
            Player {
                id: Uuid::from_u128(uuid),
                name: String::new(),
                anticapitalism: 0.0,
                base_thirst: 0.0,
                buoyancy: 0.0,
                chasiness: 0.0,
                cinnamon: 0.0,
                coldness: 0.0,
                continuation: 0.0,
                divinity: 0.0,
                ground_friction: 0.0,
                indulgence: 0.0,
                laserlikeness: 0.0,
                martyrdom: 0.0,
                moxie: 0.0,
                musclitude: 0.0,
                omniscience: 0.0,
                overpowerment: 0.0,
                patheticism: 0.0,
                pressurization: 0.0,
                ruthlessness: 0.0,
                shakespearianism: 0.0,
                tenaciousness: 0.0,
                thwackability: 0.0,
                tragicness: 0.0,
                unthwackability: 0.0,
                watchfulness: 0.0,
            }
        }
    }

    const ANNIE: &'static Player = &Player::test(0x4f7d749072814f8fb62e37e99a7c46a0);
    const ALYSSA: &'static Player = &Player::test(0x80de2b05e0d44d3392979951b2b5c950);
    const EIZABETH: &'static Player = &Player::test(0xaa6c266275f84506aa069a0993313216);
    const WYATT: &'static Player = &Player::test(0xe16c3f28eecd4571be1a606bbac36b2b);

    #[test]
    fn test_walk() {
        let mut state = State {
            bases: [None, None, None],
            ..Default::default()
        };
        state.walk(ANNIE);
        assert_eq!(state.bases, [Some(ANNIE), None, None]);
        assert_eq!(state.score.score.away, 0);

        let mut state = State {
            bases: [Some(ALYSSA), None, None],
            ..Default::default()
        };
        state.walk(ANNIE);
        assert_eq!(state.bases, [Some(ANNIE), Some(ALYSSA), None]);
        assert_eq!(state.score.score.away, 0);

        let mut state = State {
            bases: [None, Some(ALYSSA), None],
            ..Default::default()
        };
        state.walk(ANNIE);
        assert_eq!(state.bases, [Some(ANNIE), Some(ALYSSA), None]);
        assert_eq!(state.score.score.away, 0);

        let mut state = State {
            bases: [Some(EIZABETH), Some(ALYSSA), None],
            ..Default::default()
        };
        state.walk(ANNIE);
        assert_eq!(state.bases, [Some(ANNIE), Some(EIZABETH), Some(ALYSSA)]);
        assert_eq!(state.score.score.away, 0);

        let mut state = State {
            bases: [Some(EIZABETH), Some(ALYSSA), Some(WYATT)],
            ..Default::default()
        };
        state.walk(ANNIE);
        assert_eq!(state.bases, [Some(ANNIE), Some(EIZABETH), Some(ALYSSA)]);
        assert_eq!(state.score.score.away, 1);
    }
}
