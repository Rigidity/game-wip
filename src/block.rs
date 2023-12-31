use bevy::{
    math::{vec3, I64Vec3, Vec3},
    prelude::*,
};
use derive_more::{Add, AddAssign};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::{
    chunk::{ChunkPos, CHUNK_SIZE},
    mesh_builder::MeshBuilder,
};

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

fn div_floor(a: i64, b: i64) -> i64 {
    if a >= 0 || a % b == 0 {
        a / b
    } else {
        a / b - 1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Block {
    Dirt,
    Grass,
    Rock,
    Sand,
}

impl Block {
    fn face_index(self, face: BlockFace) -> u32 {
        match self {
            Self::Dirt => 0,
            Self::Grass if face == BlockFace::Top => 2,
            Self::Grass if face == BlockFace::Bottom => 0,
            Self::Grass => 1,
            Self::Rock => 3,
            Self::Sand => 4,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BlockFace {
    Left,
    Right,
    Top,
    Bottom,
    Front,
    Back,
}

pub struct BlockFaces {
    pub left: bool,
    pub right: bool,
    pub top: bool,
    pub bottom: bool,
    pub front: bool,
    pub back: bool,
}

pub fn render_cube(block: Block, chunk: &mut MeshBuilder, position: Vec3, faces: BlockFaces) {
    let x = position.x;
    let y = position.y;
    let z = position.z;

    let tex = Vec2::ZERO;
    let right = Vec2::X;
    let bottom = Vec2::Y;
    let br = right + bottom;

    // Left
    if faces.left {
        let idx = block.face_index(BlockFace::Left);
        let a = chunk.vertex(vec3(x, y, z), vec3(-1.0, 0.0, 0.0), tex + bottom, idx);
        let b = chunk.vertex(vec3(x, y + 1.0, z), vec3(-1.0, 0.0, 0.0), tex, idx);
        let c = chunk.vertex(
            vec3(x, y + 1.0, z + 1.0),
            vec3(-1.0, 0.0, 0.0),
            tex + right,
            idx,
        );
        let d = chunk.vertex(vec3(x, y, z + 1.0), vec3(-1.0, 0.0, 0.0), tex + br, idx);
        chunk.indices([a, d, c, c, b, a]);
    }

    // Right
    if faces.right {
        let idx = block.face_index(BlockFace::Right);
        let a = chunk.vertex(vec3(x + 1.0, y, z), vec3(1.0, 0.0, 0.0), tex + bottom, idx);
        let b = chunk.vertex(vec3(x + 1.0, y + 1.0, z), vec3(1.0, 0.0, 0.0), tex, idx);
        let c = chunk.vertex(
            vec3(x + 1.0, y + 1.0, z + 1.0),
            vec3(1.0, 0.0, 0.0),
            tex + right,
            idx,
        );
        let d = chunk.vertex(
            vec3(x + 1.0, y, z + 1.0),
            vec3(1.0, 0.0, 0.0),
            tex + br,
            idx,
        );
        chunk.indices([a, b, c, c, d, a]);
    }

    // Top
    if faces.top {
        let idx = block.face_index(BlockFace::Top);
        let a = chunk.vertex(vec3(x, y + 1.0, z), vec3(0.0, 1.0, 0.0), tex, idx);
        let b = chunk.vertex(
            vec3(x + 1.0, y + 1.0, z),
            vec3(0.0, 1.0, 0.0),
            tex + bottom,
            idx,
        );
        let c = chunk.vertex(
            vec3(x + 1.0, y + 1.0, z + 1.0),
            vec3(0.0, 1.0, 0.0),
            tex + br,
            idx,
        );
        let d = chunk.vertex(
            vec3(x, y + 1.0, z + 1.0),
            vec3(0.0, 1.0, 0.0),
            tex + right,
            idx,
        );
        chunk.indices([a, d, c, c, b, a]);
    }

    // Bottom
    if faces.bottom {
        let idx = block.face_index(BlockFace::Bottom);
        let a = chunk.vertex(vec3(x, y, z), vec3(0.0, -1.0, 0.0), tex, idx);
        let b = chunk.vertex(vec3(x + 1.0, y, z), vec3(0.0, -1.0, 0.0), tex + bottom, idx);
        let c = chunk.vertex(
            vec3(x + 1.0, y, z + 1.0),
            vec3(0.0, -1.0, 0.0),
            tex + br,
            idx,
        );
        let d = chunk.vertex(vec3(x, y, z + 1.0), vec3(0.0, -1.0, 0.0), tex + right, idx);
        chunk.indices([a, b, c, c, d, a]);
    }

    // Front
    if faces.front {
        let idx = block.face_index(BlockFace::Front);
        let a = chunk.vertex(vec3(x, y, z + 1.0), vec3(0.0, 0.0, 1.0), tex + bottom, idx);
        let b = chunk.vertex(
            vec3(x + 1.0, y, z + 1.0),
            vec3(0.0, 0.0, 1.0),
            tex + br,
            idx,
        );
        let c = chunk.vertex(
            vec3(x + 1.0, y + 1.0, z + 1.0),
            vec3(0.0, 0.0, 1.0),
            tex + right,
            idx,
        );
        let d = chunk.vertex(vec3(x, y + 1.0, z + 1.0), vec3(0.0, 0.0, 1.0), tex, idx);
        chunk.indices([a, b, c, c, d, a]);
    }

    // Back
    if faces.back {
        let idx = block.face_index(BlockFace::Back);
        let a = chunk.vertex(vec3(x, y, z), vec3(0.0, 0.0, -1.0), tex + bottom, idx);
        let b = chunk.vertex(vec3(x + 1.0, y, z), vec3(0.0, 0.0, -1.0), tex + br, idx);
        let c = chunk.vertex(
            vec3(x + 1.0, y + 1.0, z),
            vec3(0.0, 0.0, -1.0),
            tex + right,
            idx,
        );
        let d = chunk.vertex(vec3(x, y + 1.0, z), vec3(0.0, 0.0, -1.0), tex, idx);
        chunk.indices([a, d, c, c, b, a]);
    }
}
