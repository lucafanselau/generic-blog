use std::sync::Arc;
use std::sync::RwLock;
use std::time::Instant;

use super::utils;
use crate::input::InputState;
use crate::input::Key;
use crate::render::camera::Camera;
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

    state: Option<GameState>,
}

#[wasm_bindgen]
impl Game {
    pub fn new() -> Self {
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

        Self {
            canvas,
            context,
            window,
            state: None,
        }
    }

    pub fn init(&mut self) -> Result<(), JsValue> {
        self.context.use_program(Some(&program));

        let vertices: [f32; 9] = [-0.7, -0.7, 0.0, 0.7, -0.7, 0.0, 0.0, 0.4, 0.0];

        let position_attribute_location = self.context.get_attrib_location(&program, "position");
        let buffer = self
            .context
            .create_buffer()
            .ok_or("Failed to create buffer")?;
        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Note that `Float32Array::view` is somewhat dangerous (hence the
        // `unsafe`!). This is creating a raw view into our module's
        // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
        // (aka do a memory allocation in Rust) it'll cause the buffer to change,
        // causing the `Float32Array` to be invalid.
        //
        // As a result, after `Float32Array::view` we have to be very careful not to
        // do any memory allocations before it's dropped.
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&vertices);

            self.context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        let vao = self
            .context
            .create_vertex_array()
            .ok_or("Could not create vertex array object")?;
        self.context.bind_vertex_array(Some(&vao));

        self.context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        self.context
            .enable_vertex_attrib_array(position_attribute_location as u32);

        self.context.bind_vertex_array(Some(&vao));

        let input = InputState::register(self.window.document().unwrap(), &self.canvas);
        let camera = Camera::new(&input);

        self.state = Some(GameState {
            program,
            camera,
            input,
        });

        Ok(())
    }

    pub fn update(&mut self, dt: f32) {
        if let Some(ref mut state) = &mut self.state {
            // log!("Got dt: {}", dt);
            state.camera.update(dt, &state.input);
        }
    }

    pub fn render(&self) -> Result<(), JsValue> {
        if let Some(ref state) = self.state {
            self.context
                .clear_color(191.0 / 255.0, 219.0 / 255.0, 254.0 / 255.0, 1.0);
            self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            let loc = self
                .context
                .get_uniform_location(&state.program, "view_projection");
            // let elapsed = (self.sample_time() - state.startup) / 1000.0f64;
            let view_projection = state.camera.to_matrix().to_cols_array();
            self.context
                .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &view_projection);

            self.context
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
        }
        Ok(())
    }
}
