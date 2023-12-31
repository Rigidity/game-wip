use bevy::{
    ecs::event::ManualEventReader,
    input::mouse::MouseMotion,
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow},
};
use bevy_xpbd_3d::prelude::*;
use big_space::{FloatingOrigin, GridCell};

use crate::{chunk::CHUNK_SIZE, GameState};

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<RenderDistance>()
            .init_resource::<InputState>()
            .init_resource::<MovementSpeed>()
            .init_resource::<MouseSensitivity>()
            // Make sure the floating origin is there on startup.
            .add_systems(Startup, (setup_player, setup_input))
            .add_systems(
                Update,
                (player_look, player_move, toggle_grab_cursor, update_fog)
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
            GridCell::<i32>::new(0, 3, 0),
            FloatingOrigin,
            RigidBody::Dynamic,
            LockedAxes::ROTATION_LOCKED,
            LinearVelocity::default(),
            Friction::new(0.0),
            Restitution::new(0.0),
            Collider::cuboid(0.8, 1.7, 0.8),
        ))
        .with_children(|parent| {
            parent.spawn((
                PlayerCamera,
                Camera3dBundle {
                    camera: Camera {
                        hdr: true,
                        ..default()
                    },
                    transform: Transform::from_xyz(0.0, 1.5, 0.0),
                    ..default()
                },
                FogSettings {
                    color: Color::rgba(0.2, 0.5, 0.8, 1.0),
                    ..default()
                },
            ));
        });
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
    camera: Query<&Transform, With<PlayerCamera>>,
    mut player: Query<&mut LinearVelocity, With<Player>>,
) {
    let camera_transform = camera.single();
    let mut velocity = player.single_mut();

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

    if keyboard.just_pressed(KeyCode::Space) {
        velocity.y = 9.0;
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
