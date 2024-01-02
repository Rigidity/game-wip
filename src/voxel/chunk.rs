use std::sync::Arc;

use itertools::Itertools;
use parking_lot::RwLock;

use super::chunk_data::ChunkData;

pub const CHUNK_SIZE: usize = 32;

pub type Chunk = Arc<RwLock<ChunkData>>;

pub fn iter_blocks() -> impl Iterator<Item = (usize, usize, usize)> {
    (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .map(|((x, y), z)| (x, y, z))
}
