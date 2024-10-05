use crate::params::{Blessing, Blessings, Params, PARAMS_END, PARAMS_START};
use crate::souls::{Soul, SoulKind, VisualData};
use rkit::app::{window_height, window_size};
use rkit::draw::{Camera2D, Draw2D, ScreenMode};
use rkit::input::{
    is_key_down, is_key_pressed, is_mouse_btn_down, mouse_position, KeyCode, MouseButton,
};
use rkit::math::{vec2, Vec2};
use rkit::random;
use rkit::time;
use std::f32::consts::TAU;
use strum::IntoEnumIterator;

pub const MAP_SIZE: Vec2 = Vec2::splat(1000.0);
pub const RESOLUTION: Vec2 = Vec2::new(960.0, 540.0);
const CAMERA_SPEED: f32 = 120.0;
const GRID_SIZE: f32 = 20.0;
const KARMA_CHANGE_RADIUS: f32 = 40.0;
const INITIAL_SPAWN_TIME: f32 = 20.0;

pub struct State {
    pub camera: Camera2D,
    pub position: Vec2,
    pub souls: Vec<Soul>,
    pub ids: u64,

    pub blessings: Blessings,
    pub params: Params,

    // mouse
    pub mouse_pos: Vec2,
    pub is_guiding: bool,

    // spawner
    pub spawn_time: f32,  // time to reset timer
    pub spawn_timer: f32, // current spawn timer
    pub spawn_num: usize, // number of souls spawned

    // stats
    pub energy: u64,
}

impl State {
    pub fn new() -> Result<Self, String> {
        let camera = Camera2D::new(window_size(), ScreenMode::AspectFit(RESOLUTION));
        let position = MAP_SIZE * 0.5;
        let blessings = Blessings::new();
        let params = blessings.params();

        Ok(Self {
            camera,
            position,
            souls: vec![],
            ids: 0,

            blessings,
            params,

            mouse_pos: Vec2::ZERO,
            is_guiding: false,

            spawn_time: INITIAL_SPAWN_TIME,
            spawn_timer: INITIAL_SPAWN_TIME,
            spawn_num: 1,

            energy: 0,
        })
    }

    pub fn spawn_souls(&mut self, n: usize, kind: Option<SoulKind>) {
        let map_radius = MAP_SIZE.min_element() * 0.4;
        for i in 0..n {
            let range = match kind {
                Some(k) => match k {
                    SoulKind::Luminal => 1.0..2.0,
                    SoulKind::Neutral => -0.9..0.9,
                    SoulKind::Shadow => -2.0..1.0,
                    SoulKind::Eternal => 5.0..6.0,
                },
                None => -2.0..2.0,
            };

            let karma: f32 = random::range(range);
            let pos = (MAP_SIZE * 0.5 + radial_random_pos(map_radius)).round();
            self.souls.push(Soul {
                id: self.ids,
                karma,
                pos,
                is_following: false,
                energy_timer: self.params.energy_time,
                visuals: VisualData::new(),
            });
            self.ids += 1;
        }
    }

    pub fn update(&mut self) {
        let elapsed = time::elapsed_f32();
        let dt = time::delta_f32();

        self.update_camera(dt);
        self.is_guiding = is_guiding_souls();

        // Manage the spawner
        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_time = (self.spawn_time - 0.5).max(5.0);
            self.spawn_timer = self.spawn_time + self.params.slow_spawn_time;
            self.spawn_num = (self.spawn_num + 1).min(20);
            let souls_to_spawn = self
                .spawn_num
                .checked_sub(self.params.block_spawn_souls)
                .unwrap_or(1);
            self.spawn_souls(souls_to_spawn, Some(SoulKind::Neutral));
        }

        // update entities positions
        self.souls.iter_mut().for_each(|s| {
            s.is_following = false;

            let is_good_soul = s.is_good();
            if self.is_guiding
                && is_good_soul
                && is_close(s.pos, self.mouse_pos, self.params.sacred_radius)
            {
                s.is_following = true;
                s.pos = move_towards(s.pos, self.mouse_pos, self.params.following_speed * dt);
            }

            s.idle_movement(elapsed, dt);

            // collect energy
            if is_good_soul {
                s.energy_timer -= dt;
                if s.energy_timer <= 0.0 {
                    s.energy_timer = self.params.energy_time;
                    self.energy += self.params.energy_amount;
                }
            }
        });
        avoid_overlap(&mut self.souls, GRID_SIZE);

        // update entities karma
        update_karma(
            &mut self.souls,
            dt,
            KARMA_CHANGE_RADIUS,
            self.params.karma_change_rate,
            self.params.karma_expire_rate,
            self.params.eternals,
        );

        // TODO remove temporal upgrades
        Blessing::iter().enumerate().for_each(|(n, b)| {
            let key = match n {
                0 => Some(KeyCode::Digit0),
                1 => Some(KeyCode::Digit1),
                2 => Some(KeyCode::Digit2),
                3 => Some(KeyCode::Digit3),
                4 => Some(KeyCode::Digit4),
                5 => Some(KeyCode::Digit5),
                6 => Some(KeyCode::Digit6),
                7 => Some(KeyCode::Digit7),
                8 => Some(KeyCode::Digit8),
                9 => Some(KeyCode::Digit9),
                _ => None,
            };

            if let Some(k) = key {
                let lvl = self.blessings.level(&b);
                let price = b.price(lvl + 1);
                let can_unlock = self.blessings.can_unlock(b) && self.energy >= price;
                if can_unlock && is_key_pressed(k) {
                    let v = self.blessings.unlock(b);
                    if !v {
                        println!("Something went wrong... {:?}: {}", b, lvl);
                    } else {
                        self.energy -= price;
                        self.params = self.blessings.params();
                    }
                }
            }
        })
    }

    pub fn apply_camera(&self, draw: &mut Draw2D) {
        draw.set_camera(&self.camera);
    }

    fn update_camera(&mut self, dt: f32) {
        self.camera_movement(dt);
        self.camera.set_size(window_size().floor());
        self.camera.set_position(self.position.floor());
        // self.camera.set_zoom(0.8);
        self.camera.update();
        self.mouse_pos = self.camera.screen_to_local(mouse_position());
    }

    fn camera_movement(&mut self, dt: f32) {
        let mut mul = Vec2::ZERO;
        if is_moving_left() {
            mul.x = -1.0;
        } else if is_moving_right() {
            mul.x = 1.0;
        }

        if is_moving_up() {
            mul.y = -1.0;
        } else if is_moving_down() {
            mul.y = 1.0;
        }

        self.position += CAMERA_SPEED * mul * dt;
    }
}

fn is_moving_left() -> bool {
    is_key_down(KeyCode::KeyA)
}
fn is_moving_right() -> bool {
    is_key_down(KeyCode::KeyD)
}

fn is_moving_up() -> bool {
    is_key_down(KeyCode::KeyW)
}

fn is_moving_down() -> bool {
    is_key_down(KeyCode::KeyS)
}

fn is_guiding_souls() -> bool {
    is_mouse_btn_down(MouseButton::Right)
}

fn radial_random_pos(radius: f32) -> Vec2 {
    let angle = random::range(0.0..TAU);
    let r = random::range(0.0..1.0f32).sqrt() * radius;
    vec2(r * angle.cos(), r * angle.sin())
}

fn move_towards(from: Vec2, to: Vec2, speed: f32) -> Vec2 {
    let direction = (to - from).normalize_or_zero();
    let movement = direction * speed;
    from + movement
}

fn is_close(entity_pos: Vec2, p2: Vec2, radius: f32) -> bool {
    let dist = entity_pos.distance_squared(p2);
    let r = radius * radius;
    dist <= r
}

fn avoid_overlap(souls: &mut [Soul], min_distance: f32) {
    const REGULAR_FORCE_MULT: f32 = 0.5;
    const FOLLOWING_FORCE_MUL: f32 = 1.0;

    for i in 0..souls.len() {
        for j in i + 1..souls.len() {
            let p1 = souls[i].pos;
            let p2 = souls[j].pos;
            let distance = p1.distance(p2);
            if distance < min_distance {
                let overlap = min_distance - distance;
                let direction = (p1 - p2).normalize_or_zero();
                let is_following = souls[i].is_following || souls[j].is_following;
                let force_mul = if is_following {
                    FOLLOWING_FORCE_MUL
                } else {
                    REGULAR_FORCE_MULT
                };

                souls[i].pos += direction * (overlap * force_mul);
                souls[j].pos -= direction * (overlap * force_mul);
            }
        }
    }
}

pub fn update_karma(
    souls: &mut [Soul],
    dt: f32,
    radius: f32,
    rate: f32,
    expire_rate: f32,
    use_eternals: bool,
) {
    // collect karma and apply later (let's please borrow checker for now)
    let karma = souls
        .iter()
        .map(|soul| {
            // Eternals cannot be corrupted
            if matches!(soul.kind(), SoulKind::Eternal) {
                return soul.karma;
            }

            let max_karma = if use_eternals { 6.0 } else { 2.0 };

            let (good_souls, bad_souls) = count_souls(souls, soul.id, soul.pos, radius);

            // karma decrease slowly or increase if following
            let expiration = expire_rate * dt;
            let mut karma = if soul.is_following {
                soul.karma + expiration
            } else {
                soul.karma - expiration
            };

            // update karma
            if bad_souls > good_souls {
                // shadows have slow rate to compensate expiration time
                let new_rate = rate * 0.2;
                karma = (karma - new_rate * dt).max(-2.0);
            } else if good_souls > bad_souls {
                // more luminals following will convert faster (50 for 1 extra point)
                let extra = (good_souls as f32 / 50.0).clamp(0.0, 1.0);
                let new_rate = rate + extra;
                karma = (karma + new_rate * dt).min(max_karma);
            }

            karma.clamp(-2.0, max_karma)
        })
        .collect::<Vec<_>>();

    // apply collected karma
    for (i, soul) in souls.iter_mut().enumerate() {
        soul.karma = karma[i];
    }
}

pub fn count_souls(souls: &[Soul], id: u64, pos: Vec2, radius: f32) -> (usize, usize) {
    let mut good = 0;
    let mut bad = 0;

    for other_soul in souls {
        if other_soul.id == id {
            continue;
        }

        let dist = other_soul.pos.distance_squared(pos);
        if dist <= radius * radius {
            match other_soul.kind() {
                SoulKind::Shadow => bad += 1,

                // Luminals need to be "praying/following" to count
                SoulKind::Luminal if other_soul.is_following => good += 1,

                // Eternals always count people don't need to pray
                SoulKind::Eternal => good += 1,
                _ => {}
            }
        }
    }

    (good, bad)
}
