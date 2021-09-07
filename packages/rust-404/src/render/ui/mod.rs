pub mod element;
pub mod renderer;

pub use element::*;
use glow::Texture;
pub use renderer::*;

use crate::world::block::BlockType;

use super::{mesh::Face, Renderer};

pub fn inventory(
    frame: &mut UiFrame,
    types: &Vec<BlockType>,
    active_type: &usize,
    atlas: &Texture,
) {
    // 2 px outer padding
    // 2 px padding around sprite
    // 36px sprite

    let extend = glam::vec2((40 * 10 + 8) as f32, 48.0);
    let tl = glam::vec2(0.5, 1.0) * (glam::vec2(600.0, 400.0) - extend);

    // Draw background
    frame.rect(UiRect::new(tl, extend), UiMaterial::WHITE);
    let base = 100u32;

    for (i, t) in types.iter().take(10).enumerate() {
        // Maybe add active marker
        if *active_type == i {
            let rect = UiRect::from_coords((base - 2u32) + i as u32 * 40, 400 - 44, 40, 40);
            frame.rect(rect, UiMaterial::BLACK)
        }

        let rect = UiRect::from_coords(base + i as u32 * 40, 400 - 42, 36, 36);
        let texture = t
            .textures()
            .expect("failed to get texture for item")
            .for_face(&Face::PositiveX);
        let tex_coord = UiRect::new(texture.base(), texture.extend());
        frame.rect_with_tex(rect, tex_coord, UiMaterial::Sprite(*atlas))
    }
}
