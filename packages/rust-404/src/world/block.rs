use rand::{distributions::Standard, prelude::Distribution};

use crate::{atlas::BlockTexture, render::mesh::Face};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BlockType {
    Grass,
    Stone,
    Dirt,
    Air,
}

impl Distribution<BlockType> for Standard {
    fn sample<R: rand::Rng + ?Sized>(&self, rng: &mut R) -> BlockType {
        match rng.gen_range(0..4) {
            0 => BlockType::Grass,
            1 => BlockType::Stone,
            2 => BlockType::Dirt,
            _ => BlockType::Air,
        }
    }
}

pub enum BlockTextures {
    Uniform(BlockTexture),
    SideTopBottom {
        side: BlockTexture,
        top: BlockTexture,
        bottom: BlockTexture,
    },
}

impl BlockTextures {
    pub fn for_face(&self, face: &Face) -> BlockTexture {
        match self {
            BlockTextures::Uniform(t) => t.clone(),
            BlockTextures::SideTopBottom { side, top, bottom } => match face {
                Face::NegativeY => bottom.clone(),
                Face::PositiveY => top.clone(),
                Face::NegativeX | Face::PositiveX | Face::NegativeZ | Face::PositiveZ => {
                    side.clone()
                }
            },
        }
    }
}

fn uniform(t: BlockTexture) -> Option<BlockTextures> {
    Some(BlockTextures::Uniform(t))
}
fn side_top_bottom(
    side: BlockTexture,
    top: BlockTexture,
    bottom: BlockTexture,
) -> Option<BlockTextures> {
    Some(BlockTextures::SideTopBottom { side, top, bottom })
}

impl BlockType {
    pub fn textures(&self) -> Option<BlockTextures> {
        use BlockTexture::*;
        match &*self {
            BlockType::Dirt => uniform(Dirt),
            BlockType::Air => None,
            BlockType::Grass => side_top_bottom(DirtGrass, GrassTop, Dirt),
            BlockType::Stone => uniform(Stone),
        }
    }
}

// #[derive(Debug, Clone, Copy)]
// pub struct Block {
//     pub block_type: BlockType,
//     pub opaque: bool,
// }

// impl Block {
//     pub const DEFAULT: Self = Block {
//         block_type: BlockType::Dirt,
//         opaque: true,
//     };

//     pub const DIRT: Block = Block {
//         block_type: BlockType::Dirt,
//         ..Self::DEFAULT
//     };
//     pub const AIR: Block = Block {
//         block_type: BlockType::Air,
//         opaque: false,
//         ..Self::DEFAULT
//     };
//     pub const GRASS: Block = Block {
//         block_type: BlockType::Grass,
//         ..Self::DEFAULT
//     };
//     pub const STONE: Block = Block {
//         block_type: BlockType::Stone,
//         ..Self::DEFAULT
//     };
// }
