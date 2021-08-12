use anyhow::anyhow;
use bytemuck::*;
use wasm_bindgen::JsValue;
use web_sys::{
    WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader, WebGlVertexArrayObject,
};

use self::mesh::Mesh;

pub mod camera;
mod mesh;

const VERTEX_CODE: &'static str = include_str!("solid.vert");
const FRAGMENT_CODE: &'static str = include_str!("solid.frag");

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    pos: glam::Vec3,
}

pub struct Renderer {
    context: WebGl2RenderingContext,
    program: WebGlProgram,
}

impl Renderer {
    pub fn new(context: WebGl2RenderingContext) -> Result<Self, JsValue> {
        let vert_shader =
            Self::compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_CODE)?;

        let frag_shader = Self::compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            FRAGMENT_CODE,
        )?;

        let program = Self::link_program(&context, &vert_shader, &frag_shader)?;

        Ok(Self { context, program })
    }

    pub fn create_mesh(&self, vertices: &[Vertex]) -> anyhow::Result<Mesh> {
        let data: &[u8] = cast_slice(vertices);

        // First create the vertex array buffer, so that the bind buffer
        // call gets recorded into the vertex array
        let vao: WebGlVertexArrayObject = self
            .context
            .create_vertex_array()
            .ok_or(anyhow!("failed to create vertex array"))?;

        self.context.bind_vertex_array(Some(&vao));

        let buffer: WebGlBuffer = self
            .context
            .create_buffer()
            .ok_or(anyhow!("failed to create buffer"))?;

        self.context
            .bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&buffer));

        // Then we can upload data to the buffer
        self.context.buffer_data_with_u8_array(
            WebGl2RenderingContext::ARRAY_BUFFER,
            data,
            WebGl2RenderingContext::STATIC_DRAW,
        );

        Ok(Mesh::new(vao, buffer))
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
}
