use web_sys::{WebGlBuffer, WebGlVertexArrayObject};

pub struct Mesh {
    vao: WebGlVertexArrayObject,
    buffer: WebGlBuffer,
}

impl Mesh {
    pub fn new(vao: WebGlVertexArrayObject, buffer: WebGlBuffer) -> Self {
        Self { vao, buffer }
    }
}
