use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{ScrollArea, Slider, Ui, Window},
    EguiContexts,
};
use big_space::{FloatingOriginSettings, GridCell};

use crate::{
    player::{JumpHeight, MouseSensitivity, MovementSpeed, Player, Reach, RenderDistance},
    voxel::chunk_pos::ChunkPos,
    GameState,
};

pub struct EguiMenuPlugin;

impl Plugin for EguiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_ui.run_if(in_state(GameState::InGame)));
    }
}

fn render_ui(
    floating_origin: Res<FloatingOriginSettings>,
    diagnostics: Res<DiagnosticsStore>,
    mut render_distance: ResMut<RenderDistance>,
    mut movement_speed: ResMut<MovementSpeed>,
    mut jump_height: ResMut<JumpHeight>,
    mut mouse_sensitivity: ResMut<MouseSensitivity>,
    mut gizmo_config: ResMut<GizmoConfig>,
    mut reach: ResMut<Reach>,
    mut contexts: EguiContexts,
    player: Query<(&GridCell<i32>, &Transform, &GlobalTransform), With<Player>>,
    chunks: Query<&ChunkPos>,
) {
    let (grid_cell, transform, _global_transform) = player.single();

    let content = |ui: &mut Ui| {
        ui.add(Slider::new(&mut render_distance.0, 4..=24).text("Render Distance"));
        ui.add(Slider::new(&mut movement_speed.0, 0.0..=50.0).text("Movement Speed"));
        ui.add(Slider::new(&mut jump_height.0, 0.0..=100.0).text("Jump height"));
        ui.add(Slider::new(&mut mouse_sensitivity.0, 0.00001..=0.0002).text("Mouse Sensitivity"));
        ui.add(Slider::new(&mut reach.0, 0.0..=100.0).text("Reach"));

        ui.separator();

        ui.checkbox(&mut gizmo_config.enabled, "Debug Rendering");

        ui.separator();

        ui.label(format!(
            "Chunk: X {}, Y {}, Z {}",
            grid_cell.x, grid_cell.y, grid_cell.z
        ));

        let pos = floating_origin.grid_position_double(grid_cell, transform);

        ui.label(format!(
            "Position: X {:.2}, Y {:.2}, Z {:.2}",
            pos.x, pos.y, pos.z
        ));

        ui.label(format!("Loaded Chunks: {}", chunks.iter().len()));

        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            ui.label(format!("FPS: {value:>4.0}"));
        }
    };

    Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ScrollArea::vertical().show(ui, content);
    });
}
