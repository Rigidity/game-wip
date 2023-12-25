use bevy::prelude::*;
use big_space::GridCell;
use itertools::Itertools;

use crate::{
    block::Block,
    chunk::{ChunkPos, CHUNK_SIZE},
    level::Level,
    GameState,
};

pub struct PhysicsPlugin;

impl Plugin for PhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Gravity>().add_systems(
            FixedUpdate,
            (apply_gravity, apply_velocity)
                .chain()
                .run_if(in_state(GameState::InGame)),
        );
    }
}

#[derive(Component, Clone, Copy)]
pub struct Hitbox(pub Vec3);

pub fn normalize_position(grid_cell: &mut GridCell<i32>, translation: &mut Vec3) {
    // Adjust for overflow or underflow in each dimension
    for (grid_coord, rel_coord) in [
        (&mut grid_cell.x, &mut translation.x),
        (&mut grid_cell.y, &mut translation.y),
        (&mut grid_cell.z, &mut translation.z),
    ] {
        // Calculate the number of chunks overflowed or underflowed
        let chunk_overflow = (*rel_coord / CHUNK_SIZE as f32).floor() as i32;

        // Adjust the grid cell and relative position
        *grid_coord += chunk_overflow;
        *rel_coord -= chunk_overflow as f32 * CHUNK_SIZE as f32;

        // Ensure relative position stays within 0 and CHUNK_SIZE
        if *rel_coord < 0.0 {
            *grid_coord -= 1;
            *rel_coord += CHUNK_SIZE as f32;
        } else if *rel_coord >= CHUNK_SIZE as f32 {
            *grid_coord += 1;
            *rel_coord -= CHUNK_SIZE as f32;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Collider {
    min_x: f32,
    min_y: f32,
    min_z: f32,
    max_x: f32,
    max_y: f32,
    max_z: f32,
}

impl Collider {
    pub fn new(center: Vec3, size: Vec3) -> Self {
        Self {
            min_x: center.x - size.x / 2.0,
            min_y: center.y - size.y / 2.0,
            min_z: center.z - size.z / 2.0,
            max_x: center.x + size.x / 2.0,
            max_y: center.y + size.y / 2.0,
            max_z: center.z + size.z / 2.0,
        }
    }

    pub fn extend(self, value: Vec3) -> Self {
        Self {
            min_x: self.min_x + value.x.min(0.0),
            min_y: self.min_y + value.y.min(0.0),
            min_z: self.min_z + value.z.min(0.0),
            max_x: self.max_x + value.x.max(0.0),
            max_y: self.max_y + value.y.max(0.0),
            max_z: self.max_z + value.z.max(0.0),
        }
    }

    pub fn offset(self, value: Vec3) -> Self {
        Self {
            min_x: self.min_x + value.x,
            min_y: self.min_y + value.y,
            min_z: self.min_z + value.z,
            max_x: self.max_x + value.x,
            max_y: self.max_y + value.y,
            max_z: self.max_z + value.z,
        }
    }

    pub fn calculate_x_offset(self, other: Collider, mut x_offset: f32) -> f32 {
        if !(other.max_y > self.min_y
            && other.min_y < self.max_y
            && other.max_z > self.min_z
            && other.min_z < self.max_z)
        {
            return x_offset;
        }

        if x_offset > 0.0 && other.max_x <= self.min_x {
            let this_offset = self.min_x - other.max_x;
            if this_offset < x_offset {
                x_offset = this_offset;
            }
        } else if x_offset < 0.0 && other.min_x >= self.max_x {
            let this_offset = self.max_x - other.min_x;
            if this_offset > x_offset {
                x_offset = this_offset;
            }
        }

        x_offset
    }

    pub fn calculate_y_offset(self, other: Collider, mut y_offset: f32, is_debug: bool) -> f32 {
        if !(other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_z > self.min_z
            && other.min_z < self.max_z)
        {
            return y_offset;
        }

        if y_offset > 0.0 && other.max_y <= self.min_y {
            let this_offset = self.min_y - other.max_y;
            if this_offset < y_offset {
                y_offset = this_offset;
            }
        } else if y_offset < 0.0 && other.min_y >= self.max_y {
            if is_debug {
                // panic!();
            }
            let this_offset = self.max_y - other.min_y;
            if this_offset > y_offset {
                y_offset = this_offset;
            }
        }

        y_offset
    }

    pub fn calculate_z_offset(self, other: Collider, mut z_offset: f32) -> f32 {
        if !(other.max_x > self.min_x
            && other.min_x < self.max_x
            && other.max_y > self.min_y
            && other.min_y < self.max_y)
        {
            return z_offset;
        }

        if z_offset > 0.0 && other.max_z <= self.min_z {
            let this_offset = self.min_z - other.max_z;
            if this_offset < z_offset {
                z_offset = this_offset;
            }
        } else if z_offset < 0.0 && other.min_z >= self.max_z {
            let this_offset = self.max_z - other.min_z;
            if this_offset > z_offset {
                z_offset = this_offset;
            }
        }

        z_offset
    }

    pub fn check_collision(self, other: Collider) -> bool {
        self.min_x < other.max_x
            && self.max_x > other.min_x
            && self.min_y < other.max_y
            && self.max_y > other.min_y
            && self.min_z < other.max_z
            && self.max_z > other.min_z
    }
}

#[derive(Resource)]
struct Gravity(f32);

impl Default for Gravity {
    fn default() -> Self {
        Self(9.81 * 2.5)
    }
}

#[derive(Component, Default)]
pub struct Velocity(pub Vec3);

fn apply_gravity(time: Res<Time>, gravity: Res<Gravity>, mut query: Query<&mut Velocity>) {
    for mut velocity in query.iter_mut() {
        velocity.0 += time.delta_seconds() * Vec3::NEG_Y * gravity.0;
    }
}

fn apply_velocity(
    level: Res<Level>,
    time: Res<Time>,
    mut query: Query<(
        &GridCell<i32>,
        &GlobalTransform,
        Option<&Hitbox>,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    for (&grid_cell, &global_transform, hitbox, mut transform, mut velocity) in query.iter_mut() {
        let mut x = velocity.0.x * time.delta_seconds();
        let mut y = velocity.0.y * time.delta_seconds();
        let mut z = velocity.0.z * time.delta_seconds();

        if let Some(&hitbox) = hitbox {
            let mut normalized_grid_cell = grid_cell;
            let mut normalized_position = global_transform.translation();
            normalize_position(&mut normalized_grid_cell, &mut normalized_position);

            let mut collider = Collider::new(normalized_position, hitbox.0);

            let block_colliders: Vec<(Collider, bool)> = block_colliders_within(
                &level,
                normalized_grid_cell,
                collider.extend(Vec3::new(x, y, z)),
            )
            .collect();

            let extra = 0.001;

            for (block_collider, is_debug) in block_colliders.iter() {
                let old = y;
                y = block_collider.calculate_y_offset(collider, y, *is_debug);
                if y != old {
                    if y < old {
                        y -= extra;
                    } else {
                        y += extra;
                    }
                    velocity.0.y = 0.0;
                }
            }
            collider = collider.offset(Vec3::new(0.0, y, 0.0));

            for (block_collider, _) in block_colliders.iter() {
                let old = x;
                x = block_collider.calculate_x_offset(collider, x);
                if x != old {
                    if x < old {
                        x -= extra;
                    } else {
                        x += extra;
                    }

                    velocity.0.x = 0.0;
                }
            }
            collider = collider.offset(Vec3::new(x, 0.0, 0.0));

            for (block_collider, _) in block_colliders.iter() {
                let old = z;
                z = block_collider.calculate_z_offset(collider, z);
                if z != old {
                    if z < old {
                        z -= extra;
                    } else {
                        z += extra;
                    }
                    velocity.0.z = 0.0;
                }
            }
        }

        transform.translation += Vec3::new(x, y, z);
        velocity.0.x *= 1.0 - time.delta_seconds() * 9.0;
        velocity.0.z *= 1.0 - time.delta_seconds() * 9.0;
    }
}

fn block_colliders_within(
    level: &Level,
    grid_cell: GridCell<i32>,
    collider: Collider,
) -> impl Iterator<Item = (Collider, bool)> + '_ {
    let min_x = collider.min_x.floor() as i32;
    let max_x = collider.max_x.ceil() as i32;
    let min_y = collider.min_y.floor() as i32;
    let max_y = collider.max_y.ceil() as i32;
    let min_z = collider.min_z.floor() as i32;
    let max_z = collider.max_z.ceil() as i32;

    (min_x..=max_x)
        .cartesian_product(min_y..=max_y)
        .cartesian_product(min_z..=max_z)
        .filter_map(move |((x, y), z)| {
            let mut block_position = Vec3::new(x as f32, y as f32, z as f32);
            let mut block_grid_cell = grid_cell;
            normalize_position(&mut block_grid_cell, &mut block_position);

            let chunk = level.chunks.get(&ChunkPos::new(IVec3::new(
                block_grid_cell.x,
                block_grid_cell.y,
                block_grid_cell.z,
            )))?;

            let block = chunk.read().block(
                block_position.x as usize,
                block_position.y as usize,
                block_position.z as usize,
            );

            block?;

            let offset = block_grid_cell - grid_cell;

            block_position += Vec3::new(offset.x as f32, offset.y as f32, offset.z as f32)
                * CHUNK_SIZE as f32
                + 0.5;

            if block == Some(Block::Debug) {
                println!(
                    "\n\nFrom {grid_cell:?} at ({x}, {y}, {z})
To {block_grid_cell:?} at ({}, {}, {})
Offset {offset:?}",
                    block_position.x, block_position.y, block_position.z
                );
            }

            Some((
                Collider::new(block_position, Vec3::ONE),
                block == Some(Block::Debug),
            ))
        })
}
