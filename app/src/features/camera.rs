use bevy::core_pipeline::fxaa::{Fxaa, Sensitivity};
use bevy::core_pipeline::tonemapping::{DebandDither, Tonemapping};
use bevy::ecs::event::ManualEventReader;
use bevy::input::mouse::MouseMotion;
use bevy::prelude::*;

use super::gameplay::GameplayRuntime;
use crate::plugins::schedule::GameSet;
use crate::AppConfigResource;

pub struct CameraFeaturePlugin;
pub struct MouseLookPlugin;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CameraFlowSet {
    MouseLook,
    Follow,
}

#[derive(Component)]
pub struct FoundationCamera;

#[derive(Component)]
pub struct MouseLookCamera;

const KM_TO_WORLD: f32 = 0.03;
const CAMERA_FOLLOW_OFFSET: Vec3 = Vec3::new(0.0, 2.2, 8.5);

#[derive(Resource, Debug, Clone, Copy)]
pub struct MouseLookState {
    pub yaw: f32,
    pub pitch: f32,
    pub dragging: bool,
    pub sensitivity: f32,
    pub pitch_limit_rad: f32,
}

impl Default for MouseLookState {
    fn default() -> Self {
        Self {
            yaw: 0.0,
            pitch: 0.0,
            dragging: false,
            sensitivity: 0.0022,
            pitch_limit_rad: 80.0_f32.to_radians(),
        }
    }
}

impl Plugin for CameraFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(MouseLookPlugin)
            .add_systems(Startup, spawn_foundation_camera)
            .add_systems(Update, maintain_camera_scaffold.in_set(GameSet::Camera))
            .add_systems(
                Update,
                follow_gameplay_camera
                    .in_set(GameSet::Camera)
                    .in_set(CameraFlowSet::Follow),
            );
    }
}

impl Plugin for MouseLookPlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (CameraFlowSet::MouseLook, CameraFlowSet::Follow).chain(),
        )
        .add_systems(Startup, initialize_mouse_look_state)
        .add_systems(
            Update,
            apply_mouse_look
                .in_set(GameSet::Camera)
                .in_set(CameraFlowSet::MouseLook),
        );
    }
}

fn initialize_mouse_look_state(mut commands: Commands, config: Res<AppConfigResource>) {
    commands.insert_resource(MouseLookState {
        sensitivity: config.0.mouse_look.sensitivity,
        pitch_limit_rad: config.0.mouse_look.pitch_limit_deg.to_radians(),
        ..MouseLookState::default()
    });
}

fn spawn_foundation_camera(mut commands: Commands) {
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.08,
                far: 8_000.0,
                ..Default::default()
            }),
            tonemapping: Tonemapping::None,
            deband_dither: DebandDither::Disabled,
            transform: Transform::from_xyz(0.0, 2.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..Default::default()
        },
        Fxaa {
            enabled: true,
            edge_threshold: Sensitivity::High,
            edge_threshold_min: Sensitivity::Medium,
        },
        FoundationCamera,
        MouseLookCamera,
        Name::new("FoundationCamera"),
    ));
}

fn maintain_camera_scaffold(query: Query<&Transform, With<FoundationCamera>>) {
    std::hint::black_box(query.get_single().ok());
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Bevy Transform coordinates are f32."
)]
fn follow_gameplay_camera(
    gameplay: Option<Res<GameplayRuntime>>,
    mut cameras: Query<&mut Transform, (With<FoundationCamera>, With<MouseLookCamera>)>,
) {
    let Some(gameplay) = gameplay else {
        return;
    };
    let Ok(mut camera_transform) = cameras.get_single_mut() else {
        return;
    };

    let ship_world = Vec3::new(
        (gameplay.state.ship_position_km.x as f32) * KM_TO_WORLD,
        (gameplay.state.ship_position_km.y as f32) * KM_TO_WORLD,
        (gameplay.state.ship_position_km.z as f32) * KM_TO_WORLD,
    );
    camera_transform.translation = ship_world + camera_transform.rotation * CAMERA_FOLLOW_OFFSET;
}

fn apply_mouse_look(
    mut state: ResMut<MouseLookState>,
    mouse_buttons: Option<Res<ButtonInput<MouseButton>>>,
    motions: Option<Res<Events<MouseMotion>>>,
    mut motion_reader: Local<ManualEventReader<MouseMotion>>,
    mut cameras: Query<&mut Transform, With<MouseLookCamera>>,
) {
    let Some(mouse_buttons) = mouse_buttons else {
        return;
    };

    state.dragging = mouse_buttons.pressed(MouseButton::Left);

    let Some(motions) = motions else {
        return;
    };

    let mut delta = Vec2::ZERO;
    for event in motion_reader.read(motions.as_ref()) {
        if state.dragging {
            delta += event.delta;
        }
    }

    if delta == Vec2::ZERO {
        return;
    }

    state.yaw -= delta.x * state.sensitivity;
    state.pitch -= delta.y * state.sensitivity;
    state.pitch = state
        .pitch
        .clamp(-state.pitch_limit_rad, state.pitch_limit_rad);

    let rotation = Quat::from_euler(EulerRot::YXZ, state.yaw, state.pitch, 0.0);
    for mut camera in &mut cameras {
        camera.rotation = rotation;
    }
}
