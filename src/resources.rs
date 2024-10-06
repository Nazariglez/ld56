use rkit::draw;
use rkit::draw::Sprite;
use rkit::gfx::TextureFilter;
use rkit::math::{vec2, Rect, Vec2};

pub struct Resources {
    pub souls_icon: Sprite,
    pub blessings: [Sprite; 9],
    pub tile: Sprite,
    pub bar: Sprite,
    pub bar_outline: Sprite,
    pub neutral: Sprite,
    pub luminal: Sprite,
    pub shadow: Sprite,
    pub eternal: Sprite,
    pub shirt: Sprite,
    pub circle: Sprite,
}

impl Resources {
    pub fn new() -> Result<Self, String> {
        let base = draw::create_sprite()
            .from_image(include_bytes!("../assets/sprites.png"))
            .with_min_filter(TextureFilter::Nearest)
            .with_mag_filter(TextureFilter::Nearest)
            .build()?;

        let grid_size = Vec2::splat(16.0);
        let souls_icon = base.clone_with_frame(Rect::new(Vec2::ZERO, grid_size));
        let blessings = [
            base.clone_with_frame(Rect::new(grid_size * vec2(1.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(2.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(3.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(4.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(5.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(6.0, 0.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(0.0, 1.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(1.0, 1.0), grid_size)),
            base.clone_with_frame(Rect::new(grid_size * vec2(2.0, 1.0), grid_size)),
        ];
        let neutral = base.clone_with_frame(Rect::new(grid_size * vec2(3.0, 1.0), grid_size));
        let shirt = base.clone_with_frame(Rect::new(grid_size * vec2(4.0, 1.0), grid_size));
        let shadow = base.clone_with_frame(Rect::new(grid_size * vec2(5.0, 1.0), grid_size));
        let luminal = base.clone_with_frame(Rect::new(grid_size * vec2(6.0, 1.0), grid_size));
        let eternal = base.clone_with_frame(Rect::new(grid_size * vec2(7.0, 1.0), grid_size));
        let bar = base.clone_with_frame(Rect::new(
            grid_size * vec2(0.0, 2.0),
            grid_size * vec2(8.0, 1.0),
        ));
        let bar_outline = base.clone_with_frame(Rect::new(
            grid_size * vec2(0.0, 3.0),
            grid_size * vec2(8.0, 1.0),
        ));

        let tile = base.clone_with_frame(Rect::new(grid_size * vec2(0.0, 4.0), grid_size * 5.0));
        let circle = base.clone_with_frame(Rect::new(grid_size * vec2(5.0, 4.0), grid_size * 2.0));

        Ok(Self {
            souls_icon,
            blessings,
            tile,
            bar,
            bar_outline,
            neutral,
            luminal,
            shadow,
            eternal,
            shirt,
            circle,
        })
    }
}
