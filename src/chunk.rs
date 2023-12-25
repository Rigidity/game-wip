use std::sync::Arc;

use bevy::{math::ivec3, prelude::*, utils::HashSet};
use derive_more::Add;
use itertools::Itertools;
use num_traits::{FromPrimitive, ToPrimitive};
use parking_lot::RwLock;

use crate::{
    block::{render_cube, Block, BlockFaces},
    mesh_builder::MeshBuilder,
};

pub const CHUNK_SIZE: usize = 32;

pub fn iter_blocks() -> impl Iterator<Item = (usize, usize, usize)> {
    (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .map(|((x, y), z)| (x, y, z))
}

#[derive(Component, Debug, Clone, Copy, Hash, Deref, PartialEq, Eq, Add)]
pub struct ChunkPos(IVec3);

impl ChunkPos {
    pub const RIGHT: ChunkPos = ChunkPos(IVec3::X);
    pub const TOP: ChunkPos = ChunkPos(IVec3::Y);
    pub const FRONT: ChunkPos = ChunkPos(IVec3::Z);
    pub const LEFT: ChunkPos = ChunkPos(IVec3::NEG_X);
    pub const BOTTOM: ChunkPos = ChunkPos(IVec3::NEG_Y);
    pub const BACK: ChunkPos = ChunkPos(IVec3::NEG_Z);

    pub fn new(pos: IVec3) -> Self {
        Self(pos)
    }

    pub fn chunks_within_radius(self, chunk_radius: i32) -> HashSet<ChunkPos> {
        let radius_squared = chunk_radius * chunk_radius;
        let range = -chunk_radius..=chunk_radius;

        range
            .clone()
            .cartesian_product(range.clone())
            .cartesian_product(range)
            .map(|((x, y), z)| self + ChunkPos::new(ivec3(x, y, z)))
            .filter(|pos| self.0.distance_squared(pos.0) <= radius_squared)
            .sorted_by(|a, b| {
                a.0.distance_squared(self.0)
                    .cmp(&b.0.distance_squared(self.0))
            })
            .collect()
    }

    pub fn left(self) -> ChunkPos {
        self + ChunkPos::LEFT
    }

    pub fn right(self) -> ChunkPos {
        self + ChunkPos::RIGHT
    }

    pub fn top(self) -> ChunkPos {
        self + ChunkPos::TOP
    }

    pub fn bottom(self) -> ChunkPos {
        self + ChunkPos::BOTTOM
    }

    pub fn front(self) -> ChunkPos {
        self + ChunkPos::FRONT
    }

    pub fn back(self) -> ChunkPos {
        self + ChunkPos::BACK
    }

    pub fn adjacent_chunks(self) -> [ChunkPos; 6] {
        [
            self.left(),
            self.right(),
            self.top(),
            self.bottom(),
            self.front(),
            self.back(),
        ]
    }

    pub fn into_inner(self) -> IVec3 {
        self.0
    }
}

pub type Chunk = Arc<RwLock<ChunkData>>;

#[derive(Clone)]
pub struct ChunkData {
    blocks: Vec<Option<Block>>,
}

impl ChunkData {
    pub fn block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Option<Block> {
        &mut self.blocks[Self::index(x, y, z)]
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut last = None;
        let mut count = 0;

        for block in self.blocks.iter() {
            let block_type: u8 = block
                .and_then(|block| block.to_u8())
                .map(|id| id + 1)
                .unwrap_or_default();

            if last == Some(block_type) {
                count += 1;
            } else {
                if let Some(block_type) = last {
                    data.extend((count as u16).to_be_bytes());
                    data.push(block_type);
                }
                last = Some(block_type);
                count = 1;
            }
        }

        if let Some(block_type) = last {
            data.extend((count as u16).to_be_bytes());
            data.push(block_type);
        }

        data
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        let mut pos = 0;
        let mut chunk = Self {
            blocks: Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE),
        };

        while pos + 2 < data.len() {
            let count = u16::from_be_bytes([data[pos], data[pos + 1]]);
            pos += 2;

            for _ in 0..count {
                let byte = data[pos];
                if byte == 0 {
                    chunk.blocks.push(None);
                } else {
                    chunk.blocks.push(Some(Block::from_u8(byte - 1)?));
                }
            }

            pos += 1;
        }

        if pos == data.len() {
            Some(chunk)
        } else {
            None
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}

pub struct AdjacentChunks {
    pub left: Option<Chunk>,
    pub right: Option<Chunk>,
    pub top: Option<Chunk>,
    pub bottom: Option<Chunk>,
    pub front: Option<Chunk>,
    pub back: Option<Chunk>,
}

impl AdjacentChunks {
    pub fn compute_edges(self) -> AdjacentEdges {
        let positions = (0..CHUNK_SIZE).cartesian_product(0..CHUNK_SIZE);

        AdjacentEdges {
            left: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(left) = self.left {
                    for (y, z) in positions.clone() {
                        let is_block = left.read().block(CHUNK_SIZE - 1, y, z).is_some();
                        values[AdjacentEdges::index(y, z)] = is_block;
                    }
                }
                values
            },
            right: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(right) = self.right {
                    for (y, z) in positions.clone() {
                        let is_block = right.read().block(0, y, z).is_some();
                        values[AdjacentEdges::index(y, z)] = is_block;
                    }
                }
                values
            },
            top: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(top) = self.top {
                    for (x, z) in positions.clone() {
                        let is_block = top.read().block(x, 0, z).is_some();
                        values[AdjacentEdges::index(x, z)] = is_block;
                    }
                }
                values
            },
            bottom: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(bottom) = self.bottom {
                    for (x, z) in positions.clone() {
                        let is_block = bottom.read().block(x, CHUNK_SIZE - 1, z).is_some();
                        values[AdjacentEdges::index(x, z)] = is_block;
                    }
                }
                values
            },
            front: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(front) = self.front {
                    for (x, y) in positions.clone() {
                        let is_block = front.read().block(x, y, 0).is_some();
                        values[AdjacentEdges::index(x, y)] = is_block;
                    }
                }
                values
            },
            back: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(back) = self.back {
                    for (x, y) in positions.clone() {
                        let is_block = back.read().block(x, y, CHUNK_SIZE - 1).is_some();
                        values[AdjacentEdges::index(x, y)] = is_block;
                    }
                }
                values
            },
        }
    }
}

type AdjacentEdge = [bool; CHUNK_SIZE * CHUNK_SIZE];

pub struct AdjacentEdges {
    left: AdjacentEdge,
    right: AdjacentEdge,
    top: AdjacentEdge,
    bottom: AdjacentEdge,
    front: AdjacentEdge,
    back: AdjacentEdge,
}

impl AdjacentEdges {
    pub fn compute_faces(&self, chunk: &ChunkData, x: usize, y: usize, z: usize) -> BlockFaces {
        BlockFaces {
            left: if x == 0 {
                !self.left[Self::index(y, z)]
            } else {
                chunk.block(x - 1, y, z).is_none()
            },
            right: if x == CHUNK_SIZE - 1 {
                !self.right[Self::index(y, z)]
            } else {
                chunk.block(x + 1, y, z).is_none()
            },
            top: if y == CHUNK_SIZE - 1 {
                !self.top[Self::index(x, z)]
            } else {
                chunk.block(x, y + 1, z).is_none()
            },
            bottom: if y == 0 {
                !self.bottom[Self::index(x, z)]
            } else {
                chunk.block(x, y - 1, z).is_none()
            },
            front: if z == CHUNK_SIZE - 1 {
                !self.front[Self::index(x, y)]
            } else {
                chunk.block(x, y, z + 1).is_none()
            },
            back: if z == 0 {
                !self.back[Self::index(x, y)]
            } else {
                chunk.block(x, y, z - 1).is_none()
            },
        }
    }

    fn index(a: usize, b: usize) -> usize {
        a + b * CHUNK_SIZE
    }
}

pub async fn generate_mesh(chunk: Chunk, adjacent: AdjacentChunks) -> Mesh {
    let mut mesh_builder = MeshBuilder::new();
    let edges = adjacent.compute_edges();
    let chunk = chunk.read();

    for ((x, y), z) in (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
    {
        let Some(block) = chunk.block(x, y, z) else {
            continue;
        };

        let faces = edges.compute_faces(&chunk, x, y, z);
        render_cube(
            block,
            &mut mesh_builder,
            Vec3::new(x as f32, y as f32, z as f32),
            faces,
        );
    }

    drop(chunk);
    mesh_builder.build()
}
