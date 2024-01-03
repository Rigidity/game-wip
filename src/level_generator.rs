use noise::{Fbm, NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use splines::{Interpolation, Key, Spline};

use crate::{
    block::Block,
    voxel::{
        block_pos::BlockPos, chunk::iter_blocks, chunk_data::ChunkData, chunk_index::ChunkIndex,
        chunk_pos::ChunkPos,
    },
};

pub struct LevelGenerator {
    temperature: Perlin,
    continentalness: Fbm<Perlin>,
    spline: Spline<f64, f64>,
}

impl LevelGenerator {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        let spline = Spline::from_vec(vec![
            Key::new(-1.0, 50.0, Interpolation::Linear),
            Key::new(0.3, 100.0, Interpolation::Linear),
            Key::new(1.0, 150.0, Interpolation::Linear),
        ]);

        let mut continentalness = Fbm::new(rng.gen());
        continentalness.octaves = 4;

        Self {
            temperature: Perlin::new(rng.gen()),
            continentalness,
            spline,
        }
    }
}

impl Default for LevelGenerator {
    fn default() -> Self {
        Self::new(0)
    }
}

impl LevelGenerator {
    pub fn generate_chunk(&self, pos: ChunkPos) -> ChunkData {
        let mut chunk = ChunkData::default();
        for (x, y, z) in iter_blocks() {
            let block_pos = pos.block_pos() + BlockPos::new(x as i64, y as i64, z as i64);
            let block = self.generate_block(block_pos);
            *chunk.block_mut(ChunkIndex::new(x, y, z)) = block;
        }
        chunk
    }

    fn generate_block(&self, pos: BlockPos) -> Option<Block> {
        let temperature = self
            .temperature
            .get([pos.x as f64 / 700.0, pos.z as f64 / 700.0]);

        let continentalness = self
            .continentalness
            .get([pos.x as f64 / 400.0, pos.z as f64 / 400.0]);

        let terrain_height = self.spline.clamped_sample(continentalness).unwrap();

        if temperature > 0.4 {
            if pos.y as f64 <= terrain_height {
                Some(Block::Sand)
            } else {
                None
            }
        } else if pos.y as f64 <= terrain_height - 1.0 {
            Some(Block::Dirt)
        } else if pos.y as f64 <= terrain_height {
            Some(Block::Grass)
        } else {
            None
        }
    }
}
