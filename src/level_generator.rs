use bevy::math::I64Vec3;
use itertools::Itertools;
use noise::{NoiseFn, Perlin};

use crate::{
    block::Block,
    chunk::{ChunkData, ChunkPos, CHUNK_SIZE},
};

pub struct LevelGenerator {
    perlin: Perlin,
    surface_level: f64,
    scale_factor: f64,

    amplitude: f64,
    frequency: f64,
    octaves: usize,
    persistence: f64,
    lacunarity: f64,
}

impl Default for LevelGenerator {
    fn default() -> Self {
        Self {
            perlin: Perlin::new(0),
            surface_level: 40.0,
            scale_factor: 350.0,

            amplitude: 15.0,
            frequency: 2.5,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
        }
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
        let height = self.evaluate_fbm(
            pos.x as f64 / self.scale_factor,
            pos.z as f64 / self.scale_factor,
        ) as i64;

        if pos.y == height {
            Some(Block::Grass)
        } else if pos.y < height && pos.y >= height - 3 {
            Some(Block::Dirt)
        } else if pos.y < height {
            Some(Block::Rock)
        } else {
            None
        }
    }

    fn evaluate_fbm(&self, x: f64, y: f64) -> f64 {
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
}
