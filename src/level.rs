use std::{f32::consts::FRAC_PI_2, sync::Arc};

use bevy::{
    pbr::ExtendedMaterial,
    prelude::*,
    render::primitives::Aabb,
    tasks::{block_on, AsyncComputeTaskPool, Task},
    utils::HashMap,
};
use bevy_xpbd_3d::prelude::*;
use big_space::GridCell;
use futures_lite::future;
use itertools::Itertools;
use parking_lot::{Mutex, RwLock};
use rusqlite::Connection;

use crate::{
    chunk::{generate_mesh, AdjacentChunks, Chunk, ChunkData, CHUNK_SIZE},
    chunk_material::ChunkMaterial,
    level_generator::LevelGenerator,
    player::{Player, RenderDistance},
    voxel::chunk_pos::ChunkPos,
    GameAssets, GameState,
};

pub struct LevelPlugin;

impl Plugin for LevelPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_level)
            .add_systems(OnEnter(GameState::InGame), setup_material)
            .add_systems(
                Update,
                (update_chunks, build_meshes).run_if(in_state(GameState::InGame)),
            );
    }
}

fn remove_chunk(
    mut commands: Commands,
    level: Res<Level>,
    player: Query<&GridCell<i32>, With<Player>>,
    chunks: Query<(&ChunkPos, Entity)>,
) {
    let &grid_cell = player.single();
    let chunk_pos = ChunkPos::new(IVec3::new(grid_cell.x, grid_cell.y - 1, grid_cell.z));
    let Some(chunk) = level.chunks.get(&chunk_pos) else {
        return;
    };
    for block in chunk.write().blocks.iter_mut() {
        *block = None;
    }
    let entity = chunks.iter().find(|chunk| *chunk.0 == chunk_pos).unwrap().1;
    commands.entity(entity).insert(Dirty);

    for pos in chunk_pos.adjacent_chunks() {
        if level.chunks.contains_key(&pos) {
            let entity = chunks.iter().find(|chunk| *chunk.0 == pos).unwrap().1;
            commands.entity(entity).insert(Dirty);
        }
    }
}

#[derive(Resource)]
struct ChunkMaterialInstance(Handle<ExtendedMaterial<StandardMaterial, ChunkMaterial>>);

#[derive(Resource)]
pub struct Level {
    pub chunks: HashMap<ChunkPos, Chunk>,
    generator: Arc<LevelGenerator>,
    database: Arc<Mutex<Connection>>,
}

impl Level {
    fn adjacent(&self, pos: ChunkPos) -> AdjacentChunks {
        AdjacentChunks {
            left: self.chunks.get(&pos.left()).cloned(),
            right: self.chunks.get(&pos.right()).cloned(),
            top: self.chunks.get(&pos.top()).cloned(),
            bottom: self.chunks.get(&pos.bottom()).cloned(),
            front: self.chunks.get(&pos.front()).cloned(),
            back: self.chunks.get(&pos.back()).cloned(),
        }
    }
}

#[derive(Component)]
struct GenerateChunkTask(Task<Chunk>);

#[derive(Component)]
struct BuildMeshTask(Task<(Mesh, Option<Collider>)>);

#[derive(Component)]
pub struct Dirty;

fn setup_level(mut commands: Commands) {
    let conn = Connection::open("chunks.sqlite").unwrap();

    conn.execute(
        "
        CREATE TABLE IF NOT EXISTS chunks (
            x INTEGER NOT NULL,
            y INTEGER NOT NULL,
            z INTEGER NOT NULL,
            data BLOB NOT NULL,
            UNIQUE(x, y, z)
        )
        ",
        (),
    )
    .unwrap();

    commands.insert_resource(Level {
        chunks: HashMap::default(),
        generator: Arc::default(),
        database: Arc::new(Mutex::new(conn)),
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 7500.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 500.0, 0.0),
            rotation: Quat::from_rotation_x(-FRAC_PI_2),
            ..default()
        },
        ..default()
    });
}

fn setup_material(
    mut commands: Commands,
    game_assets: Res<GameAssets>,
    mut images: ResMut<Assets<Image>>,
    mut materials: ResMut<Assets<ExtendedMaterial<StandardMaterial, ChunkMaterial>>>,
) {
    let image_handle = game_assets.block_textures.clone();
    let image = images.get_mut(&image_handle).unwrap();

    image.reinterpret_stacked_2d_as_array(5);

    let material_handle = materials.add(ExtendedMaterial {
        base: StandardMaterial {
            perceptual_roughness: 1.0,
            ..default()
        },
        extension: ChunkMaterial {
            texture: image_handle,
        },
    });

    commands.insert_resource(ChunkMaterialInstance(material_handle));
}

fn update_chunks(
    mut commands: Commands,
    mut level: ResMut<Level>,
    render_distance: Res<RenderDistance>,
    material: Res<ChunkMaterialInstance>,
    player: Query<(&GridCell<i32>, &Transform), With<Player>>,
    mut chunks: Query<(&ChunkPos, Entity, Option<&mut GenerateChunkTask>)>,
) {
    let (grid_cell, transform) = player.single();

    let translation = (transform.translation / CHUNK_SIZE as f32).round();

    let center = ChunkPos::new(IVec3::new(
        grid_cell.x + translation.x as i32,
        grid_cell.y + translation.y as i32,
        grid_cell.z + translation.z as i32,
    ));

    let visible_chunks = center.chunks_within_radius(render_distance.0);

    let thread_pool = AsyncComputeTaskPool::get();

    let mut chunk_entities = HashMap::new();
    let mut chunk_tasks = HashMap::new();

    for (chunk_pos, entity, task) in chunks.iter_mut() {
        chunk_entities.insert(*chunk_pos, entity);
        if let Some(task) = task {
            chunk_tasks.insert(*chunk_pos, task);
        }
    }

    let mut inserted_chunks = Vec::new();

    // Insert chunk
    for (&pos, &entity) in chunk_entities
        .iter()
        .filter(|chunk| visible_chunks.contains(chunk.0))
    {
        let Some(task) = chunk_tasks.get_mut(&pos) else {
            continue;
        };

        let Some(chunk) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        level.chunks.insert(pos, chunk);
        inserted_chunks.push(pos);

        commands
            .entity(entity)
            .remove::<GenerateChunkTask>()
            .insert((Dirty, material.0.clone()));
    }

    // Regenerate chunks
    for pos in inserted_chunks
        .iter()
        .flat_map(|pos| pos.adjacent_chunks())
        .unique()
    {
        let Some(&entity) = chunk_entities.get(&pos) else {
            continue;
        };

        commands.entity(entity).insert(Dirty);
    }

    // Spawn chunks
    for &pos in visible_chunks
        .iter()
        .filter(|pos| !chunk_entities.contains_key(*pos))
        .take(25 - chunk_tasks.len())
    {
        let generator = level.generator.clone();
        let db = level.database.clone();
        let task = thread_pool.spawn(async move {
            let bin_data: rusqlite::Result<Vec<u8>> = db.lock().query_row(
                "
                SELECT data FROM chunks
                WHERE x = ? AND y = ? AND z = ?
                ",
                (pos.x, pos.y, pos.z),
                |row| row.get(0),
            );

            let chunk_data = if let Ok(bin_data) = bin_data {
                ChunkData::deserialize(&bin_data).unwrap()
            } else {
                let chunk_data = generator.generate_chunk(pos);
                let bin_data = chunk_data.serialize();
                db.lock()
                    .execute(
                        "
                    INSERT INTO chunks (x, y, z, data)
                    VALUES (?1, ?2, ?3, ?4)
                    ",
                        (pos.x, pos.y, pos.z, bin_data),
                    )
                    .unwrap();
                chunk_data
            };

            Arc::new(RwLock::new(chunk_data))
        });

        commands.spawn((
            pos,
            SpatialBundle::default(),
            GridCell::<i32>::new(pos.x, pos.y, pos.z),
            RigidBody::Static,
            Friction::new(0.0),
            Restitution::new(0.0),
            GenerateChunkTask(task),
        ));
    }

    // Despawn chunks
    for (pos, &entity) in chunk_entities
        .iter()
        .filter(|chunk| !visible_chunks.contains(chunk.0))
    {
        level.chunks.remove(pos);
        commands.entity(entity).despawn();
    }
}

fn build_meshes(
    mut commands: Commands,
    level: Res<Level>,
    dirty: Query<(Entity, &ChunkPos), With<Dirty>>,
    mut pending: Query<(Entity, &mut BuildMeshTask)>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let thread_pool = AsyncComputeTaskPool::get();

    // Insert mesh
    for (entity, mut task) in pending.iter_mut() {
        let Some((mesh, collider)) = block_on(future::poll_once(&mut task.0)) else {
            continue;
        };

        let mut entity = commands.entity(entity);

        if let Some(collider) = collider {
            entity.insert(collider);
        } else {
            entity.remove::<Collider>();
        }

        entity
            .remove::<BuildMeshTask>()
            .remove::<Aabb>()
            .insert(meshes.add(mesh));
    }

    // Spawn mesh task
    for (entity, &pos) in dirty.iter() {
        let Some(chunk) = level.chunks.get(&pos).cloned() else {
            continue;
        };

        let adjacent = level.adjacent(pos);
        let gen = generate_mesh(chunk, adjacent);
        let task = thread_pool.spawn(gen);

        commands
            .entity(entity)
            .remove::<Dirty>()
            .insert(BuildMeshTask(task));
    }
}
