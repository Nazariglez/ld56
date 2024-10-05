mod params;
mod souls;
mod state;

use crate::souls::SoulKind;
use crate::state::{State, MAP_SIZE, RESOLUTION};
use rkit::app::{window_size, window_width};
use rkit::draw::create_draw_2d;
use rkit::gfx::Color;
use rkit::input::KeyCode::WakeUp;
use rkit::math::{vec2, Vec2};
use rkit::{gfx, time};
use std::fmt::format;

const LUMINAL_COLOR: Color = Color::rgb(0.0, 0.793, 1.0);
const SHADOW_COLOR: Color = Color::rgb(0.612, 0.029, 0.029);
const ETERNAL_COLOR: Color = Color::rgb(1.0, 0.483, 0.0);
const NEUTRAL_COLOR: Color = Color::GRAY;

fn main() -> Result<(), String> {
    rkit::init_with(setup).on_update(update).run()
}

fn setup() -> State {
    let mut state = State::new().unwrap();
    state.spawn_souls(20, None);
    state.spawn_souls(50, Some(SoulKind::Neutral));
    state.spawn_souls(5, Some(SoulKind::Luminal));
    state.spawn_souls(5, Some(SoulKind::Shadow));
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
            SoulKind::Luminal => (LUMINAL_COLOR, 1.0),
            SoulKind::Eternal => (ETERNAL_COLOR, 1.0),
        };
        let pos = s.pos + s.visuals.pos_offset;
        draw.rect(pos, Vec2::splat(16.0)).color(color).alpha(alpha);
        draw.text(&format!("{:.2}", s.karma))
            .size(10.0)
            .position(pos - Vec2::Y * 16.0);
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

    let (eternals, luminals, neutrals, shadows) =
        state
            .souls
            .iter()
            .fold((0, 0, 0, 0), |(e, l, n, s), soul| match soul.kind() {
                SoulKind::Neutral => (e, l, n + 1, s),
                SoulKind::Shadow => (e, l, n, s + 1),
                SoulKind::Luminal => (e, l + 1, n, s),
                SoulKind::Eternal => (e + 1, l, n, s),
            });

    let total = state.souls.len();
    let good = (((eternals + luminals) as f32) / total as f32) * 100.0;
    let bad = (((shadows) as f32) / total as f32) * 100.0; // todo check bad with sould.is_bad
    let eternal_percent = (eternals as f32 / total as f32) * 100.0;
    let luminal_percent = (luminals as f32 / total as f32) * 100.0;
    let neutral_percent = (neutrals as f32 / total as f32) * 100.0;
    let shadow_percent = (shadows as f32 / total as f32) * 100.0;
    draw.text(&format!(
        "
Good: {:.0}%
Bad: {:.0}%
--
Eternals: {:.0}% ({})
Luminals: {:.0}% ({})
Neutrals: {:.0}% ({})
Shadows: {:.0}% ({})
Total: {}",
        good,
        bad,
        eternal_percent,
        eternals,
        luminal_percent,
        luminals,
        neutral_percent,
        neutrals,
        shadow_percent,
        shadows,
        total
    ))
    .position(vec2(10.0, 30.0))
    .size(10.0);

    draw.text(&format!("Spiritual Energy: {}", state.energy))
        .size(12.0)
        .anchor(vec2(1.0, 0.0))
        .h_align_right()
        .translate(vec2(window_width() - 10.0, 20.0));

    // draw.rect(camera_bounds.origin, camera_bounds.size * camera_ratio)
    //     .stroke_color(Color::GRAY)
    //     .stroke(3.0);

    gfx::render_to_frame(&draw).unwrap();
}

pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    c1 + (c2 - c1) * t
}
