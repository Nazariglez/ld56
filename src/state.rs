use crate::souls::{Soul, SoulKind, VisualData};
use rkit::app::window_size;
use rkit::draw::{Camera2D, Draw2D, ScreenMode};
use rkit::input::{is_key_down, is_mouse_btn_down, mouse_position, KeyCode, MouseButton};
use rkit::math::{vec2, Vec2};
use rkit::random;
use rkit::time;
use std::f32::consts::TAU;

pub const MAP_SIZE: Vec2 = Vec2::splat(1000.0);
pub const RESOLUTION: Vec2 = Vec2::new(960.0, 540.0);
const CAMERA_SPEED: f32 = 120.0;
const FOLLOW_SPEED: f32 = 80.0;
const GRID_SIZE: f32 = 16.0;
const KARMA_CHANGE_RATE: f32 = 0.1;
const KARMA_CHANGE_RADIUS: f32 = 40.0;
const KARMA_EXPIRE_RATE: f32 = 0.01;
const INITIAL_SPAWN_TIME: f32 = 20.0;
const ENERGY_COLLECTION_TIME: f32 = 3.0;

pub struct State {
    pub camera: Camera2D,
    pub position: Vec2,
    pub souls: Vec<Soul>,
    pub ids: u64,

    // mouse
    pub mouse_radius: f32,
    pub mouse_pos: Vec2,
    pub is_guiding: bool,

    // spawner
    pub spawn_time: f32,  // time to reset timer
    pub spawn_timer: f32, // current spawn timer
    pub spawn_num: usize, // number of souls spawned

    // stats
    pub energy: u64,
    pub energy_time: f32,
}

impl State {
    pub fn new() -> Result<Self, String> {
        let camera = Camera2D::new(window_size(), ScreenMode::AspectFit(RESOLUTION));
        let position = MAP_SIZE * 0.5;
        Ok(Self {
            camera,
            position,
            souls: vec![],
            ids: 0,

            mouse_radius: 60.0,
            mouse_pos: Vec2::ZERO,
            is_guiding: false,

            spawn_time: INITIAL_SPAWN_TIME,
            spawn_timer: INITIAL_SPAWN_TIME,
            spawn_num: 1,

            energy: 0,
            energy_time: ENERGY_COLLECTION_TIME,
        })
    }

    pub fn spawn_souls(&mut self, n: usize, kind: Option<SoulKind>) {
        let map_radius = MAP_SIZE.min_element() * 0.3;
        for i in 0..n {
            let range = match kind {
                Some(k) => match k {
                    SoulKind::Luminal => 1.0..2.0,
                    SoulKind::Neutral => -0.9..0.9,
                    SoulKind::Shadow => -2.0..1.0,
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
                energy_timer: ENERGY_COLLECTION_TIME,
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

        self.spawn_timer -= dt;
        if self.spawn_timer <= 0.0 {
            self.spawn_time = (self.spawn_time - 0.5).max(5.0);
            self.spawn_timer = self.spawn_time;
            self.spawn_num = (self.spawn_num + 1).min(20);
            self.spawn_souls(self.spawn_num, Some(SoulKind::Neutral));
        }

        // update entities positions
        self.souls.iter_mut().for_each(|s| {
            s.is_following = false;

            let is_luminal = matches!(s.kind(), SoulKind::Luminal);
            if self.is_guiding && is_luminal && is_close(s.pos, self.mouse_pos, self.mouse_radius) {
                s.is_following = true;
                s.pos = move_towards(s.pos, self.mouse_pos, FOLLOW_SPEED * dt);
            }

            s.idle_movement(elapsed, dt);

            if is_luminal {
                s.energy_timer -= dt;
                if s.energy_timer <= 0.0 {
                    s.energy_timer = self.energy_time;
                    self.energy += 1; // TODO this must be parametrized
                }
            }
        });
        avoid_overlap(&mut self.souls, GRID_SIZE);

        // update entities karma
        update_karma(&mut self.souls, dt, KARMA_CHANGE_RADIUS, KARMA_CHANGE_RATE);
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

pub fn update_karma(souls: &mut [Soul], dt: f32, radius: f32, rate: f32) {
    // collect karma and apply later (let's please borrow checker for now)
    let karma = souls
        .iter()
        .map(|soul| {
            let (luminals, shadows) = count_souls(souls, soul.id, soul.pos, radius);

            // TODO check if we're in sacred zone
            // karma decrease slowly or increase if following
            let expiration = KARMA_EXPIRE_RATE * dt;
            let mut karma = if soul.is_following {
                soul.karma + expiration
            } else {
                soul.karma - expiration
            };

            // update karma
            if shadows > luminals {
                // shadows have slow rate to compensate expiration time
                let new_rate = rate * 0.2;
                karma = (karma - new_rate * dt).max(-2.0);
            } else if luminals > shadows {
                // more luminals following will convert faster (50 for 1 extra point)
                let extra = (luminals as f32 / 50.0).clamp(0.0, 1.0);
                let new_rate = rate + extra;
                karma = (karma + new_rate * dt).min(2.0);
            }

            karma.clamp(-2.0, 2.0)
        })
        .collect::<Vec<_>>();

    // apply collected karma
    for (i, soul) in souls.iter_mut().enumerate() {
        soul.karma = karma[i];
    }
}

pub fn count_souls(souls: &[Soul], id: u64, pos: Vec2, radius: f32) -> (usize, usize) {
    let mut luminals = 0;
    let mut shadows = 0;

    for other_soul in souls {
        if other_soul.id == id {
            continue;
        }

        let dist = other_soul.pos.distance_squared(pos);

        match other_soul.kind() {
            SoulKind::Luminal if other_soul.is_following => {
                if dist <= radius * radius {
                    luminals += 1
                }
            }
            SoulKind::Shadow => {
                if dist <= radius * radius {
                    shadows += 1
                }
            }
            _ => {}
        }
    }

    (luminals, shadows)
}
