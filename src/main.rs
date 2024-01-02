#![allow(dead_code)]
#![allow(clippy::type_complexity)]
#![allow(clippy::too_many_arguments)]

use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin, math::DVec3, pbr::ExtendedMaterial, prelude::*,
    window::WindowResolution,
};
use bevy_asset_loader::prelude::*;

mod block;
mod chunk;
mod chunk_material;
mod egui_menu;
mod level;
mod level_generator;
mod mesh_builder;
mod player;
mod voxel;

use bevy_egui::EguiPlugin;
use bevy_xpbd_3d::prelude::*;
use big_space::{
    bevy_xpbd::floating_origin_sync::FloatingOriginSyncPlugin, FloatingOriginPlugin,
    FloatingOriginSettings,
};
use chunk_material::ChunkMaterial;
use egui_menu::EguiMenuPlugin;
use level::LevelPlugin;
use player::PlayerPlugin;
use voxel::chunk::CHUNK_SIZE;

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
    let window = Window {
        title: "Voxel Game".into(),
        resolution: WindowResolution::new(1400.0, 800.0),
        ..default()
    };

    App::new()
        .add_plugins((
            DefaultPlugins
                .build()
                .set(WindowPlugin {
                    primary_window: Some(window),
                    ..default()
                })
                .disable::<TransformPlugin>()
                .set(ImagePlugin::default_nearest()),
            FrameTimeDiagnosticsPlugin,
            MaterialPlugin::<ExtendedMaterial<StandardMaterial, ChunkMaterial>>::default(),
            FloatingOriginPlugin::<i32>::default(),
            PhysicsPlugins::default().build().disable::<SyncPlugin>(),
            FloatingOriginSyncPlugin::<i32>::default(),
            EguiPlugin,
            LevelPlugin,
            PlayerPlugin,
            EguiMenuPlugin,
        ))
        .add_state::<GameState>()
        .add_loading_state(
            LoadingState::new(GameState::LoadingAssets).continue_to_state(GameState::InGame),
        )
        .add_collection_to_loading_state::<_, GameAssets>(GameState::LoadingAssets)
        .insert_resource(FloatingOriginSettings::new(CHUNK_SIZE as f32, 0.0))
        .insert_resource(ClearColor(Color::rgb(0.2, 0.5, 0.8)))
        .insert_resource(Gravity(DVec3::NEG_Y * 26.0))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.85,
        })
        .insert_resource(PrepareConfig {
            position_to_transform: false,
            transform_to_position: true,
        })
        .run();
}
