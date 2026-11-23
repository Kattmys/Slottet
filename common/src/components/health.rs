use crate::utils;
use serde::{ Serialize, Deserialize };

// OBS: all time in millis
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Health {
    points: u16,
    max: u16,
    regen_cooldown: u128,
    regen_per_second: u16,
    last_hurt: u128,
    last_healed: u128,
}

impl Health {
    #[inline]
    pub fn hurt(&mut self, amount: u16) {
        self.points = self.points.saturating_sub(amount);
        self.last_hurt = utils::epoch_time_millis();
    }

    #[inline]
    pub fn heal(&mut self, amount: u16) {
        self.points += amount;
    }

    #[inline]
    pub fn update(&mut self) {
        let now = utils::epoch_time_millis();
        if self.points < self.max && now > self.last_hurt + self.regen_cooldown {
            self.last_healed = now;
            self.points = ((
                (self.points + self.regen_per_second) as f64
                * (now - self.last_healed) as f64 / 1000.
            ) as u16).clamp(0, self.max);
        }
    }

    #[inline]
    pub fn health(&self) -> u16 {
        self.points
    }

    #[inline]
    pub fn max_health(&self) -> u16 {
        self.max
    }

    #[inline]
    pub fn ratio(&self) -> f32 {
        self.points as f32 / self.max as f32
    }

    #[inline]
    pub fn is_dead(&self) -> bool {
        self.points == 0
    }
}

impl Default for Health {
    fn default() -> Self {
        let now = utils::epoch_time_millis();
        Self {
            points: 256,
            max: 256,
            regen_cooldown: 2_000,
            regen_per_second: 35,
            last_hurt: now,
            last_healed: now,
        }
    }
}
