use std::ops;

use bevy::prelude::*;
use num_integer::div_floor;

use super::{chunk::CHUNK_SIZE, chunk_pos::ChunkPos};

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct BlockPos {
    pub x: i64,
    pub y: i64,
    pub z: i64,
}

impl BlockPos {
    pub const X: Self = Self::new(1, 0, 0);
    pub const Y: Self = Self::new(0, 1, 0);
    pub const Z: Self = Self::new(0, 0, 1);
    pub const NEG_X: Self = Self::new(-1, 0, 0);
    pub const NEG_Y: Self = Self::new(0, -1, 0);
    pub const NEG_Z: Self = Self::new(0, 0, -1);

    pub const fn new(x: i64, y: i64, z: i64) -> Self {
        Self { x, y, z }
    }

    pub fn chunk_pos(self) -> ChunkPos {
        ChunkPos::new(
            div_floor(self.x, CHUNK_SIZE as i64) as i32,
            div_floor(self.y, CHUNK_SIZE as i64) as i32,
            div_floor(self.z, CHUNK_SIZE as i64) as i32,
        )
    }

    pub fn relative_pos(self) -> (usize, usize, usize) {
        (
            self.x.rem_euclid(CHUNK_SIZE as i64) as usize,
            self.y.rem_euclid(CHUNK_SIZE as i64) as usize,
            self.z.rem_euclid(CHUNK_SIZE as i64) as usize,
        )
    }
}

impl ops::Add for BlockPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::AddAssign for BlockPos {
    fn add_assign(&mut self, rhs: Self) {
        self.x += rhs.x;
        self.y += rhs.y;
        self.z += rhs.z;
    }
}

impl ops::Sub for BlockPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
