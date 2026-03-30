use bevy::diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;

use super::ui::UiOverlayModel;
use crate::plugins::schedule::GameSet;

pub struct DebugFeaturePlugin;

#[derive(Resource, Debug, Default)]
pub struct DebugOverlayState {
    pub engine_version: String,
    pub fps: f64,
    pub seed_label: String,
    pub stars_count: u32,
    pub debris_count: u32,
    pub camera_yaw_deg: f32,
    pub camera_pitch_deg: f32,
}

impl Plugin for DebugFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(FrameTimeDiagnosticsPlugin)
            .init_resource::<DebugOverlayState>()
            .add_systems(Startup, init_debug_overlay_state)
            .add_systems(Update, sample_fps.in_set(GameSet::Debug))
            .add_systems(Update, sync_debug_from_ui.in_set(GameSet::Debug));
    }
}

fn init_debug_overlay_state(mut overlay: ResMut<DebugOverlayState>) {
    overlay.engine_version = format!("bevy {}", env!("CARGO_PKG_VERSION"));
    overlay.seed_label = String::from("unknown");
}

fn sample_fps(mut overlay: ResMut<DebugOverlayState>, diagnostics: Res<DiagnosticsStore>) {
    overlay.fps = diagnostics
        .get(&FrameTimeDiagnosticsPlugin::FPS)
        .and_then(bevy::diagnostic::Diagnostic::smoothed)
        .unwrap_or(0.0);
}

fn sync_debug_from_ui(ui: Option<Res<UiOverlayModel>>, mut debug: ResMut<DebugOverlayState>) {
    let Some(ui) = ui else {
        return;
    };

    debug.seed_label.clone_from(&ui.seed_label);
    debug.stars_count = ui.stars_count;
    debug.debris_count = ui.debris_count;
    debug.camera_yaw_deg = ui.camera_yaw_deg;
    debug.camera_pitch_deg = ui.camera_pitch_deg;
}
