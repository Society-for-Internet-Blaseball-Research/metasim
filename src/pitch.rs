use crate::database::Player;
use crate::util::{fix, random};
use rand::{thread_rng, Rng};
use tracing::{instrument, trace};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pitch {
    Ball,
    Strike,
    Foul,
    Out,
    Single,
    Double,
    Triple,
    Dinger,
}

impl Pitch {
    #[instrument(name = "Pitch::simulate", skip(defense))]
    pub fn simulate(pitcher: &Player, batter: &Player, defense: &[Player; 9]) -> Pitch {
        // Some correlations we understand so far:
        //
        // * higher thwackability correlates to more hits
        // * higher moxie correlates to more walks
        // * higher divinity correlates to more home runs
        // * higher musclitude correlates to more doubles
        // * higher patheticism correlates to more strikeouts
        // * martyrdom doesn't really have correlations, but probably correlates to willingness to
        //   sacrifice fly?
        // * higher ground friction (baserunning stat) correlates to more triples and fewer singles
        //
        // I don't have any data on pitcher stat correlations, but I've made some guesses:
        //
        // * higher shakespearianism means pitches are more likely "dramatic" and batters will
        //   swing at them more often
        // * higher unthwackability means pitches are harder to hit
        //
        // We use the general pitcher rating where we're not sure of a correlation.

        // 1. Here's the pitch. Is it in the strike zone?
        let in_strike_zone = {
            let p = (fix(1.0 - batter.moxie, 0.2, 0.8) + fix(pitcher.pitching(), 0.0, 1.0)) / 2.0;
            let r = random();
            trace!(
                in_strike_zone = r < p,
                %p,
                %r,
                %batter.moxie,
                pitcher.pitching = %pitcher.pitching(),
            );
            r < p
        };

        // 2. Is the batter going to swing at it?
        let batter_swings = {
            let p =
                if in_strike_zone { 0.8 } else { 0.2 } + fix(pitcher.shakespearianism, 0.0, 0.15);
            let r = random();
            trace!(batter_swings = r < p, %p, %r, in_strike_zone, %pitcher.shakespearianism);
            r < p
        };

        if !batter_swings {
            if in_strike_zone {
                trace!(pitch = ?Pitch::Strike);
                return Pitch::Strike;
            } else {
                trace!(pitch = ?Pitch::Ball);
                return Pitch::Ball;
            }
        }

        // 3. The batter swings. Do they hit it?
        let batter_hits = {
            let p = if in_strike_zone { 0.8 } else { 0.1 } - fix(batter.patheticism, 0.0, 0.15)
                + fix(batter.thwackability, 0.0, 0.4)
                - fix(pitcher.unthwackability, 0.0, 0.4);
            let r = random();
            trace!(
                batter_hits = r < p,
                %p,
                %r,
                in_strike_zone,
                %batter.patheticism,
                %batter.thwackability,
                %pitcher.unthwackability,
            );
            r < p
        };
        if !batter_hits {
            trace!(pitch = ?Pitch::Strike);
            return Pitch::Strike;
        }

        // 4. The batter hits the ball. Where does it go?

        // An article [1] seems to indicate that foul balls occur in 40% of swings. I might be
        // using that number wrong here though!
        //
        // [1]: https://www.beyondtheboxscore.com/2014/6/4/5776990/swing-rate-ball-strike-counts-swinging-strikes
        let foul = {
            let p = 0.4;
            let r = random();
            trace!(foul = r < p, %p, %r);
            r < p
        };
        if foul {
            trace!(pitch = ?Pitch::Foul);
            return Pitch::Foul;
        }

        let home_run = {
            let p = fix(batter.divinity, 0.0, 0.06);
            let r = random();
            trace!(home_run = r < p, %p, %r, %batter.divinity);
            r < p
        };
        if home_run {
            trace!(pitch = ?Pitch::Dinger);
            return Pitch::Dinger;
        }

        // 5. The ball is in play. Pick a defender at random and roll
        let out = {
            let defender = &defense[thread_rng().gen_range(0, 9)];
            let p = fix(defender.defense(), 0.2, 0.6) + fix(batter.thwackability, 0.0, 0.2);
            let r = random();
            trace!(out = r < p, %p, %r, defender.defense = %defender.defense(), %batter.thwackability, ?defender);
            r < p
        };
        if out {
            trace!(pitch = ?Pitch::Out);
            return Pitch::Out;
        }

        let single = {
            let p = 1.0 - fix(batter.musclitude, 0.2, 0.4);
            let r = random();
            trace!(single = r < p, %p, %r, %batter.musclitude);
            r < p
        };
        if single {
            trace!(pitch = ?Pitch::Single);
            return Pitch::Single;
        }

        let triple = {
            let p = fix(batter.ground_friction, 0.1, 0.4);
            let r = random();
            trace!(triple = r < p, %p, %r, %batter.ground_friction);
            r < p
        };
        if triple {
            trace!(pitch = ?Pitch::Triple);
            return Pitch::Triple;
        }

        trace!(pitch = ?Pitch::Double);
        Pitch::Double
    }
}
