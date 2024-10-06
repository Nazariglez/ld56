mod params;
mod resources;
mod souls;
mod state;

use crate::params::Blessing;
use crate::souls::{KarmaConversion, SoulKind};
use crate::state::{is_close, move_towards, Mode, State, MAP_SIZE, RESOLUTION};
use rkit::app::{window_height, window_size, window_width};
use rkit::draw::{create_draw_2d, Draw2D, Transform2D};
use rkit::gfx::Color;
use rkit::input::{is_mouse_btn_pressed, keys_pressed, mouse_position, MouseButton};
use rkit::math::{vec2, Rect, Vec2};
use rkit::{gfx, time};
use std::thread::spawn;
use strum::IntoEnumIterator;

const LUMINAL_COLOR: Color = Color::rgb(0.171, 0.863, 0.929);
const SHADOW_COLOR: Color = Color::rgb(0.4325, 0.0489, 0.0872);
const ETERNAL_COLOR: Color = Color::rgb(1.0, 0.596, 0.171);
const NEUTRAL_COLOR: Color = Color::WHITE;

fn main() -> Result<(), String> {
    rkit::init_with(setup).on_update(update).run()
}

fn setup() -> State {
    State::new().unwrap()
}

fn init_spawn(state: &mut State) {
    state.spawn_souls(30, None);
    state.spawn_souls(50, Some(SoulKind::Neutral));
    state.spawn_souls(8, Some(SoulKind::Luminal));
    state.spawn_souls(8, Some(SoulKind::Shadow));
}

fn update(state: &mut State) {
    state.update();

    let dt = time::delta_f32();
    let elapsed = time::elapsed_f32();
    let win_size = window_size();
    let mouse_pos = mouse_position();

    let mut draw = create_draw_2d();
    state.apply_camera(&mut draw);

    draw.clear(Color::BLACK);

    let cam_bounds = state.camera.bounds();
    let cam_pos = state.camera.position();
    let tex_size = state.res.tile.size();
    let offset = cam_pos.rem_euclid(tex_size);
    draw.pattern(&state.res.tile)
        .position(cam_bounds.origin)
        .size(cam_bounds.size)
        .image_offset(offset);

    match state.mode {
        Mode::Menu => {
            gfx::render_to_frame(&draw).unwrap();
            draw_menu(state);
            return;
        }
        Mode::End => {}
        _ => {}
    }

    //draw bounds
    draw.rect(Vec2::ZERO, MAP_SIZE)
        .stroke_color(Color::GRAY.with_alpha(0.5))
        .stroke(4.0);

    let alpha = if state.is_guiding { 0.3 } else { 0.04 };
    let circle_size = state.params.sacred_radius.floor() * 2.0;
    let elapsed_time = elapsed.sin().abs();
    let min_size = circle_size * 0.8;
    let animated_size = min_size + (circle_size - min_size) * elapsed_time;

    draw.circle(state.params.sacred_radius)
        .alpha(0.01)
        .position(state.mouse_pos - state.params.sacred_radius);
    draw.image(&state.res.circle)
        .alpha(alpha)
        .size(Vec2::splat(animated_size))
        .anchor(Vec2::splat(0.5))
        .translate(state.mouse_pos);

    state.souls.iter().for_each(|s| {
        // skip if it's not visible
        if !state
            .camera
            .is_rect_visible(Rect::new(s.pos, Vec2::splat(16.0)))
        {
            return;
        }

        let (tex, color, alpha) = match s.kind() {
            SoulKind::Neutral => {
                let k = s.karma;
                let color = if k > 0.0 {
                    lerp_color(NEUTRAL_COLOR, LUMINAL_COLOR, k)
                } else if k < 0.0 {
                    lerp_color(NEUTRAL_COLOR, SHADOW_COLOR, -k)
                } else {
                    NEUTRAL_COLOR
                };
                (&state.res.neutral, color, 0.9)
            }
            SoulKind::Shadow => (&state.res.shadow, SHADOW_COLOR, 1.0),
            SoulKind::Luminal => (&state.res.luminal, LUMINAL_COLOR, 1.0),
            SoulKind::Eternal => (&state.res.eternal, ETERNAL_COLOR, 1.0),
        };

        let pos = s.pos + s.visuals.pos_offset;
        // shadow
        draw.image(&state.res.neutral)
            .translate(pos + vec2(16.0, state.res.neutral.height()))
            .anchor(vec2(0.0, 1.0))
            .color(Color::BLACK)
            .skew(vec2(0.6, -0.6))
            .scale(vec2(1.0, 1.5))
            .alpha(0.3);

        // entity

        let conversion_color = match s.conversion {
            KarmaConversion::Neutral => None,
            KarmaConversion::Good => (!s.is_good()).then_some(LUMINAL_COLOR),
            KarmaConversion::Bad => (!s.is_bad()).then_some(SHADOW_COLOR),
        };

        if let Some(cc) = conversion_color {
            draw.image(&state.res.karma_circle)
                .anchor(Vec2::splat(0.5))
                .scale(Vec2::splat(1.4))
                .translate(pos + 8.0)
                .alpha(0.5)
                .color(cc);
        }

        draw.image(tex).position(pos).alpha(alpha);
        draw.image(&state.res.shirt).position(pos).color(color);
    });

    gfx::render_to_frame(&draw).unwrap();

    // debug
    let mut draw = create_draw_2d();

    // spiritual energy
    let spiritual_energy_pos = vec2(win_size.x - 20.0, 20.0);
    draw.image(&state.res.souls_icon)
        .anchor(vec2(1.0, 0.0))
        .scale(Vec2::splat(2.0))
        .translate(spiritual_energy_pos);

    // progress bar
    let scale = 2.0;
    let t_size = state.res.bar.size();
    let xx = (win_size * 0.5 - t_size * 0.5 * scale).x;
    draw.image(&state.res.bar)
        .translate(vec2(xx, 20.0))
        .color(Color::GRAY)
        .alpha(0.5)
        .scale(Vec2::splat(2.0));

    draw.image(&state.res.bar)
        .translate(vec2(xx, 20.0))
        .scale(Vec2::splat(2.0))
        .color(LUMINAL_COLOR)
        .crop(
            Vec2::ZERO,
            (t_size * vec2(state.good_progress, 1.0)).round(),
        );

    let bad_progress_width = t_size.x * state.bad_progress;
    draw.image(&state.res.bar)
        .anchor(vec2(1.0, 0.0))
        .translate(vec2(xx + t_size.x * scale, 20.0))
        .scale(Vec2::splat(2.0))
        .color(SHADOW_COLOR)
        .crop(
            vec2(t_size.x - bad_progress_width, 0.0),
            vec2(bad_progress_width, t_size.y),
        );

    draw.image(&state.res.bar_outline)
        .translate(vec2(xx, 20.0))
        .scale(Vec2::splat(2.0));

    // spiritual energy movement
    let spirit_target = spiritual_energy_pos + vec2(-16.0, 16.0);
    state.energy_positions.iter_mut().for_each(|p| {
        draw.image(&state.res.souls_icon)
            .anchor(Vec2::splat(0.5))
            .translate(*p);

        *p = move_towards(*p, spirit_target, 800.0 * dt);
    });

    state
        .energy_positions
        .retain(|p| !is_close(*p, spirit_target, 16.0));

    // blessings
    let mut tooltip: Option<(Blessing, Vec2)> = None;
    let offset = Vec2::splat(20.0);
    let padding = Vec2::splat(48.0);
    let grid_size = 3;
    Blessing::iter().enumerate().for_each(|(i, b)| {
        let grid = vec2((i % grid_size) as f32, (i / grid_size) as f32);
        let pos = offset + padding * grid;

        let lvl = state.blessings.level(&b);
        let price = b.price(lvl);
        let can_unlock = state.blessings.can_unlock(b);

        let alpha = if lvl == 0 && can_unlock {
            0.8
        } else if lvl >= 1 {
            1.0
        } else {
            0.3
        };

        let mut color = if can_unlock && state.energy >= price {
            ETERNAL_COLOR
        } else if lvl >= 1 {
            Color::WHITE
        } else if can_unlock {
            Color::GRAY
        } else {
            Color::BLACK
        };

        let bounds = Rect::new(pos, Vec2::splat(16.0 * scale));
        if bounds.contains(mouse_pos) {
            if lvl == 0 {
                color = Color::GRAY;
            }

            tooltip = Some((b, pos));
        }

        draw.image(&state.res.blessings[i])
            .scale(Vec2::splat(scale))
            .translate(pos)
            .alpha(alpha)
            .color(color);
    });

    Blessing::iter().enumerate().for_each(|(i, b)| {
        let grid = vec2((i % grid_size) as f32, (i / grid_size) as f32);
        let pos = offset + padding * grid;

        let lvl = state.blessings.level(&b);
        if lvl != 0 {
            draw.text(&lvl.to_string())
                .size(14.0)
                .color(ETERNAL_COLOR)
                .translate(pos + 32.0)
                .anchor(Vec2::splat(0.5));
        }
    });

    draw.text(&state.energy.to_string())
        .anchor(vec2(1.0, 0.5))
        .translate(vec2(win_size.x - 60.0, 20.0 + 16.0))
        .size(16.0);

    draw.text(&format!("{:.0}%", state.good_progress * 100.0))
        .anchor(vec2(1.0, 0.5))
        .translate(vec2(xx - 6.0, 20.0 + 16.0))
        .size(12.0);

    draw.text(&format!("{:.0}%", state.bad_progress * 100.0))
        .anchor(vec2(0.0, 0.5))
        .translate(vec2(xx + t_size.x * scale + 6.0, 20.0 + 16.0))
        .size(12.0);

    draw.text(&format!(
        "Next wave: {:.1}s ({} souls)",
        state.spawn_timer - state.params.slow_spawn_time,
        state.spawn_num - state.params.block_spawn_souls
    ))
    .anchor(vec2(0.5, 0.0))
    .h_align_center()
    .translate(vec2(win_size.x * 0.5, 60.0))
    .size(8.0);

    if let Some((b, pos)) = tooltip {
        let size = vec2(250.0, 200.0);
        let pos = pos + 16.0;
        draw.rect(pos, vec2(250.0, 200.0))
            .alpha(0.9)
            .fill_color(Color::BLACK)
            .fill()
            .stroke_color(Color::GRAY)
            .stroke(4.0);

        let (name, desc) = b.info();
        let lvl = state.blessings.level(&b);
        let max_lvl = b.levels();
        let price = if lvl < max_lvl {
            Some(b.price(lvl))
        } else {
            None
        };

        draw.text(name)
            .color(ETERNAL_COLOR)
            .h_align_center()
            .anchor(vec2(0.5, 0.0))
            .translate(pos + vec2(size.x * 0.5, 16.0))
            .size(12.0);

        draw.text(desc)
            .h_align_center()
            .anchor(vec2(0.5, 0.0))
            .translate(pos + vec2(size.x * 0.5, 40.0))
            .max_width(size.x * 0.95)
            .h_align_center()
            .size(11.0);

        let last = draw.last_text_bounds();
        if let Some(price) = price {
            let base_pos = pos + vec2(20.0, 60.0 + last.height());
            draw.text("Requires: ")
                .color(Color::GRAY)
                .translate(base_pos)
                .size(10.0);

            let img_pos = base_pos + Vec2::Y * 20.0;
            draw.image(&state.res.souls_icon).position(img_pos);

            draw.text(&price.to_string())
                .translate(img_pos + vec2(20.0, 8.0))
                .anchor(vec2(0.0, 0.5))
                .size(9.0);

            if let Some((lvl, rb)) = b.require() {
                let idx = rb as usize;
                let tex = &state.res.blessings[idx];
                let img_pos = img_pos + Vec2::Y * 20.0;
                draw.image(tex).position(img_pos);

                draw.text(&lvl.to_string())
                    .translate(img_pos + vec2(20.0, 8.0))
                    .anchor(vec2(0.0, 0.5))
                    .size(9.0);
            }
        }

        // if click ask for upgrade
        if is_mouse_btn_pressed(MouseButton::Left) {
            state.unlock_blessing(b);
        }
    }

    #[cfg(debug_assertions)]
    {
        draw.text(&format!(
            "FPS: {:.0}, ms: {:.0}",
            time::fps(),
            time::delta_f32() * 1000.0
        ))
        .position(vec2(10.0, window_height() - 20.0))
        .size(6.0);
    }

    gfx::render_to_frame(&draw).unwrap();
}

pub fn lerp_color(c1: Color, c2: Color, t: f32) -> Color {
    c1 + (c2 - c1) * t
}

pub fn draw_menu(state: &mut State) {
    let mut draw = create_draw_2d();
    draw.text("Karma Keepers")
        .anchor(vec2(0.5, 0.0))
        .translate(vec2(window_width() * 0.5, 50.0))
        .size(40.0);

    draw.text("Guide the tiny souls, balance the forces of light and shadow, and preserve the fragile harmony of the spiritual realm.")
        .anchor(vec2(0.5, 0.0))
        .translate(vec2(window_width() * 0.5, 100.0))
        .h_align_center()
        .max_width(window_width() * 0.8)
        .color(Color::GRAY)
        .size(14.0);

    draw.text("Press ANY key to start")
        .anchor(Vec2::splat(0.5))
        .translate(window_size() * 0.5)
        .h_align_center()
        .max_width(window_width() * 0.8)
        .color(Color::WHITE)
        .size(20.0);

    draw.text("Your mision is to reach 100% influence (blue) before it drops to zero! Move the camera with WASD, and use the left mouse button to guide the good souls to follow you, turning other souls blue. Keep an eye on your spiritual energy to unlock blessings (top-left) for extra perks. Manage the balance between light and shadow, and don’t let your influence fade away!")
        .anchor(vec2(0.5, 1.0))
        .translate(vec2(window_width() * 0.5, window_height() - 50.0))
        .h_align_center()
        .max_width(window_width() * 0.6)
        .color(Color::GRAY)
        .size(12.0);

    draw.text("@Nazariglez")
        .anchor(vec2(1.0, 1.0))
        .position(window_size() - 20.0)
        .size(12.0)
        .color(ETERNAL_COLOR);

    gfx::render_to_frame(&draw).unwrap();

    if !keys_pressed().is_empty() {
        state.mode = Mode::Playing;
        init_spawn(state);
    }
}
