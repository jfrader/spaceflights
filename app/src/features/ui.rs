use bevy::prelude::*;
use spaceflights_core::ControlMode;

use super::camera::MouseLookState;
use super::gameplay::GameplayRuntime;
use super::input::InputIntentBuffer;
use super::music::MusicRenderStats;
use super::world::WorldRenderStats;
use crate::plugins::persistence::PersistenceHealth;
use crate::plugins::schedule::GameSet;

pub struct UiFeaturePlugin;

#[derive(Resource, Debug, Clone)]
pub struct UiOverlayModel {
    pub engine_label: String,
    pub fps_placeholder: f32,
    pub mode_label: String,
    pub ship_speed_km_s: f64,
    pub ship_distance_km: f64,
    pub queued_inputs: u32,
    pub persistence_backend: String,
    pub seed_label: String,
    pub stars_count: u32,
    pub debris_count: u32,
    pub profile_label: String,
    pub camera_yaw_deg: f32,
    pub camera_pitch_deg: f32,
    pub music_backend: String,
    pub music_phrase_index: u64,
    pub music_notes_emitted: u64,
}

impl Default for UiOverlayModel {
    fn default() -> Self {
        Self {
            engine_label: String::from("bevy"),
            fps_placeholder: 0.0,
            mode_label: String::from("ship"),
            ship_speed_km_s: 0.0,
            ship_distance_km: 0.0,
            queued_inputs: 0,
            persistence_backend: String::from("unknown"),
            seed_label: String::from("unknown"),
            stars_count: 0,
            debris_count: 0,
            profile_label: String::from("unknown"),
            camera_yaw_deg: 0.0,
            camera_pitch_deg: 0.0,
            music_backend: String::from("unknown"),
            music_phrase_index: 0,
            music_notes_emitted: 0,
        }
    }
}

impl Plugin for UiFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiOverlayModel>().add_systems(
            Update,
            (maintain_ui_overlay_model, sync_ui_with_runtime).in_set(GameSet::Ui),
        );
    }
}

fn maintain_ui_overlay_model(mut overlay: ResMut<UiOverlayModel>) {
    overlay.fps_placeholder = overlay.fps_placeholder.max(0.0);
}

pub fn sync_ui_with_runtime(
    runtime: Option<Res<GameplayRuntime>>,
    intents: Option<Res<InputIntentBuffer>>,
    persistence: Option<Res<PersistenceHealth>>,
    world_stats: Option<Res<WorldRenderStats>>,
    mouse_look: Option<Res<MouseLookState>>,
    music_stats: Option<Res<MusicRenderStats>>,
    mut overlay: ResMut<UiOverlayModel>,
) {
    if let Some(runtime) = runtime {
        overlay.mode_label = String::from(mode_label(runtime.state.mode));
        overlay.ship_speed_km_s = runtime.state.ship_speed_km_s;
        overlay.ship_distance_km = runtime.state.ship_distance_km;
    }

    if let Some(intents) = intents {
        overlay.queued_inputs = intents.queued_intents;
    }

    if let Some(persistence) = persistence {
        overlay
            .persistence_backend
            .clone_from(&persistence.backend_name);
    }

    if let Some(world_stats) = world_stats {
        overlay.seed_label.clone_from(&world_stats.seed_label);
        overlay.profile_label.clone_from(&world_stats.profile_label);
        overlay.stars_count = world_stats.star_count;
        overlay.debris_count = world_stats.debris_count;
    }

    if let Some(mouse_look) = mouse_look {
        overlay.camera_yaw_deg = mouse_look.yaw.to_degrees();
        overlay.camera_pitch_deg = mouse_look.pitch.to_degrees();
    }

    if let Some(music_stats) = music_stats {
        overlay.music_backend.clone_from(&music_stats.backend_name);
        overlay.music_phrase_index = music_stats.phrase_index;
        overlay.music_notes_emitted = music_stats.notes_emitted;
    }
}

fn mode_label(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Ship => "ship",
        ControlMode::Module => "module",
        ControlMode::Eva => "eva",
    }
}
