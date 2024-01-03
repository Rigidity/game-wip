use bevy::prelude::*;
use bevy_xpbd_3d::components::Collider;
use itertools::Itertools;

use crate::{
    block::{render_cube, BlockFaces},
    mesh_builder::MeshBuilder,
    voxel::{
        chunk::{iter_blocks, Chunk, CHUNK_SIZE},
        chunk_data::ChunkData,
        chunk_index::ChunkIndex,
    },
};

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
                        let is_block = left
                            .read()
                            .block(ChunkIndex::new(CHUNK_SIZE - 1, y, z))
                            .is_some();
                        values[AdjacentEdges::index(y, z)] = is_block;
                    }
                }
                values
            },
            right: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(right) = self.right {
                    for (y, z) in positions.clone() {
                        let is_block = right.read().block(ChunkIndex::new(0, y, z)).is_some();
                        values[AdjacentEdges::index(y, z)] = is_block;
                    }
                }
                values
            },
            top: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(top) = self.top {
                    for (x, z) in positions.clone() {
                        let is_block = top.read().block(ChunkIndex::new(x, 0, z)).is_some();
                        values[AdjacentEdges::index(x, z)] = is_block;
                    }
                }
                values
            },
            bottom: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(bottom) = self.bottom {
                    for (x, z) in positions.clone() {
                        let is_block = bottom
                            .read()
                            .block(ChunkIndex::new(x, CHUNK_SIZE - 1, z))
                            .is_some();
                        values[AdjacentEdges::index(x, z)] = is_block;
                    }
                }
                values
            },
            front: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(front) = self.front {
                    for (x, y) in positions.clone() {
                        let is_block = front.read().block(ChunkIndex::new(x, y, 0)).is_some();
                        values[AdjacentEdges::index(x, y)] = is_block;
                    }
                }
                values
            },
            back: {
                let mut values = [true; CHUNK_SIZE * CHUNK_SIZE];
                if let Some(back) = self.back {
                    for (x, y) in positions.clone() {
                        let is_block = back
                            .read()
                            .block(ChunkIndex::new(x, y, CHUNK_SIZE - 1))
                            .is_some();
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
                chunk.block(ChunkIndex::new(x - 1, y, z)).is_none()
            },
            right: if x == CHUNK_SIZE - 1 {
                !self.right[Self::index(y, z)]
            } else {
                chunk.block(ChunkIndex::new(x + 1, y, z)).is_none()
            },
            top: if y == CHUNK_SIZE - 1 {
                !self.top[Self::index(x, z)]
            } else {
                chunk.block(ChunkIndex::new(x, y + 1, z)).is_none()
            },
            bottom: if y == 0 {
                !self.bottom[Self::index(x, z)]
            } else {
                chunk.block(ChunkIndex::new(x, y - 1, z)).is_none()
            },
            front: if z == CHUNK_SIZE - 1 {
                !self.front[Self::index(x, y)]
            } else {
                chunk.block(ChunkIndex::new(x, y, z + 1)).is_none()
            },
            back: if z == 0 {
                !self.back[Self::index(x, y)]
            } else {
                chunk.block(ChunkIndex::new(x, y, z - 1)).is_none()
            },
        }
    }

    fn index(a: usize, b: usize) -> usize {
        a + b * CHUNK_SIZE
    }
}

pub async fn generate_mesh(chunk: Chunk, adjacent: AdjacentChunks) -> (Mesh, Option<Collider>) {
    let mut mesh_builder = MeshBuilder::new();
    let edges = adjacent.compute_edges();
    let chunk = chunk.read();

    for (x, y, z) in iter_blocks() {
        let Some(block) = chunk.block(ChunkIndex::new(x, y, z)) else {
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
    let mesh = mesh_builder.build();

    let collider = if mesh.count_vertices() > 0 {
        Collider::trimesh_from_mesh(&mesh)
    } else {
        None
    };

    (mesh, collider)
}
