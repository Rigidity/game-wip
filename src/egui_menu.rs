use bevy::{
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    prelude::*,
};
use bevy_egui::{
    egui::{Slider, Window},
    EguiContexts,
};
use big_space::GridCell;

use crate::{
    player::{MouseSensitivity, MovementSpeed, Player, RenderDistance},
    GameState,
};

pub struct EguiMenuPlugin;

impl Plugin for EguiMenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, render_ui.run_if(in_state(GameState::InGame)));
    }
}

fn render_ui(
    diagnostics: Res<DiagnosticsStore>,
    mut render_distance: ResMut<RenderDistance>,
    mut movement_speed: ResMut<MovementSpeed>,
    mut mouse_sensitivity: ResMut<MouseSensitivity>,
    mut contexts: EguiContexts,
    player: Query<(&GridCell<i32>, &GlobalTransform), With<Player>>,
) {
    let (grid_cell, global_transform) = player.single();

    Window::new("Debug").show(contexts.ctx_mut(), |ui| {
        ui.add(Slider::new(&mut render_distance.0, 4..=24).text("Render Distance"));
        ui.add(Slider::new(&mut movement_speed.0, 50.0..=500.0).text("Movement Speed"));
        ui.add(Slider::new(&mut mouse_sensitivity.0, 0.00001..=0.0002).text("Mouse Sensitivity"));

        let translation = global_transform.translation();
        ui.label(format!(
            "Chunk Pos (x = {}, y = {}, z = {})",
            grid_cell.x, grid_cell.y, grid_cell.z
        ));
        ui.label(format!(
            "Local Pos (x = {}, y = {}, z = {})",
            translation.x, translation.y, translation.z
        ));

        if let Some(value) = diagnostics
            .get(FrameTimeDiagnosticsPlugin::FPS)
            .and_then(|fps| fps.smoothed())
        {
            ui.label(format!("FPS: {value:>4.0}"));
        }
    });
}
