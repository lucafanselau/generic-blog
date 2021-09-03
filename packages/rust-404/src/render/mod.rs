use std::{rc::Rc, sync::Arc};

use __core::cmp::Ordering;
use anyhow::{anyhow, bail, Context as AnyhowContext};
use bytemuck::*;
use enum_iterator::IntoEnumIterator;
use glow::{Context, Framebuffer, HasContext, PixelPackData, Program, Shader, Texture};
use wasm_bindgen::{prelude::Closure, JsCast};
use web_sys::HtmlImageElement;

use crate::{atlas::BlockTexture, world::chunk::CHUNK_SIZE};

use self::{
    camera::Camera,
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
    meshes: Vec<(&'a Mesh, Option<glam::Mat4>)>,
}

impl<'a> RenderTask<'a> {
    pub fn push(&mut self, mesh: &'a Mesh) {
        self.meshes.push((mesh, None));
    }
    pub fn push_with_transform(&mut self, mesh: &'a Mesh, transform: glam::Mat4) {
        self.meshes.push((mesh, Some(transform)))
    }
}

pub struct Renderer {
    context: Context,
    program: Program,
    picking_program: Program,
    picking_fb: Framebuffer,
    atlas: Texture,
}

impl Renderer {
    pub async fn new(context: Context) -> anyhow::Result<Self> {
        let program = unsafe {
            let vert_shader = Self::compile_shader(&context, glow::VERTEX_SHADER, VERTEX_CODE)?;
            let frag_shader = Self::compile_shader(&context, glow::FRAGMENT_SHADER, FRAGMENT_CODE)?;
            Self::link_program(&context, vert_shader, frag_shader)?
        };

        let picking_program = unsafe {
            let picking_vert =
                Self::compile_shader(&context, glow::VERTEX_SHADER, PICKING_VERTEX_CODE)?;
            let picking_frag =
                Self::compile_shader(&context, glow::FRAGMENT_SHADER, PICKING_FRAGMENT_CODE)?;
            Self::link_program(&context, picking_vert, picking_frag)?
        };

        // create picking framebuffer
        let picking_fb = unsafe {
            let texture = context
                .create_texture()
                .map_err(|e| anyhow!("failed to create picking color attachment: {}", e))?;

            context.bind_texture(glow::TEXTURE_2D, Some(texture));

            context.tex_image_2d(
                glow::TEXTURE_2D,
                0,
                glow::RGBA as i32,
                600,
                400,
                0,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                None,
            );

            let renderbuffer = context
                .create_renderbuffer()
                .map_err(|e| anyhow!("failed to create picking depth renderbuffer: {}", e))?;

            context.bind_renderbuffer(glow::RENDERBUFFER, Some(renderbuffer));

            context.renderbuffer_storage(glow::RENDERBUFFER, glow::DEPTH_COMPONENT16, 600, 400);

            let fb = context
                .create_framebuffer()
                .map_err(|e| anyhow!("failed to create picking framebuffer: {}", e))?;

            context.bind_framebuffer(glow::FRAMEBUFFER, Some(fb));

            context.framebuffer_texture_2d(
                glow::FRAMEBUFFER,
                glow::COLOR_ATTACHMENT0,
                glow::TEXTURE_2D,
                Some(texture),
                0,
            );

            // make a depth buffer and the same size as the targetTexture
            context.framebuffer_renderbuffer(
                glow::FRAMEBUFFER,
                glow::DEPTH_ATTACHMENT,
                glow::RENDERBUFFER,
                Some(renderbuffer),
            );

            fb
        };

        // create the texture atlas
        let atlas = unsafe {
            context
                .create_texture()
                .map_err(|e| anyhow!("Failed to create a atlas_texture: {}", e))?
        };

        // And upload image data to it
        unsafe {
            Self::load_image(&context, BlockTexture::SRC, |gl, img, img_src| {
                context.bind_texture(glow::TEXTURE_2D, Some(atlas));
                context.tex_image_2d_with_html_image(
                    glow::TEXTURE_2D,
                    0,
                    glow::RGBA as _,
                    glow::RGBA,
                    glow::UNSIGNED_BYTE,
                    &img,
                );
                context.tex_parameter_i32(
                    glow::TEXTURE_2D,
                    glow::TEXTURE_MIN_FILTER,
                    glow::LINEAR_MIPMAP_LINEAR as _,
                );
                context.generate_mipmap(glow::TEXTURE_2D);
                Ok(())
            })
            .await?;
        }

        unsafe {
            context.enable(glow::DEPTH_TEST);
        };

        Ok(Self {
            context,
            program,
            picking_program,
            picking_fb,
            atlas,
        })
    }

    pub fn create_mesh(&self, vertices: &[Vertex]) -> anyhow::Result<Mesh> {
        let data: &[u8] = cast_slice(vertices);

        unsafe {
            // First create the vertex array buffer, so that the bind buffer
            // call gets recorded into the vertex array
            let vao = self
                .context
                .create_vertex_array()
                .map_err(|e| anyhow!("failed to create vertex array: {}", e))?;

            self.context.bind_vertex_array(Some(vao));

            let buffer = self
                .context
                .create_buffer()
                .map_err(|e| anyhow!("failed to create buffer: {}", e))?;

            self.context.bind_buffer(glow::ARRAY_BUFFER, Some(buffer));

            // Then we can upload data to the buffer
            self.context
                .buffer_data_u8_slice(glow::ARRAY_BUFFER, data, glow::STATIC_DRAW);

            const attrib_data: [i32; 4] = [3, 3, 2, 3];
            let total_size = attrib_data.iter().sum::<i32>() * FLOAT_SIZE;
            let mut offset = 0;
            for (index, size) in attrib_data.iter().copied().enumerate() {
                self.context.vertex_attrib_pointer_f32(
                    index as u32,
                    size,
                    glow::FLOAT,
                    false,
                    total_size,
                    offset * FLOAT_SIZE,
                );

                offset += size;
            }

            attrib_data
                .iter()
                .enumerate()
                .for_each(|(i, _)| self.context.enable_vertex_attrib_array(i as u32));
            Ok(Mesh::new(vao, buffer, vertices.len() as i32))
        }
    }

    pub fn destroy_mesh(&self, mesh: Mesh) {
        let Mesh { buffer, vao, .. } = mesh;
        unsafe {
            self.context.delete_vertex_array(vao);
            self.context.delete_buffer(buffer);
        }
    }

    unsafe fn compile_shader(
        context: &Context,
        shader_type: u32,
        source: &str,
    ) -> anyhow::Result<Shader> {
        let shader = context
            .create_shader(shader_type)
            .map_err(|e| anyhow!("Unable to create shader object: {}", e))?;
        context.shader_source(shader, source);
        context.compile_shader(shader);

        if context.get_shader_compile_status(shader) {
            Ok(shader)
        } else {
            bail!(
                "failed to compile shader: {}",
                context.get_shader_info_log(shader)
            );
        }
    }

    // pub async fn load_texture(gl: &Context, t: Textures, texture: Texture) -> anyhow::Result<()> {
    //     Self::load_image(gl, t.src(), move |gl, img, img_src| unsafe {
    //         let level: u8 = t.into();
    //         gl.bind_texture(glow::TEXTURE_2D, Some(texture));
    //         gl.tex_image_3d(
    //             glow::TEXTURE_2D,
    //             level as i32,
    //             glow::RGBA as i32,
    //             glow::RGBA,
    //             glow::UNSIGNED_BYTE,
    //             &img,
    //         );
    //         gl.generate_mipmap(glow::TEXTURE_2D);
    //         Ok(())
    //     })
    //     .await
    // }

    pub async fn load_image<T: 'static>(
        context: &Context,
        img_src: &'static str,
        cb: impl FnOnce(&Context, HtmlImageElement, &'static str) -> anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        use futures::channel::oneshot::channel;
        let (sender, receiver) = channel::<()>();

        let img = HtmlImageElement::new()
            .map_err(|e| anyhow!("failed to create image element {:?}", e))?;

        let closure = {
            Closure::once(Box::new(move || {
                sender
                    .send(())
                    .expect("failed to send image completed method");
            }) as Box<dyn FnOnce()>)
        };

        img.set_onload(Some(closure.as_ref().unchecked_ref()));
        img.set_src(img_src);

        // wait for completion
        let _ = receiver.await?;

        cb(context, img, img_src)
    }

    // pub async fn load_texture(&self, img_src: &'static str) -> anyhow::Result<WebGlTexture> {
    //     let (sender, receiver) =
    //         futures::channel::oneshot::channel::<anyhow::Result<WebGlTexture>>();

    //     let img = Rc::new(
    //         HtmlImageElement::new()
    //             .map_err(|e| anyhow!("failed to create image element {:?}", e))?,
    //     );

    //     let closure = {
    //         let img = img.clone();
    //         let gl: WebGl2RenderingContext = self.context.clone();

    //         Closure::once(Box::new(move || {
    //             sender
    //                 .send(Self::onload(gl, img, img_src))
    //                 .expect("failed to send");
    //         }) as Box<dyn FnOnce()>)
    //     };

    //     img.set_onload(Some(closure.as_ref().unchecked_ref()));
    //     img.set_src(img_src);

    //     let texture = receiver.await??;

    //     // Now it should also be safe to drop the closure, since the image has been loaded
    //     Ok(texture)
    // }

    unsafe fn link_program(
        context: &Context,
        vert_shader: Shader,
        frag_shader: Shader,
    ) -> anyhow::Result<Program> {
        let program = context
            .create_program()
            .map_err(|e| anyhow!("Unable to create shader object: {}", e))?;

        context.attach_shader(program, vert_shader);
        context.attach_shader(program, frag_shader);
        context.link_program(program);

        if context.get_program_link_status(program) {
            Ok(program)
        } else {
            bail!(
                "failed to link program: {}",
                context.get_program_info_log(program)
            );
        }
    }

    pub fn task<'a>(&self) -> RenderTask<'a> {
        RenderTask {
            meshes: Default::default(),
        }
    }

    pub fn pick<'a>(&self, task: &RenderTask<'a>, camera: &Camera) -> Option<(glam::Vec3, Face)> {
        // The opengl part of the picking procedure is unsafe due to glow
        let data = unsafe {
            self.context
                .bind_framebuffer(glow::FRAMEBUFFER, Some(self.picking_fb));
            self.context
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            self.context.use_program(Some(self.picking_program));

            let loc = self
                .context
                .get_uniform_location(self.picking_program, "view_projection");
            self.context.uniform_matrix_4_f32_slice(
                loc.as_ref(),
                false,
                camera.projection_view.as_ref(),
            );

            for (mesh, _transform) in task.meshes.iter() {
                // TODO: Model matrix
                self.context.bind_vertex_array(Some(mesh.vao));

                self.context
                    .draw_arrays(glow::TRIANGLES, 0, mesh.vertices_count)
            }

            // read back pixel (probably a bad time for that)
            // NOTE: Maybe do 2x2 area and like avg over that
            let mut data = [0u8; 4];
            self.context.read_pixels(
                300,
                200,
                1,
                1,
                glow::RGBA,
                glow::UNSIGNED_BYTE,
                PixelPackData::Slice(&mut data),
            );
            data
        };

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
    }

    pub fn render<'a>(&self, task: &RenderTask<'a>, camera: &Camera) {
        unsafe {
            // Second Pass (Main Pass)
            self.context.bind_framebuffer(glow::FRAMEBUFFER, None);

            self.context
                .clear(glow::COLOR_BUFFER_BIT | glow::DEPTH_BUFFER_BIT);

            self.context.use_program(Some(self.program));

            let loc = self
                .context
                .get_uniform_location(self.program, "view_projection");
            self.context.uniform_matrix_4_f32_slice(
                loc.as_ref(),
                false,
                camera.projection_view.as_ref(),
            );

            {
                let loc = self.context.get_uniform_location(self.program, "light_pos");
                // TODO: Make that dynamic
                const SUN: glam::Vec3 = glam::const_vec3!([10.0, 10.0, 10.0]);
                self.context.uniform_3_f32_slice(loc.as_ref(), SUN.as_ref())
            }

            {
                let loc = self.context.get_uniform_location(self.program, "view_pos");
                self.context
                    .uniform_3_f32_slice(loc.as_ref(), camera.pos.as_ref())
            }

            let loc = self.context.get_uniform_location(self.program, "model");

            self.context.active_texture(glow::TEXTURE0);

            // TODO: Atlas
            self.context
                .bind_texture(glow::TEXTURE_2D, Some(self.atlas));

            for (mesh, transform) in task.meshes.iter() {
                self.context.bind_vertex_array(Some(mesh.vao));

                let model = match transform {
                    Some(transform) => transform.as_ref(),
                    None => glam::Mat4::IDENTITY.as_ref(),
                };
                self.context
                    .uniform_matrix_4_f32_slice(loc.as_ref(), false, model);

                self.context
                    .draw_arrays(glow::TRIANGLES, 0, mesh.vertices_count)
            }
        }
    }
}
