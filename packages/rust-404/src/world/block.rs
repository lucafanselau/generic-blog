#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Grass,
    Stone,
    Dirt,
    Air,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    pub block_type: BlockType,
}

impl Block {
    pub const STONE: Block = Block {
        block_type: BlockType::Stone,
    };

    pub const DIRT: Block = Block {
        block_type: BlockType::Dirt,
    };
    pub const AIR: Block = Block {
        block_type: BlockType::Air,
    };

    pub fn is_solid(&self) -> bool {
        self.block_type != BlockType::Air
    }
}
