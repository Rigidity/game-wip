#![allow(dead_code)]
#![allow(clippy::type_complexity)]

use bevy::{
    audio::PlaybackMode, diagnostic::FrameTimeDiagnosticsPlugin, pbr::ExtendedMaterial, prelude::*,
};
use bevy_asset_loader::prelude::*;

mod block;
mod chunk;
mod chunk_material;
mod egui_menu;
mod level;
mod level_generator;
mod mesh_builder;
mod physics;
mod player;

use bevy_egui::EguiPlugin;
use big_space::{FloatingOriginPlugin, FloatingOriginSettings};
use chunk::CHUNK_SIZE;
use chunk_material::ChunkMaterial;
use egui_menu::EguiMenuPlugin;
use level::LevelPlugin;
use physics::PhysicsPlugin;
use player::PlayerPlugin;

#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    LoadingAssets,
    InGame,
}

#[derive(AssetCollection, Resource)]
pub struct GameAssets {
    #[asset(path = "blocks.png")]
    pub block_textures: Handle<Image>,
}

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .disable::<TransformPlugin>()
                .set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, ChunkMaterial>>::default(),
            FloatingOriginPlugin::<i32>::default(),
            EguiPlugin,
            LevelPlugin,
            PlayerPlugin,
            PhysicsPlugin,
            EguiMenuPlugin,
        ))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::LoadingAssets).continue_to_state(GameState::InGame),
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameState::LoadingAssets)
        .insert_resource(GlobalVolume::new(0.25))
        .insert_resource(GizmoConfig {
            enabled: false,
            ..default()
        })
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.85,
        })
        .insert_resource(FloatingOriginSettings::new(CHUNK_SIZE as f32, 0.0))
        .add_systems(Startup, setup_music)
        .run();
}

fn setup_music(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(AudioBundle {
        source: asset_server.load("music.ogg"),
        settings: PlaybackSettings {
            mode: PlaybackMode::Loop,
            ..default()
        },
    });
}
