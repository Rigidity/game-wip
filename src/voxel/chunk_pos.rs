use std::ops;

use bevy::prelude::*;
use big_space::GridCell;
use indexmap::IndexSet;
use itertools::Itertools;

use super::{block_pos::BlockPos, chunk::CHUNK_SIZE};

#[derive(Component, Debug, Clone, Copy, Hash, PartialEq, Eq)]
pub struct ChunkPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

impl ChunkPos {
    pub const X: Self = Self::new(1, 0, 0);
    pub const Y: Self = Self::new(0, 1, 0);
    pub const Z: Self = Self::new(0, 0, 1);
    pub const NEG_X: Self = Self::new(-1, 0, 0);
    pub const NEG_Y: Self = Self::new(0, -1, 0);
    pub const NEG_Z: Self = Self::new(0, 0, -1);

    pub const fn new(x: i32, y: i32, z: i32) -> Self {
        Self { x, y, z }
    }

    pub fn block_pos(self) -> BlockPos {
        let scale = CHUNK_SIZE as i64;
        BlockPos::new(
            self.x as i64 * scale,
            self.y as i64 * scale,
            self.z as i64 * scale,
        )
    }

    pub fn chunks_within_radius(self, chunk_radius: i32) -> IndexSet<ChunkPos> {
        let radius_squared = chunk_radius * chunk_radius;
        let range = -chunk_radius..=chunk_radius;

        range
            .clone()
            .cartesian_product(range.clone())
            .cartesian_product(range)
            .map(|((x, y), z)| self + ChunkPos::new(x, y, z))
            .filter(|pos| self.distance_squared(*pos) <= radius_squared)
            .sorted_by(|a, b| a.distance_squared(self).cmp(&b.distance_squared(self)))
            .collect()
    }

    fn distance_squared(self, other: Self) -> i32 {
        let diff = self - other;
        (diff.x * diff.x) + (diff.y * diff.y) + (diff.z * diff.z)
    }

    pub fn left(self) -> ChunkPos {
        self + ChunkPos::NEG_X
    }

    pub fn right(self) -> ChunkPos {
        self + ChunkPos::X
    }

    pub fn top(self) -> ChunkPos {
        self + ChunkPos::Y
    }

    pub fn bottom(self) -> ChunkPos {
        self + ChunkPos::NEG_Y
    }

    pub fn front(self) -> ChunkPos {
        self + ChunkPos::Z
    }

    pub fn back(self) -> ChunkPos {
        self + ChunkPos::NEG_Z
    }

    pub fn adjacent(self) -> [ChunkPos; 6] {
        [
            self.left(),
            self.right(),
            self.top(),
            self.bottom(),
            self.front(),
            self.back(),
        ]
    }
}

impl From<GridCell<i32>> for ChunkPos {
    fn from(value: GridCell<i32>) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl From<ChunkPos> for GridCell<i32> {
    fn from(value: ChunkPos) -> Self {
        Self::new(value.x, value.y, value.z)
    }
}

impl ops::Add for ChunkPos {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        Self::new(self.x + rhs.x, self.y + rhs.y, self.z + rhs.z)
    }
}

impl ops::Sub for ChunkPos {
    type Output = Self;

    fn sub(self, rhs: Self) -> Self::Output {
        Self::new(self.x - rhs.x, self.y - rhs.y, self.z - rhs.z)
    }
}
