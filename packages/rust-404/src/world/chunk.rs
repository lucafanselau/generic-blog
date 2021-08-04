use super::block::Block;

const CHUNK_SIZE: usize = 32;

pub struct Chunk {
    blocks: [[[Block; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
}

impl Chunk {
    pub fn new() -> Self {
        Chunk {
            blocks: [[[Block::STONE; CHUNK_SIZE]; CHUNK_SIZE]; CHUNK_SIZE],
        }
    }
}
