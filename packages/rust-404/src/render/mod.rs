use std::rc::Rc;

use __core::cmp::Ordering;
use anyhow::anyhow;
use bytemuck::*;
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::{
    HtmlImageElement, WebGl2RenderingContext, WebGlBuffer, WebGlFramebuffer, WebGlProgram,
    WebGlShader, WebGlTexture, WebGlVertexArrayObject,
};

use crate::{atlas::Atlas, world::chunk::CHUNK_SIZE};

use self::{
    camera::{Camera, UP},
    mesh::{Face, Mesh},
};

pub mod camera;
pub mod mesh;

const VERTEX_CODE: &'static str = include_str!("shaders/solid.vert");
const FRAGMENT_CODE: &'static str = include_str!("shaders/solid.frag");

const PICKING_VERTEX_CODE: &'static str = include_str!("shaders/picking.vert");
const PICKING_FRAGMENT_CODE: &'static str = include_str!("shaders/picking.frag");

#[derive(Debug, Clone, Copy, Zeroable, Pod)]
#[repr(C)]
pub struct Vertex {
    pos: glam::Vec3,
    normal: glam::Vec3,
    tex_coord: glam::Vec2,
    base_loc: glam::Vec3,
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
    picking_program: WebGlProgram,
    picking_fb: WebGlFramebuffer,
}

impl Renderer {
    pub fn new(context: WebGl2RenderingContext) -> anyhow::Result<Self> {
        let vert_shader =
            Self::compile_shader(&context, WebGl2RenderingContext::VERTEX_SHADER, VERTEX_CODE)?;

        let frag_shader = Self::compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            FRAGMENT_CODE,
        )?;

        let program = Self::link_program(&context, &vert_shader, &frag_shader)?;

        let picking_vert = Self::compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            PICKING_VERTEX_CODE,
        )?;

        let picking_frag = Self::compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            PICKING_FRAGMENT_CODE,
        )?;

        let picking_program = Self::link_program(&context, &picking_vert, &picking_frag)?;

        // create picking framebuffer
        let picking_fb = {
            let texture: WebGlTexture = context
                .create_texture()
                .ok_or(anyhow!("failed to create picking color attachment"))?;

            context.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));

            context.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_array_buffer_view(
                WebGl2RenderingContext::TEXTURE_2D,
                0,
                WebGl2RenderingContext::RGBA as i32,
                600,
                400,
                0,
                WebGl2RenderingContext::RGBA,
                WebGl2RenderingContext::UNSIGNED_BYTE,
                None,
            ).map_err(|e| anyhow!("failed to upload picking texture data: {:?}", e))?;

            let renderbuffer = context
                .create_renderbuffer()
                .ok_or(anyhow!("failed to create picking depth renderbuffer"))?;

            context.bind_renderbuffer(WebGl2RenderingContext::RENDERBUFFER, Some(&renderbuffer));

            context.renderbuffer_storage(
                WebGl2RenderingContext::RENDERBUFFER,
                WebGl2RenderingContext::DEPTH_COMPONENT16,
                600,
                400,
            );

            let fb = context
                .create_framebuffer()
                .ok_or(anyhow!("failed to create picking framebuffer"))?;

            context.bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&fb));

            context.framebuffer_texture_2d(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::COLOR_ATTACHMENT0,
                WebGl2RenderingContext::TEXTURE_2D,
                Some(&texture),
                0,
            );

            // make a depth buffer and the same size as the targetTexture
            context.framebuffer_renderbuffer(
                WebGl2RenderingContext::FRAMEBUFFER,
                WebGl2RenderingContext::DEPTH_ATTACHMENT,
                WebGl2RenderingContext::RENDERBUFFER,
                Some(&renderbuffer),
            );

            fb
        };

        context.enable(WebGl2RenderingContext::DEPTH_TEST);

        Ok(Self {
            context,
            program,
            picking_program,
            picking_fb,
        })
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

        let size = 11 * FLOAT_SIZE;

        // Setup vao with vertex attribute data
        self.context.vertex_attrib_pointer_with_i32(
            0,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            size,
            0,
        );
        self.context.vertex_attrib_pointer_with_i32(
            1,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            size,
            3 * FLOAT_SIZE,
        );
        self.context.vertex_attrib_pointer_with_i32(
            2,
            2,
            WebGl2RenderingContext::FLOAT,
            false,
            size,
            6 * FLOAT_SIZE,
        );

        self.context.vertex_attrib_pointer_with_i32(
            3,
            3,
            WebGl2RenderingContext::FLOAT,
            false,
            size,
            8 * FLOAT_SIZE,
        );

        (0..4u32).for_each(|i| self.context.enable_vertex_attrib_array(i));
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
    ) -> anyhow::Result<WebGlShader> {
        let shader = context
            .create_shader(shader_type)
            .ok_or(anyhow!("Unable to create shader object"))?;
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
                .map(|s| anyhow!("failed to compile shader: {}", s))
                .unwrap_or(anyhow!("Unknown error creating shader")))
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
    ) -> anyhow::Result<WebGlProgram> {
        let program = context
            .create_program()
            .ok_or(anyhow!("Unable to create shader object"))?;

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
                .map(|s| anyhow!("failed to link program: {}", s))
                .unwrap_or(anyhow!("Unknown error creating program object")))
        }
    }

    pub fn task<'a>(&self) -> RenderTask<'a> {
        RenderTask {
            meshes: Default::default(),
        }
    }

    pub fn render<'a>(
        &self,
        task: RenderTask<'a>,
        camera: &Camera,
        atlas: &Atlas,
        selection_ring: &Mesh,
    ) {
        let view_projection = camera.to_matrix().to_cols_array();

        // First pass (Picking framebuffer)
        let focused = {
            self.context
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, Some(&self.picking_fb));
            self.context.clear(
                WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
            );

            self.context.use_program(Some(&self.picking_program));

            let loc = self
                .context
                .get_uniform_location(&self.picking_program, "view_projection");
            self.context
                .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &view_projection);

            for mesh in task.meshes.iter() {
                self.context.bind_vertex_array(Some(&mesh.vao));

                self.context
                    .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, mesh.vertices_count)
            }

            // read back pixel (probably a bad time for that)
            // NOTE: Maybe do 2x2 area and like avg over that
            let mut data = [0u8; 4];
            self.context
                .read_pixels_with_u8_array_and_dst_offset(
                    300,
                    200,
                    1,
                    1,
                    WebGl2RenderingContext::RGBA,
                    WebGl2RenderingContext::UNSIGNED_BYTE,
                    &mut data,
                    0,
                )
                .expect("failed to read back data");

            // -> by now data should be the 4 pixel value
            // Do some simple calculation to figure out the coordinate
            if data[3] /*eg. alpha*/ != 0 {
                let loc = glam::UVec3::new(data[0] as _, data[1] as _, data[2] as _).as_f32();
                let loc = loc * ((CHUNK_SIZE as f32 - 1.0) / 255.0f32);

                // And also figure out the face
                Face::FACES
                    .iter()
                    .map(|f| (f, f.normal()))
                    // First of we only consider front-facing faces wrt. to camera direction, eg. with an angle between 90° and 180°
                    // since |normal| = |view_dir| = 1 => cos(angle) = normal.dot(view_dir)
                    // and for 90° <= angle <= 180° => -1 <= cos(angle) = normal.dot(view_dir) <= 0
                    .filter(|(_f, normal)| normal.dot(camera.dir) <= 0.0f32)
                    .filter_map(|(f, normal)| {
                        // Next we need to find the hit point of the face plane (d (point on plane), n (normal)) and the ray (o (camera pos), rd (camera dir))
                        // The plane is given by x.dot(n) = d.dot(n) and the ray by x = o + t * rd
                        // plugging that into the plane equation yields a result for t = ((d - o) ∙ n) / (rd ∙ n) (where ∙ denotes the dot product)
                        let d = loc + normal * 0.5f32;
                        let divisor = camera.dir.dot(normal);
                        // divisor == 0 indicates parallel dir -> eg. no collision or embedded (edge case does need to handle)
                        if divisor == 0.0f32 {
                            return None;
                        }
                        let t = (d - camera.pos).dot(normal) / divisor;

                        // We then obtain the hit point through x = o + t * rd
                        let x = camera.pos + t * camera.dir;
                        // log!("found hit-point {} for loc {} and face {:?}", x, loc, f);

                        // We can then obtain the u/v coordinates of the point in the plane trough the parametric form of the plane equation
                        // x = d + u ∙ e0 + v ∙ e1 (where e0 and e1 are the edge vectors of the quad)
                        let e0 = f.orthogonal();
                        let e1 = e0.cross(normal).normalize();
                        // Since e0 ∙ e1 = 0 (eg. orthogonal) and e0 ∙ e0 = e1 ∙ e1 = 1
                        // we can compute u = (x - d) * e0 and v = (x - d) * e1
                        let u = (x - d).dot(e0);
                        let v = (x - d).dot(e1);
                        // log!("[{:?}] u: {}, v: {}", f, u, v);
                        // Then the hit-point lies inside of the quad if u, v ∈ [-0.5;0.5]
                        (-0.5 <= u && u <= 0.5 && -0.5 <= v && v <= 0.5).then(|| (f, x))
                    })
                    .min_by(|(_, a), (_, b)| {
                        a.distance_squared(camera.pos)
                            .partial_cmp(&b.distance_squared(camera.pos))
                            .unwrap_or(Ordering::Equal)
                    })
                    .map(|(f, _)| (loc, f.clone()))
            } else {
                None
            }
        };

        // Second Pass (Main Pass)
        {
            self.context
                .bind_framebuffer(WebGl2RenderingContext::FRAMEBUFFER, None);

            self.context.clear(
                WebGl2RenderingContext::COLOR_BUFFER_BIT | WebGl2RenderingContext::DEPTH_BUFFER_BIT,
            );

            self.context.use_program(Some(&self.program));

            let loc = self
                .context
                .get_uniform_location(&self.program, "view_projection");
            self.context
                .uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &view_projection);

            let loc = self.context.get_uniform_location(&self.program, "model");
            self.context.uniform_matrix4fv_with_f32_array(
                loc.as_ref(),
                false,
                &glam::Mat4::IDENTITY.to_cols_array(),
            );

            self.context
                .active_texture(WebGl2RenderingContext::TEXTURE0);

            self.context
                .bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&atlas.texture));

            for mesh in task.meshes.iter() {
                self.context.bind_vertex_array(Some(&mesh.vao));

                self.context
                    .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, mesh.vertices_count)
            }

            if let Some((focused, face)) = focused {
                let normal = face.normal();
                let selection_model_matrix = glam::Mat4::from_translation(focused + 0.5 * normal);
                let selection_model_matrix = selection_model_matrix
                    * glam::Mat4::from_axis_angle(UP.cross(normal), UP.angle_between(normal));
                self.context.uniform_matrix4fv_with_f32_array(
                    loc.as_ref(),
                    false,
                    &selection_model_matrix.to_cols_array(),
                );

                self.context
                    .bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

                self.context.bind_vertex_array(Some(&selection_ring.vao));
                self.context.draw_arrays(
                    WebGl2RenderingContext::TRIANGLES,
                    0,
                    selection_ring.vertices_count,
                );
            }
        }
    }
}
