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
    energy_amount: 10,
    following_speed: 150.0,
    eternals: true,
    slow_spawn_time: 20.0,
    block_spawn_souls: 18,
};

// Improvements
pub struct Improvements {}
