use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

use super::utils;
use crate::atlas::Atlas;
use crate::input::InputState;
use crate::input::Key;
use crate::render::camera::Camera;
use crate::render::mesh::cube;
use crate::render::mesh::Mesh;
use crate::render::Renderer;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::EventTarget;
use web_sys::KeyboardEvent;
use web_sys::Window;
use web_sys::{HtmlCanvasElement, Performance, WebGl2RenderingContext, WebGlProgram, WebGlShader};

struct GameState {
    camera: Camera,
    input: InputState,
}

#[wasm_bindgen]
pub struct Game {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,
    window: Window,

    camera: Camera,
    input: InputState,
    renderer: Renderer,

    mesh: Mesh,
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

        let input = InputState::register(window.document().unwrap(), &canvas);
        let camera = Camera::new(&input);

        let renderer = Renderer::new(context.clone()).expect("failed to create renderer");

        let vertices = cube(glam::Vec3::splat(1.0));

        let mesh = renderer
            .create_mesh(&vertices)
            .expect("failed to create mesh");

        let atlas = Atlas::new(&renderer).await.expect("failed to create atlas");

        Self {
            canvas,
            context,
            window,
            input,
            camera,
            renderer,
            mesh,
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
        self.renderer.render(task, &self.camera, &self.atlas);
    }
}
