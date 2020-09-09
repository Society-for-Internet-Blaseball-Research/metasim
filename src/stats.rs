use crate::database::Player;
use crate::util::fix;

fn js_round(x: f64) -> f64 {
    if x.is_sign_negative() && (x.fract() + 0.5).abs() < f64::EPSILON {
        x.round() + 1.0
    } else {
        x.round()
    }
}

impl Player {
    pub fn current_vibe(&self, day: u8) -> f64 {
        let day = f64::from(day);
        let t = 6.0 + js_round(10.0 * self.buoyancy);
        let n = std::f64::consts::PI * (2.0 / t * day + 0.5);
        0.5 * (self.pressurization + self.cinnamon) * n.sin() - 0.5 * self.pressurization
            + 0.5 * self.cinnamon
    }

    pub fn vibe_check(&mut self, day: u8) {
        let adj = self.current_vibe(day) / 5.0;

        let pow = fix(adj.abs(), 1.0, 3.0);
        self.tragicness = if adj.is_sign_negative() {
            self.tragicness.powf(pow.recip())
        } else {
            self.tragicness.powf(pow)
        };

        self.anticapitalism += adj;
        self.base_thirst += adj;
        self.buoyancy += adj;
        self.chasiness += adj;
        self.cinnamon += adj;
        self.coldness += adj;
        self.continuation += adj;
        self.divinity += adj;
        self.ground_friction += adj;
        self.indulgence += adj;
        self.laserlikeness += adj;
        self.martyrdom += adj;
        self.moxie += adj;
        self.musclitude += adj;
        self.omniscience += adj;
        self.overpowerment += adj;
        self.patheticism -= adj;
        self.pressurization += adj;
        self.ruthlessness += adj;
        self.shakespearianism += adj;
        self.tenaciousness += adj;
        self.thwackability += adj;
        self.unthwackability += adj;
        self.watchfulness += adj;
    }

    #[allow(unused)]
    pub fn batting(&self) -> f64 {
        (1.0 - self.tragicness).powf(0.01)
            * self.thwackability.powf(0.35)
            * self.moxie.powf(0.075)
            * self.divinity.powf(0.35)
            * self.musclitude.powf(0.075)
            * (1.0 - self.patheticism).powf(0.05)
            * self.martyrdom.powf(0.02)
    }

    pub fn pitching(&self) -> f64 {
        self.shakespearianism.powf(0.1)
            * self.unthwackability.powf(0.5)
            * self.coldness.powf(0.025)
            * self.overpowerment.powf(0.15)
            * self.ruthlessness.powf(0.4)
    }

    pub fn baserunning(&self) -> f64 {
        self.laserlikeness.powf(0.5)
            * self.continuation.powf(0.1)
            * self.base_thirst.powf(0.1)
            * self.indulgence.powf(0.1)
            * self.ground_friction.powf(0.1)
    }

    pub fn defense(&self) -> f64 {
        self.omniscience.powf(0.2)
            * self.tenaciousness.powf(0.2)
            * self.watchfulness.powf(0.1)
            * self.anticapitalism.powf(0.1)
            * self.chasiness.powf(0.1)
    }
}

#[cfg(test)]
mod tests {
    use super::{js_round, Player};
    use assert_approx_eq::assert_approx_eq;

    #[test]
    fn test_js_round() {
        assert_approx_eq!(js_round(5.95), 6.0);
        assert_approx_eq!(js_round(5.5), 6.0);
        assert_approx_eq!(js_round(5.05), 5.0);
        assert_approx_eq!(js_round(-5.05), -5.0);
        assert_approx_eq!(js_round(-5.5), -5.0);
        assert_approx_eq!(js_round(-5.95), -6.0);
    }

    #[test]
    fn test_stat_lines() {
        let player = Player {
            id: "083d09d4-7ed3-4100-b021-8fbe30dd43e8".parse().unwrap(),
            name: "Jessica Telephone".to_string(),
            anticapitalism: 0.7515585414248713,
            base_thirst: 1.0637081572225477,
            buoyancy: 0.8990386558929515,
            chasiness: 0.6621760105524166,
            cinnamon: 0.3542455840750931,
            coldness: 0.5342490771184405,
            continuation: 0.9551231462319436,
            divinity: 1.1964554950661912,
            ground_friction: 1.0670693806274312,
            indulgence: 0.7491018871160651,
            laserlikeness: 0.5583179230872257,
            martyrdom: 1.5543961867934053,
            moxie: 0.9624746391865744,
            musclitude: 1.3062941924123677,
            omniscience: 0.6904289779019639,
            overpowerment: 0.6692427934187828,
            patheticism: 0.23590198952572453,
            pressurization: 0.19240544381174618,
            ruthlessness: 0.5021803174782815,
            shakespearianism: 0.9757369039444792,
            tenaciousness: 0.42571806043038707,
            thwackability: 1.0932820621017052,
            tragicness: 0.1,
            unthwackability: 0.3801831639063583,
            watchfulness: 0.6369065120599864,
        };
        assert_approx_eq!(player.current_vibe(9), -0.14020491564481974, f64::EPSILON);
        assert_approx_eq!(player.batting(), 1.1112415068761758, f64::EPSILON);
        assert_approx_eq!(player.pitching(), 0.43281742624990555, f64::EPSILON);
        assert_approx_eq!(player.baserunning(), 0.7318167167251749, f64::EPSILON);
        assert_approx_eq!(player.defense(), 0.6978296398191142, f64::EPSILON);
    }
}
