#[derive(Debug, Clone, Copy)]
enum BlockType {
    Grass,
    Stone,
}

#[derive(Debug, Clone, Copy)]
pub struct Block {
    block_type: BlockType,
}

impl Block {
    pub const STONE: Block = Block {
        block_type: BlockType::Stone,
    };
}
