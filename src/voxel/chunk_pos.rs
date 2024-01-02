use bevy::prelude::*;
use derive_more::{Add, AddAssign};
use indexmap::IndexSet;
use itertools::Itertools;

#[derive(Component, Debug, Clone, Copy, Hash, Deref, PartialEq, Eq, Add, AddAssign)]
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

    pub fn chunks_within_radius(self, chunk_radius: i32) -> IndexSet<ChunkPos> {
        let radius_squared = chunk_radius * chunk_radius;
        let range = -chunk_radius..=chunk_radius;

        range
            .clone()
            .cartesian_product(range.clone())
            .cartesian_product(range)
            .map(|((x, y), z)| self + ChunkPos::new(IVec3::new(x, y, z)))
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
