mod souls;
mod state;

use crate::souls::SoulKind;
use crate::state::{State, MAP_SIZE, RESOLUTION};
use rkit::app::window_size;
use rkit::draw::create_draw_2d;
use rkit::gfx::Color;
use rkit::input::KeyCode::WakeUp;
use rkit::math::{vec2, Vec2};
use rkit::{gfx, time};
use std::fmt::format;

const LUMINAL_COLOR: Color = Color::rgb(0.0, 0.793, 1.0);
const SHADOW_COLOR: Color = Color::rgb(0.612, 0.029, 0.029);
const NEUTRAL_COLOR: Color = Color::GRAY;

fn main() -> Result<(), String> {
    rkit::init_with(setup).on_update(update).run()
}

fn setup() -> State {
    let mut state = State::new().unwrap();
    state.spawn_souls(20, None);
    state.spawn_souls(130, Some(SoulKind::Neutral));
    state.spawn_souls(10, Some(SoulKind::Luminal));
    state.spawn_souls(10, Some(SoulKind::Shadow));
    state
}

fn update(state: &mut State) {
    state.update();

    let mut draw = create_draw_2d();
    state.apply_camera(&mut draw);

    draw.clear(Color::BLACK);

    //draw bounds
    draw.rect(Vec2::ZERO, MAP_SIZE)
        .stroke_color(Color::GRAY.with_alpha(0.5))
        .stroke(4.0);

    // draw.rect(MAP_SIZE * 0.5, Vec2::splat(16.0));
    state.souls.iter().for_each(|s| {
        let (color, alpha) = match s.kind() {
            SoulKind::Luminal => (LUMINAL_COLOR, 1.0),
            SoulKind::Neutral => {
                let k = s.karma;
                let color = if k > 0.0 {
                    lerp_color(NEUTRAL_COLOR, LUMINAL_COLOR, k)
                } else if k < 0.0 {
                    lerp_color(NEUTRAL_COLOR, SHADOW_COLOR, -k)
                } else {
                    NEUTRAL_COLOR
                };
                (color, 0.3)
            }
            SoulKind::Shadow => (SHADOW_COLOR, 1.0),
        };
        let pos = s.pos + s.visuals.pos_offset;
        draw.rect(pos, Vec2::splat(16.0)).color(color).alpha(alpha);
    });

    let (color, alpha) = if state.is_guiding {
        (Color::MAGENTA, 0.3)
    } else {
        (Color::YELLOW, 0.05)
    };
    draw.circle(state.mouse_radius)
        .color(color)
        .alpha(alpha)
        .position(state.mouse_pos - state.mouse_radius);

    gfx::render_to_frame(&draw).unwrap();

    // debug
    let mut draw = create_draw_2d();
    // let camera_ratio = state.camera.ratio();
    // let camera_bounds = state.camera.bounds();

    draw.text(&format!(
        "FPS: {:.0}, ms: {:.0}",
        time::fps(),
        time::delta_f32() * 1000.0
    ))
    .position(Vec2::splat(10.0))
    .size(10.0);

    let (luminal_count, neutral_count, shadow_count) =
        state
            .souls
            .iter()
            .fold((0, 0, 0), |(l, n, s), soul| match soul.kind() {
                SoulKind::Luminal => (l + 1, n, s),
                SoulKind::Neutral => (l, n + 1, s),
                SoulKind::Shadow => (l, n, s + 1),
            });

    draw.text(&format!(
        "Luminals: {}\nNeutrals: {}\nShadows: {}\n",
        luminal_count, neutral_count, shadow_count
    ))
    .position(vec2(10.0, 30.0))
    .size(10.0);

    // draw.rect(camera_bounds.origin, camera_bounds.size * camera_ratio)
    //     .stroke_color(Color::GRAY)
    //     .stroke(3.0);

    gfx::render_to_frame(&draw).unwrap();
}

pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    c1 + (c2 - c1) * t
}
