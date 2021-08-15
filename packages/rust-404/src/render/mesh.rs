use web_sys::{WebGlBuffer, WebGlVertexArrayObject};

use crate::atlas::Textures;

use super::camera::UP;
use super::Vertex;

pub struct Mesh {
    pub vao: WebGlVertexArrayObject,
    pub buffer: WebGlBuffer,
    pub vertices_count: i32,
}

impl Mesh {
    pub fn new(vao: WebGlVertexArrayObject, buffer: WebGlBuffer, vertices_count: i32) -> Self {
        Self {
            vao,
            buffer,
            vertices_count,
        }
    }
}

pub fn cube(scale: glam::Vec3) -> Vec<Vertex> {
    let scale = 0.5 * scale;

    let mut vertices = Vec::with_capacity(36);

    let mut calc_from_norm = |norm: glam::Vec3, orthogonal: glam::Vec3| {
        let base = norm * scale;
        let orthogonal = orthogonal * scale;

        let right = (base.cross(orthogonal)).normalize() * scale;

        let vec = |a: f32, b: f32| -> Vertex {
            let tex_coord = glam::vec2(a, b) * 0.5 + glam::vec2(0.5, 0.5);
            // let tex_coord = Textures::DIRT.base + tex_coord * Textures::DIRT.extend;

            Vertex {
                pos: base + (a * right + b * orthogonal),
                normal: norm,
                tex_coord,
            }
        };

        // First triangle
        vertices.push(vec(-1.0, 1.0));
        vertices.push(vec(-1.0, -1.0));
        vertices.push(vec(1.0, -1.0));

        // second triangle
        vertices.push(vec(-1.0, 1.0));
        vertices.push(vec(1.0, -1.0));
        vertices.push(vec(1.0, 1.0));
    };

    // X Normals
    calc_from_norm(glam::vec3(1.0, 0.0, 0.0), UP);
    calc_from_norm(glam::vec3(-1.0, 0.0, 0.0), UP);
    // Z Normals
    calc_from_norm(glam::vec3(0.0, 0.0, 1.0), UP);
    calc_from_norm(glam::vec3(0.0, 0.0, -1.0), UP);
    // Y Normals
    calc_from_norm(glam::vec3(0.0, 1.0, 0.0), glam::vec3(1.0, 0.0, 0.0));
    calc_from_norm(glam::vec3(0.0, -1.0, 0.0), glam::vec3(1.0, 0.0, 0.0));

    vertices
}
