use rkit::math::{vec2, Vec2};
use rkit::random;
use std::f32::consts::TAU;

#[derive(Copy, Clone, Debug, Default)]
pub enum SoulKind {
    #[default]
    Neutral,
    Shadow,
    Luminal,
    Eternal,
}

pub struct VisualData {
    pub pos_offset: Vec2,
    pub phase_shift: f32,
    pub speed_multiplier: f32,
}

impl VisualData {
    pub fn new() -> Self {
        let pos_offset = Vec2::ZERO;
        let phase_shift = random::range(0.0..TAU);
        let speed_multiplier = random::range(0.8..1.2);
        Self {
            pos_offset,
            phase_shift,
            speed_multiplier,
        }
    }

    pub fn idle_movement(&mut self, t: f32) {
        let frequency = 10.0;
        let v_amplitude = 2.0;
        let h_amplitude = 8.0;
        let h_offset = h_amplitude * ((t * self.speed_multiplier) + self.phase_shift).cos();
        let v_offset =
            v_amplitude * ((t * frequency * self.speed_multiplier) + self.phase_shift).sin();

        self.pos_offset = vec2(h_offset, v_offset);
    }
}

#[derive(Copy, Clone)]
pub enum KarmaConversion {
    Neutral,
    Good,
    Bad,
}

pub struct Soul {
    pub id: u64,
    pub karma: f32,
    pub pos: Vec2,
    pub is_following: bool,
    pub energy_timer: f32,
    pub visuals: VisualData,
    pub conversion: KarmaConversion,
}

impl PartialEq for Soul {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}

impl Soul {
    pub fn kind(&self) -> SoulKind {
        if self.is_neutral() {
            SoulKind::Neutral
        } else if self.is_bad() {
            SoulKind::Shadow
        } else if self.karma >= 5.0 {
            SoulKind::Eternal
        } else {
            SoulKind::Luminal
        }
    }

    pub fn is_good(&self) -> bool {
        self.karma >= 1.0
    }

    pub fn is_bad(&self) -> bool {
        self.karma <= -1.0
    }

    pub fn is_neutral(&self) -> bool {
        self.karma > -1.0 && self.karma < 1.0
    }

    pub fn idle_movement(&mut self, t: f32, dt: f32) {
        // move the souls a bit of their position
        let h_mul = random::range(-0.1..0.1);
        let v_mul = random::range(-0.1..0.1);
        self.pos += vec2(h_mul, v_mul) * dt * 200.0;

        // increase visual movement
        self.visuals.idle_movement(t);
    }
}
