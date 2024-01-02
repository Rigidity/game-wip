use itertools::Itertools;

pub const CHUNK_SIZE: usize = 32;

pub fn iter_blocks() -> impl Iterator<Item = (usize, usize, usize)> {
    (0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .cartesian_product(0..CHUNK_SIZE)
        .map(|((x, y), z)| (x, y, z))
}
