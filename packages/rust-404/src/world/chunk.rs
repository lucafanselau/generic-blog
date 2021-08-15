use crate::{
    render::{
        mesh::{build_face, Face},
        Vertex,
    },
    world::block::BlockType,
};

use super::block::Block;

const CHUNK_SIZE: usize = 4;

pub struct Chunk {
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

fn gen_3d_range(from: i32, to: i32) -> impl Iterator<Item = glam::IVec3> {
    (from..to).flat_map(move |a| {
        (from..to).flat_map(move |b| (from..to).map(move |c| glam::ivec3(a, b, c)))
    })
}

impl Chunk {
    pub fn new() -> Self {
        let mut blocks = [[[Block::DIRT; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE];

        for pos in gen_3d_range(0, CHUNK_SIZE as i32) {
            if rand::random() {
                blocks[pos.x as usize][pos.y as usize][pos.z as usize] = Block::AIR;
            }
        }

        Chunk { blocks }
    }
    fn sample_vec(&self, vec: glam::IVec3) -> Option<Block> {
        if vec
            .to_array()
            .iter()
            .any(|c| *c < 0 || *c >= CHUNK_SIZE as i32)
        {
            None
        } else {
            Some(self.blocks[vec.x as usize][vec.y as usize][vec.z as usize])
        }
    }

    pub fn chunk_vertices(&self) -> Vec<Vertex> {
        let mut vertices = Vec::new();
        for pos in gen_3d_range(0, CHUNK_SIZE as i32) {
            if let Some(block) = self.sample_vec(pos) {
                if !block.is_solid() {
                    continue;
                }
                for face in Face::FACES.iter() {
                    let neighbor = pos + face.neighbord_dir();
                    match self.sample_vec(neighbor) {
                        Some(Block {
                            block_type: BlockType::Air,
                        })
                        | None => build_face(&mut vertices, face, &pos),
                        _ => (),
                    }
                }
            }
        }
        vertices
    }
}
