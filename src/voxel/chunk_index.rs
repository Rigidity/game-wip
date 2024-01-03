use super::chunk::CHUNK_SIZE;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ChunkIndex(u16);

impl ChunkIndex {
    pub fn new(x: usize, y: usize, z: usize) -> Self {
        Self((x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE) as u16)
    }

    pub fn iter() -> impl Iterator<Item = Self> {
        let len = CHUNK_SIZE as u16;
        (0..len).map(Self)
    }

    pub(super) fn as_usize(self) -> usize {
        self.0 as usize
    }
}
