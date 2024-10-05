use crate::souls::{Soul, SoulKind, VisualData};
use rkit::app::window_size;
use rkit::draw::{Camera2D, Draw2D, ScreenMode};
use rkit::input::{is_key_down, is_key_pressed, mouse_position, KeyCode};
use rkit::math::{vec2, Vec2};
use rkit::random;
use rkit::time;
use std::f32::consts::TAU;

pub const MAP_SIZE: Vec2 = Vec2::splat(1000.0);
pub const RESOLUTION: Vec2 = Vec2::new(960.0, 540.0);
const MOVEMENT_SPEED: f32 = 120.0;

pub struct State {
    pub camera: Camera2D,
    pub position: Vec2,
    pub souls: Vec<Soul>,

    // mouse
    pub mouse_radius: f32,
    pub mouse_pos: Vec2,
}

impl State {
    pub fn new() -> Result<Self, String> {
        let camera = Camera2D::new(window_size(), ScreenMode::AspectFit(RESOLUTION));
        let position = MAP_SIZE * 0.5;
        Ok(Self {
            camera,
            position,
            souls: vec![],

            mouse_radius: 50.0,
            mouse_pos: Vec2::ZERO,
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
            println!("karma {}", karma);
            let pos = (MAP_SIZE * 0.5 + radial_random_pos(map_radius)).round();
            self.souls.push(Soul {
                karma,
                pos,
                visuals: VisualData::new(),
            });
        }
    }

    pub fn update(&mut self) {
        let elapsed = time::elapsed_f32();
        let dt = time::delta_f32();

        self.update_camera(dt);
        self.souls
            .iter_mut()
            .for_each(|s| s.idle_movement(elapsed, dt));
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
        if move_left() {
            mul.x = -1.0;
        } else if move_right() {
            mul.x = 1.0;
        }

        if move_up() {
            mul.y = -1.0;
        } else if move_down() {
            mul.y = 1.0;
        }

        self.position += MOVEMENT_SPEED * mul * dt;
    }
}

fn move_left() -> bool {
    is_key_down(KeyCode::KeyA)
}
fn move_right() -> bool {
    is_key_down(KeyCode::KeyD)
}

fn move_up() -> bool {
    is_key_down(KeyCode::KeyW)
}

fn move_down() -> bool {
    is_key_down(KeyCode::KeyS)
}

fn radial_random_pos(radius: f32) -> Vec2 {
    let angle = random::range(0.0..TAU);
    let r = random::range(0.0..1.0f32).sqrt() * radius;
    vec2(r * angle.cos(), r * angle.sin())
}
