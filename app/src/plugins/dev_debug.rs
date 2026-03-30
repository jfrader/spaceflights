use bevy::prelude::*;
use spaceflights_core::ControlMode;

use crate::features::camera::MouseLookCamera;
use crate::features::gameplay::GameplayRuntime;

const KM_TO_WORLD: f32 = 0.006;
const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 3.0, 10.0);

pub struct DevGameplayShellPlugin;

#[derive(Component)]
struct DevDebugPlayer;

#[derive(Resource, Debug, Clone, Copy)]
struct DevDebugBaseDistance {
    km: f64,
}

#[derive(Resource, Debug, Default)]
struct DevDebugInitialized {
    done: bool,
}

impl Plugin for DevGameplayShellPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(DevDebugInitialized::default())
            .add_systems(Startup, spawn_dev_debug_scene)
            .add_systems(Update, initialize_mid_game_state)
            .add_systems(Update, sync_player_transform)
            .add_systems(Update, follow_player_camera);
    }
}

fn spawn_dev_debug_scene(
    mut commands: Commands,
    _meshes: Option<ResMut<Assets<Mesh>>>,
    _materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    commands.spawn((
        Camera3dBundle {
            projection: Projection::Perspective(PerspectiveProjection {
                near: 0.08,
                far: 8_000.0,
                ..Default::default()
            }),
            transform: Transform::from_xyz(0.0, 3.0, 10.0),
            ..Default::default()
        },
        MouseLookCamera,
        Name::new("DevDebugCamera"),
    ));

    let player = commands
        .spawn((
            DevDebugPlayer,
            SpatialBundle::from_transform(Transform::from_xyz(0.0, 0.0, 0.0)),
            Name::new("DevDebugPlayer"),
        ))
        .id();

    std::hint::black_box(player);
}

fn initialize_mid_game_state(
    mut runtime: Option<ResMut<GameplayRuntime>>,
    mut initialized: ResMut<DevDebugInitialized>,
    mut commands: Commands,
) {
    if initialized.done {
        return;
    }

    let Some(runtime) = runtime.as_mut() else {
        return;
    };

    runtime.state.mode = ControlMode::Module;
    runtime.state.ship_speed_km_s = 320.0;
    runtime.state.ship_distance_km = 2_000_000.0;
    runtime.state.mission_elapsed_seconds = 3600.0;

    commands.insert_resource(DevDebugBaseDistance {
        km: runtime.state.ship_distance_km,
    });

    initialized.done = true;
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Bevy Transform coordinates are f32."
)]
fn sync_player_transform(
    runtime: Option<Res<GameplayRuntime>>,
    base_distance: Option<Res<DevDebugBaseDistance>>,
    mut player_query: Query<&mut Transform, With<DevDebugPlayer>>,
) {
    let Some(runtime) = runtime else {
        return;
    };
    let Some(base_distance) = base_distance else {
        return;
    };

    let Ok(mut transform) = player_query.get_single_mut() else {
        return;
    };

    let traveled_km = runtime.state.ship_distance_km - base_distance.km;
    transform.translation.z = -(traveled_km as f32) * KM_TO_WORLD;
}

fn follow_player_camera(
    player_query: Query<&Transform, With<DevDebugPlayer>>,
    mut camera_query: Query<&mut Transform, (With<MouseLookCamera>, Without<DevDebugPlayer>)>,
) {
    let Ok(player_transform) = player_query.get_single() else {
        return;
    };

    let Ok(mut camera_transform) = camera_query.get_single_mut() else {
        return;
    };

    camera_transform.translation =
        player_transform.translation + camera_transform.rotation * CAMERA_OFFSET;
}
