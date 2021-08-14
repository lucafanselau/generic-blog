use std::rc::Rc;

use anyhow::anyhow;
use bytemuck::*;
use wasm_bindgen::{prelude::Closure, JsCast, JsValue};
use web_sys::{
    HtmlCanvasElement, HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlProgram,
    WebGlShader, WebGlTexture, WebGlVertexArrayObject,
};

use crate::atlas::Atlas;

use self::{camera::Camera, mesh::Mesh};

pub mod camera;
pub mod mesh;

const VERTEX_CODE: &'static str = include_str!("solid.vert");
const FRAGMENT_CODE: &'static str = include_str!("solid.frag");

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    pos: glam::Vec3,
    normal: glam::Vec3,
    tex_coord: glam::Vec2,
}

const FLOAT_SIZE: i32 = std::mem::size_of::<f32>() as i32;

pub struct RenderTask<'a> {
    meshes: Vec<&'a Mesh>,
}

impl<'a> RenderTask<'a> {
    pub fn push(&mut self, mesh: &'a Mesh) {
        self.meshes.push(mesh);
    }
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

        context.enable(WebGl2RenderingContext::DEPTH_TEST);

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

        // Setup vao with vertex attribute data
        self.context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            8 * FLOAT_SIZE,
            0,
        );
        self.context.vertex_attrib_pointer_with_i32(
            1,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            8 * FLOAT_SIZE,
            3 * FLOAT_SIZE,
        );
        self.context.vertex_attrib_pointer_with_i32(
            2,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            8 * FLOAT_SIZE,
            6 * FLOAT_SIZE,
        );
        let position_attribute_location =
            self.context.get_attrib_location(&self.program, "position");

        let normal_attribute_location = self.context.get_attrib_location(&self.program, "norm");
        let tex_coord_attribute_location =
            self.context.get_attrib_location(&self.program, "tex_coord");

        self.context
            .enable_vertex_attrib_array(position_attribute_location as u32);
        self.context
            .enable_vertex_attrib_array(normal_attribute_location as u32);
        self.context
            .enable_vertex_attrib_array(tex_coord_attribute_location as u32);

        Ok(Mesh::new(vao, buffer, vertices.len() as i32))
    }

    pub fn destroy_mesh(&self, mesh: Mesh) {
        let Mesh { buffer, vao, .. } = mesh;
        self.context.delete_vertex_array(Some(&vao));
        self.context.delete_buffer(Some(&buffer));
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

    fn onload(
        gl: WebGl2RenderingContext,
        img: Rc<HtmlImageElement>,
        img_src: &'static str,
    ) -> anyhow::Result<WebGlTexture> {
        let texture = gl
            .create_texture()
            .ok_or(anyhow!("Failed to create a webgl texture"))?;

        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            WebGl2RenderingContext::TEXTURE_2D,
            0,
            WebGl2RenderingContext::RGBA as i32,
            WebGl2RenderingContext::RGBA,
            WebGl2RenderingContext::UNSIGNED_BYTE,
            &img,
        )
        .map_err(|e| {
            anyhow!(
                "failed to upload image data for image: {}: {:?}",
                img_src,
                e
            )
        })?;

        gl.generate_mipmap(WebGl2RenderingContext::TEXTURE_2D);

        Ok(texture)
    }

    pub async fn load_texture(&self, img_src: &'static str) -> anyhow::Result<WebGlTexture> {
        let (sender, receiver) =
            futures::channel::oneshot::channel::<anyhow::Result<WebGlTexture>>();

        let img = Rc::new(
            HtmlImageElement::new()
                .map_err(|e| anyhow!("failed to create image element {:?}", e))?,
        );

        let closure = {
            let img = img.clone();
            let gl: WebGl2RenderingContext = self.context.clone();

            Closure::once(Box::new(move || {
                sender
                    .send(Self::onload(gl, img, img_src))
                    .expect("failed to send");
            }) as Box<dyn FnOnce()>)
        };

        img.set_onload(Some(closure.as_ref().unchecked_ref()));
        img.set_src(img_src);

        let texture = receiver.await??;

        // Now it should also be safe to drop the closure, since the image has been loaded
        Ok(texture)
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

    pub fn task<'a>(&self) -> RenderTask<'a> {
        RenderTask {
            meshes: Default::default(),
        }
    }

    pub fn render<'a>(&self, task: RenderTask<'a>, camera: &Camera, atlas: &Atlas) {
        self.context.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        self.context.use_program(Some(&self.program));

        let loc = self
            .context
            .get_uniform_location(&self.program, "view_projection");
        // let elapsed = (self.sample_time() - state.startup) / 1000.0f64;
        let view_projection = camera.to_matrix().to_cols_array();
        self.context
            .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &view_projection);

        self.context
            .active_texture(WebGl2RenderingContext::TEXTURE0);

        self.context
            .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&atlas.texture));

        for mesh in task.meshes.iter() {
            self.context.bind_vertex_array(Some(&mesh.vao));

            self.context
                .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, mesh.vertices_count)
        }
    }
}
