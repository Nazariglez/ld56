use rustc_hash::FxHashMap;
use strum_macros::EnumIter;

#[derive(Copy, Clone, Debug)]
pub struct Params {
    pub sacred_radius: f32,
    pub karma_change_rate: f32,
    pub karma_expire_rate: f32,
    pub energy_time: f32,
    pub energy_amount: u64,
    pub following_speed: f32,
    pub eternals: bool,
    // pub eternals_radius: f32,
    pub slow_spawn_time: f32,
    pub block_spawn_souls: usize,
}

pub const PARAMS_START: Params = Params {
    sacred_radius: 50.0,
    karma_change_rate: 0.1,
    karma_expire_rate: 0.015,
    energy_time: 2.0,
    energy_amount: 1,
    following_speed: 85.0,
    eternals: false,
    slow_spawn_time: 0.0,
    block_spawn_souls: 0,
};

pub const PARAMS_END: Params = Params {
    sacred_radius: 130.0,
    karma_change_rate: 0.3,
    karma_expire_rate: 0.005,
    energy_time: 1.0,
    energy_amount: 20,
    following_speed: 150.0,
    eternals: true,
    slow_spawn_time: 20.0,
    block_spawn_souls: 18,
};

#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash, EnumIter)]
#[repr(u8)]
pub enum Blessing {
    CircleOfGrace,
    KarmaAmplification,
    KarmaPersistence,
    SpiritualFocus,
    AscendantEnergy,
    Devotion,
    GracefulPause,
    SpiritualGate,
    EternalGrace,
}

impl Blessing {
    pub fn levels(&self) -> u8 {
        match self {
            Blessing::EternalGrace => 1,
            _ => 10,
        }
    }

    pub fn price(&self, level: u8) -> u64 {
        let multi = 1.3 + (level as f32 / 12.0);
        let base: f32 = match self {
            Blessing::CircleOfGrace => 10.0,
            Blessing::KarmaAmplification => 30.0,
            Blessing::KarmaPersistence => 80.0,
            Blessing::SpiritualFocus => 150.0,
            Blessing::AscendantEnergy => 300.0,
            Blessing::Devotion => 550.0,
            Blessing::GracefulPause => 1000.0,
            Blessing::SpiritualGate => 1800.0,
            Blessing::EternalGrace => 2590.0,
        };

        (base.powf(multi)) as _
    }

    pub fn require(&self) -> Option<(u8, Blessing)> {
        Some(match self {
            Blessing::CircleOfGrace => return None,
            Blessing::KarmaAmplification => (4, Blessing::CircleOfGrace),
            Blessing::KarmaPersistence => (3, Blessing::KarmaAmplification),
            Blessing::SpiritualFocus => (3, Blessing::KarmaPersistence),
            Blessing::AscendantEnergy => (2, Blessing::SpiritualFocus),
            Blessing::Devotion => (2, Blessing::AscendantEnergy),
            Blessing::GracefulPause => (1, Blessing::Devotion),
            Blessing::SpiritualGate => (1, Blessing::GracefulPause),
            Blessing::EternalGrace => (1, Blessing::SpiritualGate),
        })
    }

    pub fn info(&self) -> (&str, &str) {
        match self {
            Blessing::CircleOfGrace => {
                let name = "Circle of Grace";
                let desc = "Increases the sacred zone radius. Luminals follow this zone, converting nearby souls.";
                (name, desc)
            }
            Blessing::KarmaAmplification => {
                let name = "Karma Amplification";
                let desc = "Increases the effectiveness of prayers, converting souls faster.";
                (name, desc)
            }
            Blessing::KarmaPersistence => {
                let name = "Karma Persistence";
                let desc = "Slows corruption, making souls more resistant to it.";
                (name, desc)
            }
            Blessing::SpiritualFocus => {
                let name = "Spiritual Focus";
                let desc = "Increases the focus of prayers, generating spiritual energy faster.";
                (name, desc)
            }
            Blessing::AscendantEnergy => {
                let name = "Ascendant Energy";
                let desc = "Increases the amount of energy generated by each soul.";
                (name, desc)
            }
            Blessing::Devotion => {
                let name = "Devotion";
                let desc = "Souls follow the sacred circle with increased speed and dedication.";
                (name, desc)
            }
            Blessing::GracefulPause => {
                let name = "Graceful Pause";
                let desc = "Increases the time between new soul spawns, preventing flooding.";
                (name, desc)
            }
            Blessing::SpiritualGate => {
                let name = "Spiritual Gate";
                let desc = "Reduces the number of souls entering the realm per wave.";
                (name, desc)
            }
            Blessing::EternalGrace => {
                let name = "Eternal Grace";
                let desc = "Luminals can become incorruptible, converting nearby souls without the need for the sacred circle.";
                (name, desc)
            }
        }
    }
}

pub struct Blessings {
    list: FxHashMap<Blessing, u8>,
}

impl Blessings {
    pub fn new() -> Self {
        let mut list = FxHashMap::default();
        list.insert(Blessing::CircleOfGrace, 1);
        Self { list }
    }

    pub fn level(&self, blessing: &Blessing) -> u8 {
        self.list.get(blessing).cloned().unwrap_or(0)
    }

    pub fn can_unlock(&self, blessing: Blessing) -> bool {
        match blessing.require() {
            Some((lvl, b)) => self.list.get(&b).map_or(false, |v| *v >= lvl),
            None => true,
        }
    }

    pub fn unlock(&mut self, blessing: Blessing) -> bool {
        if !self.can_unlock(blessing) {
            return false;
        }

        let lvl = self.list.entry(blessing).or_insert(0);
        if *lvl < blessing.levels() {
            *lvl += 1;
        }
        true
    }

    pub fn params(&self) -> Params {
        let mut params = PARAMS_START;

        if let Some(lvl) = self.list.get(&Blessing::CircleOfGrace) {
            let max = Blessing::CircleOfGrace.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.sacred_radius - PARAMS_START.sacred_radius;
            params.sacred_radius += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::KarmaAmplification) {
            let max = Blessing::KarmaAmplification.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.karma_change_rate - PARAMS_START.karma_change_rate;
            params.karma_change_rate += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::KarmaPersistence) {
            let max = Blessing::KarmaPersistence.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.karma_expire_rate - PARAMS_START.karma_expire_rate;
            params.karma_expire_rate += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::SpiritualFocus) {
            let max = Blessing::SpiritualFocus.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.energy_time - PARAMS_START.energy_time;
            params.energy_time += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::AscendantEnergy) {
            let max = Blessing::AscendantEnergy.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.energy_amount - PARAMS_START.energy_amount;
            params.energy_amount += (diff as f32 * n).floor() as u64;
        }

        if let Some(lvl) = self.list.get(&Blessing::Devotion) {
            let max = Blessing::Devotion.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.following_speed - PARAMS_START.following_speed;
            params.following_speed += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::GracefulPause) {
            let max = Blessing::GracefulPause.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.slow_spawn_time - PARAMS_START.slow_spawn_time;
            params.slow_spawn_time += diff * n;
        }

        if let Some(lvl) = self.list.get(&Blessing::SpiritualGate) {
            let max = Blessing::SpiritualGate.levels() as f32;
            let n = *lvl as f32 / max;
            let diff = PARAMS_END.block_spawn_souls - PARAMS_START.block_spawn_souls;
            params.block_spawn_souls += (diff as f32 * n).floor() as usize;
        }

        if let Some(lvl) = self.list.get(&Blessing::EternalGrace) {
            if *lvl > 0 {
                params.eternals = true;
            }
        }

        params
    }
}
