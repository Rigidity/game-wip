use num_traits::{FromPrimitive, ToPrimitive};

use crate::block::Block;

use super::chunk::CHUNK_SIZE;

#[derive(Clone)]
pub struct ChunkData {
    blocks: Vec<Option<Block>>,
}

impl ChunkData {
    pub fn block(&self, x: usize, y: usize, z: usize) -> Option<Block> {
        self.blocks[Self::index(x, y, z)]
    }

    pub fn block_mut(&mut self, x: usize, y: usize, z: usize) -> &mut Option<Block> {
        &mut self.blocks[Self::index(x, y, z)]
    }

    pub fn serialize(&self) -> Vec<u8> {
        let mut data = Vec::new();
        let mut last = None;
        let mut count = 0;

        for block in self.blocks.iter() {
            let block_type: u8 = block
                .and_then(|block| block.to_u8())
                .map(|id| id + 1)
                .unwrap_or_default();

            if last == Some(block_type) {
                count += 1;
            } else {
                if let Some(block_type) = last {
                    data.extend((count as u16).to_be_bytes());
                    data.push(block_type);
                }
                last = Some(block_type);
                count = 1;
            }
        }

        if let Some(block_type) = last {
            if data.is_empty() {
                data.push(block_type);
            } else {
                data.extend((count as u16).to_be_bytes());
                data.push(block_type);
            }
        }

        data
    }

    pub fn deserialize(data: &[u8]) -> Option<Self> {
        let mut pos = 0;
        let mut chunk = Self {
            blocks: Vec::with_capacity(CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE),
        };

        if data.len() == 1 {
            for _ in 0..chunk.blocks.capacity() {
                if data[0] == 0 {
                    chunk.blocks.push(None);
                } else {
                    chunk.blocks.push(Some(Block::from_u8(data[0] - 1)?));
                }
            }
            pos += 1;
        }

        while pos + 2 < data.len() {
            let count = u16::from_be_bytes([data[pos], data[pos + 1]]);
            pos += 2;

            for _ in 0..count {
                let byte = data[pos];
                if byte == 0 {
                    chunk.blocks.push(None);
                } else {
                    chunk.blocks.push(Some(Block::from_u8(byte - 1)?));
                }
            }

            pos += 1;
        }

        if pos == data.len() {
            Some(chunk)
        } else {
            None
        }
    }

    fn index(x: usize, y: usize, z: usize) -> usize {
        x + y * CHUNK_SIZE + z * CHUNK_SIZE * CHUNK_SIZE
    }
}

impl Default for ChunkData {
    fn default() -> Self {
        Self {
            blocks: vec![None; CHUNK_SIZE * CHUNK_SIZE * CHUNK_SIZE],
        }
    }
}
