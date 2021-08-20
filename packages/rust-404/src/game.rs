use super::utils;
use crate::atlas::Atlas;
use crate::input::InputState;

use crate::render::camera::Camera;
use crate::render::mesh::build_selection_ring;

use crate::render::mesh::Mesh;
use crate::render::Renderer;
use crate::world::chunk::Chunk;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use web_sys::WebGl2RenderingContext;

#[wasm_bindgen]
pub struct Game {
    camera: Camera,
    input: InputState,
    renderer: Renderer,

    mesh: Mesh,
    selection_ring: Mesh,
    atlas: Atlas,
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

        let input = InputState::register(window.document().unwrap());
        let camera = Camera::new(&input);

        let renderer = Renderer::new(context.clone()).expect("failed to create renderer");

        // let vertices = cube(glam::Vec3::splat(1.0));

        let chunk = Chunk::new();

        let mesh = renderer
            .create_mesh(&chunk.chunk_vertices())
            .expect("failed to create mesh");

        let selection_ring = renderer
            .create_mesh(&build_selection_ring())
            .expect("failed to create selection ring mesh");

        let atlas = Atlas::new(&renderer).await.expect("failed to create atlas");

        Self {
            input,
            camera,
            renderer,
            mesh,
            selection_ring,
            atlas,
        }
    }

    pub fn update(&mut self, dt: f32) {
        // log!("Got dt: {}", dt);
        self.camera.update(dt, &self.input);
    }

    pub fn render(&self) {
        let mut task = self.renderer.task();
        task.push(&self.mesh);
        self.renderer
            .render(task, &self.camera, &self.atlas, &self.selection_ring);
    }
}
