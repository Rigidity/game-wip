use bevy::{math::I64Vec3, prelude::*};
use derive_more::{Add, AddAssign};
use num_integer::div_floor;

use crate::chunk::CHUNK_SIZE;

use super::chunk_pos::ChunkPos;

#[derive(Component, Debug, Clone, Copy, Hash, Deref, PartialEq, Eq, Add, AddAssign)]
pub struct BlockPos(I64Vec3);

impl BlockPos {
    pub const RIGHT: BlockPos = BlockPos(I64Vec3::X);
    pub const TOP: BlockPos = BlockPos(I64Vec3::Y);
    pub const FRONT: BlockPos = BlockPos(I64Vec3::Z);
    pub const LEFT: BlockPos = BlockPos(I64Vec3::NEG_X);
    pub const BOTTOM: BlockPos = BlockPos(I64Vec3::NEG_Y);
    pub const BACK: BlockPos = BlockPos(I64Vec3::NEG_Z);

    pub fn new(pos: I64Vec3) -> Self {
        Self(pos)
    }

    pub fn chunk(self) -> ChunkPos {
        ChunkPos::new(IVec3::new(
            div_floor(self.x, CHUNK_SIZE as i64) as i32,
            div_floor(self.y, CHUNK_SIZE as i64) as i32,
            div_floor(self.z, CHUNK_SIZE as i64) as i32,
        ))
    }

    pub fn relative_to_chunk(self) -> (usize, usize, usize) {
        (
            self.0.x.rem_euclid(CHUNK_SIZE as i64) as usize,
            self.0.y.rem_euclid(CHUNK_SIZE as i64) as usize,
            self.0.z.rem_euclid(CHUNK_SIZE as i64) as usize,
        )
    }

    pub fn into_inner(self) -> I64Vec3 {
        self.0
    }
}
