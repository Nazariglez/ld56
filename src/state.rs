use crate::params::{Blessing, Blessings, Params, PARAMS_END, PARAMS_START};
use crate::resources::Resources;
use crate::souls::{KarmaConversion, Soul, SoulKind, VisualData};
use rkit::app::{window_height, window_size};
use rkit::draw::{Camera2D, Draw2D, ScreenMode};
use rkit::input::{
    is_key_down, is_key_pressed, is_mouse_btn_down, mouse_position, mouse_wheel_delta, KeyCode,
    MouseButton,
};
use rkit::math::{vec2, Vec2};
use rkit::random;
use rkit::time;
use static_aabb2d_index::{StaticAABB2DIndex, StaticAABB2DIndexBuilder};
use std::f32::consts::TAU;
use strum::IntoEnumIterator;

pub const MAP_SIZE: Vec2 = Vec2::splat(1000.0);
// pub const RESOLUTION: Vec2 = Vec2::new(960.0, 540.0);
pub const RESOLUTION: Vec2 = Vec2::new(640.0, 360.0);
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
    pub good_progress: f32,
    pub bad_progress: f32,

    // res
    pub res: Resources,

    // visual
    pub energy_positions: Vec<Vec2>,
}

impl State {
    pub fn new() -> Result<Self, String> {
        let camera = Camera2D::new(window_size(), ScreenMode::AspectFit(RESOLUTION));
        let position = MAP_SIZE * 0.5;
        let blessings = Blessings::new();
        let params = blessings.params();
        let res = Resources::new()?;

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
            good_progress: 0.0,
            bad_progress: 0.0,

            res,

            energy_positions: Vec::with_capacity(200),
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
                conversion: KarmaConversion::Neutral,
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

        let mut good = 0;
        let mut bad = 0;

        let mut aabb_builder = StaticAABB2DIndexBuilder::<f32>::new(self.souls.len());

        // update entities positions
        self.souls.iter_mut().for_each(|s| {
            if s.is_good() {
                good += 1;
            } else if s.is_bad() {
                bad += 1;
            }

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
            s.pos = s.pos.clamp(Vec2::ZERO, MAP_SIZE); // keep my beloved souls inside the map

            // collect energy
            if is_good_soul {
                s.energy_timer -= dt;
                if s.energy_timer <= 0.0 {
                    s.energy_timer = self.params.energy_time;
                    self.energy += self.params.energy_amount;
                    self.energy_positions
                        .push(self.camera.local_to_screen(s.pos));
                }
            }

            aabb_builder.add(s.pos.x, s.pos.y, s.pos.x + 16.0, s.pos.y + 16.0);
        });

        let aabb_index = aabb_builder.build().unwrap();

        avoid_overlap(&mut self.souls, GRID_SIZE, &aabb_index);

        // update progress
        self.good_progress = good as f32 / self.souls.len() as f32;
        self.bad_progress = bad as f32 / self.souls.len() as f32;

        // update entities karma
        update_karma(
            &mut self.souls,
            dt,
            KARMA_CHANGE_RADIUS,
            self.params.karma_change_rate,
            self.params.karma_expire_rate,
            self.params.eternals,
            &aabb_index,
        );
    }

    pub fn unlock_blessing(&mut self, b: Blessing) -> bool {
        let lvl = self.blessings.level(&b);
        let price = b.price(lvl);
        let can_unlock = self.blessings.can_unlock(b) && self.energy >= price;
        if can_unlock {
            let v = self.blessings.unlock(b);
            if v {
                self.energy -= price;
                self.params = self.blessings.params();
            }

            return v;
        }

        return false;
    }

    pub fn apply_camera(&self, draw: &mut Draw2D) {
        draw.set_camera(&self.camera);
    }

    fn update_camera(&mut self, dt: f32) {
        self.camera_movement(dt);
        self.camera.set_size(window_size().floor());
        self.camera.set_position(self.position.floor());

        // FIXME not working on the engine, delta is always a positive number using wheel (works with trackpad)
        // let w_dt = mouse_wheel_delta().y;
        // if !w_dt.is_zero() {
        //     println!("{w_dt}");
        //     let zoom = (self.camera.zoom() + w_dt);
        //     self.camera.set_zoom(zoom);
        // }

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
    is_mouse_btn_down(MouseButton::Left)
}

fn radial_random_pos(radius: f32) -> Vec2 {
    let angle = random::range(0.0..TAU);
    let r = random::range(0.0..1.0f32).sqrt() * radius;
    vec2(r * angle.cos(), r * angle.sin())
}

pub fn move_towards(from: Vec2, to: Vec2, speed: f32) -> Vec2 {
    let direction = (to - from).normalize_or_zero();
    let movement = direction * speed;
    from + movement
}

pub fn is_close(entity_pos: Vec2, p2: Vec2, radius: f32) -> bool {
    let dist = entity_pos.distance_squared(p2);
    let r = radius * radius;
    dist <= r
}

fn avoid_overlap(souls: &mut [Soul], min_distance: f32, aabb_index: &StaticAABB2DIndex<f32>) {
    const REGULAR_FORCE_MULT: f32 = 0.5;
    const FOLLOWING_FORCE_MUL: f32 = 1.0;

    for i in 0..souls.len() {
        let p1 = souls[i].pos;
        let min = p1 - min_distance;
        // TODO this size is hardcoded and don't check anchors...
        let max = p1 + 16.0 + min_distance;
        let close_souls = aabb_index.query(min.x, min.y, max.x, max.y);
        for n in close_souls {
            if i == n {
                continue;
            }

            let p2 = souls[n].pos;
            let distance = p1.distance(p2);
            if distance < min_distance {
                let overlap = min_distance - distance;
                let direction = (p1 - p2).normalize_or_zero();
                let is_following = souls[i].is_following || souls[n].is_following;
                let force_mul = if is_following {
                    FOLLOWING_FORCE_MUL
                } else {
                    REGULAR_FORCE_MULT
                };

                souls[i].pos += direction * (overlap * force_mul);
                souls[n].pos -= direction * (overlap * force_mul);
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
    aabb_index: &StaticAABB2DIndex<f32>,
) {
    let radius_squared = radius * radius;

    // Store karma changes to apply later
    let karma_updates = souls
        .iter()
        .enumerate()
        .map(|(i, soul)| {
            let mut conversion = KarmaConversion::Neutral;

            // Eternals are immune to corruption
            if matches!(soul.kind(), SoulKind::Eternal) {
                return (i, soul.karma, conversion);
            }

            let max_karma = if use_eternals { 6.0 } else { 2.0 };

            let pos = soul.pos;
            let min = pos - radius;
            let max = pos + radius;
            let close_souls = aabb_index.query(min.x, min.y, max.x, max.y);

            let mut good_souls = 0;
            let mut bad_souls = 0;

            // Count good and bad souls in the radius
            for n in close_souls {
                if n == i {
                    continue;
                }

                let other_soul = &souls[n];
                if other_soul.pos.distance_squared(pos) <= radius_squared {
                    match other_soul.kind() {
                        SoulKind::Shadow => bad_souls += 1,
                        SoulKind::Luminal if other_soul.is_following => good_souls += 1,
                        SoulKind::Eternal => good_souls += 1,
                        _ => {}
                    }
                }
            }

            let expiration = expire_rate * dt;
            let mut karma = if soul.is_following {
                soul.karma + expiration
            } else {
                soul.karma - expiration
            };

            if bad_souls > good_souls {
                let new_rate = rate * 0.2;
                karma = (karma - new_rate * dt).max(-2.0);
                conversion = KarmaConversion::Bad;
            } else if good_souls > bad_souls {
                let extra = (good_souls as f32 / 50.0).clamp(0.0, 1.0);
                let new_rate = rate + extra;
                karma = (karma + new_rate * dt).min(max_karma);
                conversion = KarmaConversion::Good;
            }

            (i, karma.clamp(-2.0, max_karma), conversion)
        })
        .collect::<Vec<_>>();

    // update the karma
    for (i, new_karma, conversion) in karma_updates {
        souls[i].karma = new_karma;
        souls[i].conversion = conversion;
    }
}
