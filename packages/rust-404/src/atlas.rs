use web_sys::WebGlTexture;

use crate::render::Renderer;
#[non_exhaustive]
pub struct Textures {
    pub base: glam::Vec2,
    pub extend: glam::Vec2,
}
const E: f32 = 8.0 / 1152.0;

impl Textures {
    const fn entry(x: u32, y: u32, width: u32, height: u32) -> Textures {
        Self {
            base: glam::const_vec2!([x as f32 / 1152.0 + E, y as f32 / 1280.0 + E]),
            extend: glam::const_vec2!([width as f32 / 1152.0 - E, height as f32 / 1280.0 - E]),
        }
    }

    pub const DIRT: Self = Self::entry(896, 640, 128, 128);
}
pub struct Atlas {
    pub texture: WebGlTexture,
}

impl Atlas {
    pub async fn new(renderer: &Renderer) -> anyhow::Result<Self> {
        let texture = renderer.load_texture("dirt.png").await?;

        Ok(Self { texture })
    }
}
