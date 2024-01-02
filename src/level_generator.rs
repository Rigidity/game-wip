use bevy::math::I64Vec3;
use bevy::prelude::*;
use itertools::Itertools;
use noise::{NoiseFn, Perlin};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;

use crate::{
    block::Block,
    chunk::{ChunkData, ChunkPos, CHUNK_SIZE},
};

pub struct LevelGenerator {
    sea_level: f64,
    scale_factor: f64,
    elevation: NoiseMap,
}

impl LevelGenerator {
    pub fn new(seed: u64) -> Self {
        let mut rng = ChaCha8Rng::seed_from_u64(seed);

        Self {
            sea_level: 60.0,
            scale_factor: 250.0,
            elevation: NoiseMap {
                perlin: Perlin::new(rng.gen()),
                octaves: 4,
                amplitude: 15.0,
                frequency: 2.0,
                ..default()
            },
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
        for ((x, y), z) in (0..CHUNK_SIZE)
            .cartesian_product(0..CHUNK_SIZE)
            .cartesian_product(0..CHUNK_SIZE)
        {
            let block_pos = pos.into_inner().as_i64vec3() * CHUNK_SIZE as i64
                + I64Vec3::new(x as i64, y as i64, z as i64);
            let block = self.generate_block(block_pos);
            *chunk.block_mut(x, y, z) = block;
        }
        chunk
    }

    fn generate_block(&self, pos: I64Vec3) -> Option<Block> {
        let elevation = self.elevation.value_2d(
            pos.x as f64 / self.scale_factor,
            pos.z as f64 / self.scale_factor,
        );

        if pos.y as f64 <= self.sea_level + elevation - 1.0 {
            Some(Block::Dirt)
        } else if pos.y as f64 <= self.sea_level + elevation {
            Some(Block::Grass)
        } else {
            None
        }
    }
}

struct NoiseMap {
    perlin: Perlin,
    octaves: usize,
    amplitude: f64,
    frequency: f64,
    persistence: f64,
    lacunarity: f64,
}

impl Default for NoiseMap {
    fn default() -> Self {
        Self {
            perlin: Perlin::default(),
            octaves: 1,
            amplitude: 15.0,
            frequency: 2.0,
            persistence: 0.5,
            lacunarity: 2.0,
        }
    }
}

impl NoiseMap {
    fn value_2d(&self, x: f64, y: f64) -> f64 {
        let mut amplitude = self.amplitude;
        let mut frequency = self.frequency;
        let mut value = 0.0;
        for _ in 0..self.octaves {
            value += amplitude * self.perlin.get([x * frequency, y * frequency]);
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }
        value
    }

    fn value_3d(&self, x: f64, y: f64, z: f64) -> f64 {
        let mut amplitude = self.amplitude;
        let mut frequency = self.frequency;
        let mut value = 0.0;
        for _ in 0..self.octaves {
            value += amplitude
                * self
                    .perlin
                    .get([x * frequency, y * frequency, z * frequency]);
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }
        value
    }
}
