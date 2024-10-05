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

fn main() -> Result<(), String> {
    rkit::init_with(setup).on_update(update).run()
}

fn setup() -> State {
    let mut state = State::new().unwrap();
    state.spawn_souls(100, None);
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
        let color = match s.kind() {
            SoulKind::Luminal => Color::BLUE,
            SoulKind::Neutral => Color::GRAY,
            SoulKind::Shadow => Color::MAROON,
        };
        let pos = s.pos + s.visuals.pos_offset;
        draw.rect(pos, Vec2::splat(16.0)).color(color);
    });

    draw.circle(state.mouse_radius)
        .color(Color::MAGENTA)
        .alpha(0.05)
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
