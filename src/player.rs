use std::{f32::consts::PI as PI_32, f64::consts::PI as PI_64};

use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    math::{DQuat, DVec3},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_xpbd_3d::prelude::*;
use big_space::{FloatingOrigin, GridCell};

use crate::{
    block::Block,
    level::{Dirty, Level},
    voxel::{block_pos::BlockPos, chunk::CHUNK_SIZE, chunk_pos::ChunkPos},
    GameState,
};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderDistance>()
            .init_resource::<InputState>()
            .init_resource::<MovementSpeed>()
            .init_resource::<JumpHeight>()
            .init_resource::<MouseSensitivity>()
            .add_systems(Startup, (setup_player, setup_input))
            .add_systems(
                Update,
                (
                    player_look,
                    player_move,
                    break_block,
                    toggle_grab_cursor,
                    update_fog,
                )
                    .run_if(in_state(GameState::InGame)),
            );
    }
}

#[derive(Resource)]
pub struct RenderDistance(pub i32);

impl Default for RenderDistance {
    fn default() -> Self {
        Self(10)
    }
}

#[derive(Resource, Default)]
struct InputState {
    motion_reader: ManualEventReader<MouseMotion>,
}

#[derive(Resource)]
pub struct MovementSpeed(pub f32);

impl Default for MovementSpeed {
    fn default() -> Self {
        Self(110.0)
    }
}

#[derive(Resource)]
pub struct JumpHeight(pub f64);

impl Default for JumpHeight {
    fn default() -> Self {
        Self(8.0)
    }
}

#[derive(Resource)]
pub struct MouseSensitivity(pub f32);

impl Default for MouseSensitivity {
    fn default() -> Self {
        Self(0.0001)
    }
}

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

fn setup_player(mut commands: Commands) {
    commands
        .spawn((
            Player,
            SpatialBundle::default(),
            GridCell::<i32>::new(0, 6, 0),
            FloatingOrigin,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            Friction::new(0.0),
            Restitution::new(0.0),
            Collider::cuboid(0.8, 1.7, 0.8),
            ShapeCaster::new(
                Collider::cuboid(0.79, 0.1, 0.79),
                DVec3::ZERO,
                DQuat::default(),
                DVec3::NEG_Y,
            )
            .with_max_time_of_impact(1.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3dBundle {
                    camera: Camera {
                        hdr: true,
                        ..default()
                    },
                    projection: Projection::Perspective(PerspectiveProjection {
                        fov: PI_32 / 180.0 * 60.0,
                        ..default()
                    }),
                    transform: Transform::from_xyz(0.0, 0.7, 0.0),
                    ..default()
                },
                FogSettings {
                    color: Color::rgba(0.2, 0.5, 0.8, 1.0),
                    ..default()
                },
            ));
        });
}

fn break_block(
    mut commands: Commands,
    spatial_query: SpatialQuery,
    level: Res<Level>,
    mouse: Res<Input<MouseButton>>,
    player: Query<(Entity, &GridCell<i32>), With<Player>>,
    camera: Query<&GlobalTransform, With<PlayerCamera>>,
    chunks: Query<(Entity, &ChunkPos)>,
) {
    let (player_entity, grid_cell) = player.single();
    let global_transform = camera.single();

    let Some(hit) = spatial_query.cast_ray(
        global_transform.translation().as_dvec3(),
        global_transform.forward().as_dvec3(),
        5.0,
        true,
        SpatialQueryFilter::new().without_entities([player_entity]),
    ) else {
        return;
    };

    let (hit_pos, block) = if mouse.just_pressed(MouseButton::Left) {
        let hit_pos = global_transform.translation()
            + global_transform.forward() * (hit.time_of_impact + 0.01) as f32;
        (hit_pos, None)
    } else if mouse.just_pressed(MouseButton::Right) {
        let hit_pos = global_transform.translation()
            + global_transform.forward() * (hit.time_of_impact - 0.01) as f32;
        (hit_pos, Some(Block::Sand))
    } else {
        return;
    };

    let floor = hit_pos.floor().as_i64vec3();
    let block_pos =
        ChunkPos::from(*grid_cell).block_pos() + BlockPos::new(floor.x, floor.y, floor.z);
    let chunk_pos = block_pos.chunk_pos();

    let Some(chunk) = level.chunks.get(&chunk_pos) else {
        return;
    };

    let pos = block_pos.relative_pos();
    let mut chunk = chunk.write();
    let block_mut = chunk.block_mut(pos.0, pos.1, pos.2);

    if block.is_none() {
        *block_mut = None;
    } else if block_mut.is_none() {
        *block_mut = block;
    } else {
        return;
    }

    drop(chunk);

    for pos in [vec![chunk_pos], chunk_pos.adjacent().to_vec()]
        .iter()
        .flatten()
    {
        let Some((entity, _)) = chunks.iter().find(|c| c.1 == pos) else {
            continue;
        };

        commands.entity(entity).insert(Dirty);
    }
}

fn raycast_blocks(
    level: &Level,
    mut block_pos: BlockPos,
    direction: Vec3,
    max_distance: i32,
) -> Result<BlockPos, BlockPos> {
    // Determine the step direction (1 or -1) for x, y, z
    let step_x = if direction.x >= 0.0 {
        BlockPos::X
    } else {
        BlockPos::NEG_X
    };
    let step_y = if direction.y >= 0.0 {
        BlockPos::Y
    } else {
        BlockPos::NEG_Y
    };
    let step_z = if direction.z >= 0.0 {
        BlockPos::Z
    } else {
        BlockPos::NEG_Z
    };

    // How far along the ray must we move for each component
    // to cross a block boundary?
    let delta_x = (1.0 / direction.x).abs();
    let delta_y = (1.0 / direction.y).abs();
    let delta_z = (1.0 / direction.z).abs();

    // Initial values
    let mut t_next_x = delta_x;
    let mut t_next_y = delta_y;
    let mut t_next_z = delta_z;

    // Traverse the grid up to max_distance
    for _ in 0..max_distance {
        // Check for a block at the current position
        let (x, y, z) = block_pos.relative_pos();
        if let Some(chunk) = level.chunks.get(&block_pos.chunk_pos()) {
            if chunk.read().block(x, y, z).is_some() {
                return Ok(block_pos);
            }
        };

        // Move ray to the next nearest block boundary in x, y, or z
        if t_next_x < t_next_y && t_next_x < t_next_z {
            block_pos += step_x;
            t_next_x += delta_x;
        } else if t_next_y < t_next_z {
            block_pos += step_y;
            t_next_y += delta_y;
        } else {
            block_pos += step_z;
            t_next_z += delta_z;
        }
    }

    // Ray didn't hit any block within max_distance
    Err(block_pos)
}

fn setup_input(mut window: Query<&mut Window, With<PrimaryWindow>>) {
    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    grab_cursor(&mut window);
}

fn update_fog(
    render_distance: Res<RenderDistance>,
    mut query: Query<&mut FogSettings, With<PlayerCamera>>,
) {
    let mut fog = query.single_mut();

    let scale = CHUNK_SIZE as f32;
    let distance = render_distance.0 as f32 * scale;

    fog.falloff = FogFalloff::Linear {
        start: distance - scale * 2.0,
        end: distance - scale,
    };
}

fn player_look(
    mouse_sensitivity: Res<MouseSensitivity>,
    motion: Res<Events<MouseMotion>>,
    mut state: ResMut<InputState>,
    window: Query<&Window, With<PrimaryWindow>>,
    mut player: Query<&mut Transform, With<PlayerCamera>>,
) {
    let Ok(window) = window.get_single() else {
        return;
    };

    if window.cursor.grab_mode == CursorGrabMode::None {
        return;
    }

    let mut transform = player.single_mut();

    for event in state.motion_reader.read(&motion) {
        let (mut yaw, mut pitch, _) = transform.rotation.to_euler(EulerRot::YXZ);

        let window_scale = window.height().min(window.width());
        pitch -= (mouse_sensitivity.0 * event.delta.y * window_scale).to_radians();
        yaw -= (mouse_sensitivity.0 * event.delta.x * window_scale).to_radians();

        let yaw_rotation = Quat::from_axis_angle(Vec3::Y, yaw);
        let pitch_rotation = Quat::from_axis_angle(Vec3::X, pitch.clamp(-1.54, 1.54));
        transform.rotation = yaw_rotation * pitch_rotation;
    }
}

fn player_move(
    time: Res<Time>,
    keyboard: Res<Input<KeyCode>>,
    movement_speed: Res<MovementSpeed>,
    jump_height: Res<JumpHeight>,
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<(&mut LinearVelocity, &ShapeHits, &Rotation), With<Player>>,
) {
    let camera_transform = camera.single();
    let (mut velocity, shape_hits, rotation) = player.single_mut();

    let local_z: Vec3 = camera_transform.local_z();
    let forward = -Vec3::new(local_z.x, 0.0, local_z.z);
    let right = Vec3::new(local_z.z, 0.0, -local_z.x);

    let mut movement = Vec3::ZERO;

    macro_rules! apply {
            ($op:tt $dir:ident if $key:expr) => {
                if keyboard.pressed($key) {
                    movement $op $dir;
                }
            };
        }

    apply!(+= forward if KeyCode::W);
    apply!(-= forward if KeyCode::S);
    apply!(+= right if KeyCode::D);
    apply!(-= right if KeyCode::A);

    velocity.0 +=
        (movement.normalize_or_zero() * time.delta_seconds() * movement_speed.0).as_dvec3();

    velocity.0.x *= (1.0 - time.delta_seconds() * 10.0).max(0.0) as f64;
    velocity.0.z *= (1.0 - time.delta_seconds() * 10.0).max(0.0) as f64;

    let on_ground = shape_hits
        .iter()
        .any(|hit| rotation.rotate(-hit.normal2).angle_between(DVec3::Y).abs() <= PI_64 * 0.45);

    if keyboard.pressed(KeyCode::Space) && on_ground {
        velocity.y = jump_height.0;
    }
}

fn toggle_grab_cursor(
    keyboard: Res<Input<KeyCode>>,
    mut window: Query<&mut Window, With<PrimaryWindow>>,
) {
    if !keyboard.just_pressed(KeyCode::Escape) {
        return;
    }

    let Ok(mut window) = window.get_single_mut() else {
        return;
    };

    if window.cursor.grab_mode == CursorGrabMode::None {
        grab_cursor(&mut window);
    } else {
        ungrab_cursor(&mut window);
    }
}

fn grab_cursor(window: &mut Window) {
    window.cursor.grab_mode = CursorGrabMode::Confined;
    window.cursor.visible = false;
}

fn ungrab_cursor(window: &mut Window) {
    window.cursor.grab_mode = CursorGrabMode::None;
    window.cursor.visible = true;
}
