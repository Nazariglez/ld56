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
    karma_change_rate: 0.09,
    karma_expire_rate: 0.015,
    energy_time: 3.0,
    energy_amount: 1,
    following_speed: 75.0,
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
            Blessing::CircleOfGrace => 15.0,
            Blessing::KarmaAmplification => 40.0,
            Blessing::KarmaPersistence => 90.0,
            Blessing::SpiritualFocus => 190.0,
            Blessing::AscendantEnergy => 390.0,
            Blessing::Devotion => 790.0,
            Blessing::GracefulPause => 1590.0,
            Blessing::SpiritualGate => 3190.0,
            Blessing::EternalGrace => 6390.0,
        };

        (base.powf(multi)) as _
    }

    pub fn require(&self) -> Option<(u8, Blessing)> {
        Some(match self {
            Blessing::CircleOfGrace => return None,
            Blessing::KarmaAmplification => (4, Blessing::CircleOfGrace),
            Blessing::KarmaPersistence => (4, Blessing::KarmaAmplification),
            Blessing::SpiritualFocus => (3, Blessing::KarmaPersistence),
            Blessing::AscendantEnergy => (3, Blessing::SpiritualFocus),
            Blessing::Devotion => (2, Blessing::AscendantEnergy),
            Blessing::GracefulPause => (2, Blessing::Devotion),
            Blessing::SpiritualGate => (1, Blessing::GracefulPause),
            Blessing::EternalGrace => (1, Blessing::SpiritualGate),
        })
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
