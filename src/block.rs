use bevy::{
    math::{vec3, Vec3},
    prelude::*,
};
use num_derive::{FromPrimitive, ToPrimitive};

use crate::mesh_builder::MeshBuilder;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ToPrimitive, FromPrimitive)]
#[repr(u8)]
pub enum Block {
    Dirt,
    Grass,
    Rock,
    Debug,
}

impl Block {
    fn face_index(self, face: BlockFace) -> u32 {
        match self {
            Self::Dirt => 0,
            Self::Grass if face == BlockFace::Top => 2,
            Self::Grass if face == BlockFace::Bottom => 0,
            Self::Grass => 1,
            Self::Rock => 3,
            Self::Debug => 3,
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
