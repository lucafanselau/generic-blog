use std::time::Instant;

use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::Window;
use web_sys::{HtmlCanvasElement, Performance, WebGl2RenderingContext, WebGlProgram, WebGlShader};

struct GameState {
    program: WebGlProgram,
    startup: f64,
}

#[wasm_bindgen]
pub struct Game {
    canvas: HtmlCanvasElement,
    context: WebGl2RenderingContext,
    window: Window,

    state: Option<GameState>,
}

use super::utils;

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
        let vert_shader = Self::compile_shader(
            &self.context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r##"#version 300 es

        uniform vec4 offset;

        in vec4 position;

        void main() {

            gl_Position = offset + position;
        }
        "##,
        )?;

        let frag_shader = Self::compile_shader(
            &self.context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r##"#version 300 es

        precision highp float;
        out vec4 outColor;

        void main() {
            outColor = vec4(1, 1, 1, 1);
        }
        "##,
        )?;
        let program = Self::link_program(&self.context, &vert_shader, &frag_shader)?;
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

        self.state = Some(GameState {
            program,
            startup: self.sample_time(),
        });

        Ok(())
    }

    pub fn render(&self) -> Result<(), JsValue> {
        if let Some(ref state) = self.state {
            self.context.clear_color(0.5, 0.2, 0.8, 1.0);
            self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

            let loc = self.context.get_uniform_location(&state.program, "offset");
            let elapsed = (self.sample_time() - state.startup) / 1000.0f64;
            self.context
                .uniform4f(loc.as_ref(), elapsed.sin() as f32, 0.0, 0.0, 0.0);

            self.context
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 3);
        }
        Ok(())
    }

    fn compile_shader(
        context: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, String> {
        let shader = context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    fn link_program(
        context: &WebGl2RenderingContext,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = context
            .create_program()
            .ok_or_else(|| String::from("Unable to create shader object"))?;

        context.attach_shader(&program, vert_shader);
        context.attach_shader(&program, frag_shader);
        context.link_program(&program);

        if context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(context
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }

    fn sample_time(&self) -> f64 {
        self.window
            .performance()
            .map(|p| p.now())
            .expect("failed to sample time")
    }
}
