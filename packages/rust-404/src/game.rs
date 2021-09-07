use std::sync::mpsc;
use std::sync::mpsc::Receiver;

use super::utils;
use crate::input::Button;
use crate::input::InputState;

use crate::input::Key;
use crate::render::camera::Camera;
use crate::render::camera::UP;
use crate::render::mesh::build_selection_ring;
use crate::render::mesh::Face;
use crate::render::Material;

use crate::render::mesh::Mesh;
use crate::render::ui;
use crate::render::ui::UiMaterial;
use crate::render::ui::UiRect;
use crate::render::Renderer;
use crate::world::block::BlockType;
use crate::world::chunk::Chunk;
use enum_iterator::IntoEnumIterator;
use glow::Texture;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::WebGl2RenderingContext;

#[wasm_bindgen]
pub struct Game {
    camera: Camera,
    input: InputState,
    renderer: Renderer,

    light_dir: glam::Vec3,
    types: Vec<BlockType>,
    active_type: usize,
    chunk: Chunk,
    last_picked: Option<(glam::Vec3, Face)>,
    mouse_rx: Receiver<Button>,
    mesh: Mesh,
    selection_ring: Mesh,
    crosshair: Texture,
}

#[wasm_bindgen]
impl Game {
    pub async fn new() -> Self {
        utils::set_panic_hook();

        let window = web_sys::window().unwrap();
        let document = window.document().unwrap();
        let canvas = document.get_element_by_id("canvas").unwrap();
        let canvas: web_sys::HtmlCanvasElement =
            canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        let context = canvas
            .get_context("webgl2")
            .unwrap()
            .unwrap()
            .dyn_into::<WebGl2RenderingContext>()
            .unwrap();

        // TODO: This is only a valid function for the wasm32 target (catch that, w/o rust-analyzer sucking hard)
        let context = glow::Context::from_webgl2_context(context);

        let input = InputState::register(window.document().unwrap());
        let camera = Camera::new(&input);

        let renderer = Renderer::new(context)
            .await
            .expect("failed to create renderer");

        // let vertices = cube(glam::Vec3::splat(1.0));

        let chunk = Chunk::new();

        let mesh = renderer
            .create_mesh(&chunk.chunk_vertices())
            .expect("failed to create mesh");

        let selection_ring = renderer
            .create_mesh(&build_selection_ring())
            .expect("failed to create selection ring mesh");

        // let atlas = Atlas::new(&renderer).await.expect("failed to create atlas");

        let (tx, rx) = mpsc::channel::<Button>();
        input.add_mouse_down_cb(move |btn| {
            tx.send(btn).expect("failed to send mouse event");
        });

        let types = BlockType::into_enum_iter()
            .filter(|t| t.textures().is_some())
            .collect();

        let crosshair = renderer
            .load_texture("crosshair.png")
            .await
            .expect("failed to create crosshair texture");

        Self {
            input,
            camera,
            renderer,
            mouse_rx: rx,

            light_dir: glam::Vec3::ZERO,
            types,
            active_type: 0,
            chunk,
            last_picked: None,
            mesh,
            selection_ring,
            crosshair,
        }
    }

    pub fn update(&mut self, dt: f32, total: f32) {
        // log!("Got dt: {}", dt);
        self.camera.update(dt, &self.input);

        // Update sun position
        let axis = UP;
        self.light_dir = (glam::Mat3::from_axis_angle(axis, total / 10.0) * glam::Vec3::X
            + glam::vec3(0.0, 1.0, 0.0))
        .normalize();

        if self.input.is_pressed(&Key::R) {
            self.active_type = (self.active_type + 1) % self.types.len();
            log!(
                "Changed active block to: {:?}",
                self.types.get(self.active_type)
            );
        }

        // And maybe place a block
        if let Ok(btn) = self.mouse_rx.try_recv() {
            if let Some((pos, face)) = self.last_picked.as_ref() {
                let recompute = match btn {
                    Button::Primary => {
                        // Set the currently selected block to be air
                        self.chunk
                            .set(pos.as_ivec3(), BlockType::Air)
                            .expect("failed to set air");
                        true
                    }
                    Button::Secondary => {
                        // Add a block in the direction of the face
                        let pos = pos.as_ivec3() + face.neighbor_dir();
                        let block_type = *self
                            .types
                            .get(self.active_type)
                            .unwrap_or(&BlockType::Stone);
                        self.chunk.set(pos, block_type).is_ok()
                    }
                    _ => false,
                };
                if recompute {
                    self.mesh = self
                        .renderer
                        .create_mesh(&self.chunk.chunk_vertices())
                        .expect("failed to create mesh");
                }
            }
        }
    }

    pub fn render(&mut self) {
        let (mut task, mut frame) = self.renderer.start_frame();
        task.push(&self.mesh);

        // Pick with the chunks
        let picked = self.renderer.pick(&task, &self.camera);
        if let Some((focused, face)) = &picked {
            // -> if we currently pick a block, add a selection ring
            let normal = face.normal();
            let transform = glam::Mat4::from_translation(focused.clone() + 0.5 * normal);
            let transform =
                transform * glam::Mat4::from_axis_angle(UP.cross(normal), UP.angle_between(normal));
            task.push_with_transform_and_material(
                &self.selection_ring,
                transform,
                Material::Solid(glam::vec4(0.7, 0.7, 0.7, 1.0)),
            )
        }
        self.last_picked = picked;

        // Draw some ui at the end
        frame.rect(
            UiRect::from_coords(290, 190, 20, 20),
            UiMaterial::Sprite(self.crosshair),
        );

        ui::inventory(
            &mut frame,
            &self.types,
            &self.active_type,
            &self.renderer.get_atlas(),
        );

        self.renderer
            .render(task, frame, &self.camera, &self.light_dir);
    }
}
