use rkit::draw::create_draw_2d;
use rkit::gfx;
use rkit::gfx::Color;
use rkit::math::vec2;

fn main() -> Result<(), String> {
    rkit::init().on_update(update).run()
}

fn update(state: &mut ()) {
    let mut draw = create_draw_2d();
    draw.clear(Color::BLACK);
    gfx::render_to_frame(&draw).unwrap();
}
