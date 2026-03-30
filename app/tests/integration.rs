use bevy::prelude::*;

use spaceflights_app::features::world::WorldRenderStats;
use spaceflights_app::plugins::persistence::PersistenceHealth;
use spaceflights_app::plugins::schedule::GameSet;
use spaceflights_app::{build_app, build_gameplay_debug_app, EnabledFeatures, RuntimeMode};
use spaceflights_core::{AppConfig, PersistenceBackend};

#[derive(Resource, Default, Debug)]
struct ExecutionTrace(Vec<&'static str>);

#[test]
fn app_boots_with_all_plugins() {
    let mut app = build_app(
        AppConfig::default(),
        RuntimeMode::Headless,
        EnabledFeatures::default(),
    );
    app.update();
}

#[test]
fn schedule_order_is_deterministic() {
    let mut app = build_app(
        AppConfig::default(),
        RuntimeMode::Headless,
        EnabledFeatures::default(),
    );
    app.insert_resource(ExecutionTrace::default());

    app.add_systems(Update, marker_persistence.in_set(GameSet::Persistence));
    app.add_systems(Update, marker_input.in_set(GameSet::Input));
    app.add_systems(Update, marker_gameplay.in_set(GameSet::Gameplay));
    app.add_systems(Update, marker_music.in_set(GameSet::Music));
    app.add_systems(Update, marker_world.in_set(GameSet::World));
    app.add_systems(Update, marker_camera.in_set(GameSet::Camera));
    app.add_systems(Update, marker_ui.in_set(GameSet::Ui));
    app.add_systems(Update, marker_debug.in_set(GameSet::Debug));

    app.update();

    let trace = app.world().resource::<ExecutionTrace>();
    assert_eq!(
        trace.0,
        vec![
            "persistence",
            "input",
            "gameplay",
            "music",
            "camera",
            "world",
            "ui",
            "debug",
        ]
    );
}

#[test]
fn features_can_be_disabled_without_breaking_boot() {
    let enabled = EnabledFeatures {
        camera: false,
        ui: false,
        ..EnabledFeatures::default()
    };

    let mut app = build_app(AppConfig::default(), RuntimeMode::Headless, enabled);
    app.update();
}

#[test]
fn sqlite_backend_initializes_persistence() {
    let mut app = build_app(
        AppConfig::default(),
        RuntimeMode::Headless,
        EnabledFeatures::default(),
    );
    app.update();

    let health = app.world().resource::<PersistenceHealth>();
    assert_eq!(health.backend_name, "sqlite");
    assert!(health.initialized);
    assert!(health.error.is_none());
}

#[test]
fn spacetimedb_backend_can_be_selected_without_crashing() {
    let mut config = AppConfig::default();
    config.persistence.backend = PersistenceBackend::SpaceTimeDb;

    let mut app = build_app(config, RuntimeMode::Headless, EnabledFeatures::default());
    app.update();

    let health = app.world().resource::<PersistenceHealth>();
    assert_eq!(health.backend_name, "spacetimedb");
    assert!(!health.initialized);
    assert!(health.error.is_some());
}

#[test]
fn gameplay_debug_build_boots_headless() {
    let mut app = build_gameplay_debug_app(AppConfig::default(), RuntimeMode::Headless);
    app.update();
}

#[test]
fn world_resources_initialize_in_main_and_debug_builds() {
    let mut main = build_app(
        AppConfig::default(),
        RuntimeMode::Headless,
        EnabledFeatures::default(),
    );
    main.update();
    let main_stats = main.world().resource::<WorldRenderStats>();
    assert!(main_stats.star_count > 0);
    assert!(main_stats.debris_count > 0);

    let mut debug = build_gameplay_debug_app(AppConfig::default(), RuntimeMode::Headless);
    debug.update();
    let debug_stats = debug.world().resource::<WorldRenderStats>();
    assert!(debug_stats.star_count > 0);
    assert!(debug_stats.debris_count > 0);
}

#[test]
fn world_counts_remain_bounded_over_many_updates() {
    let mut app = build_gameplay_debug_app(AppConfig::default(), RuntimeMode::Headless);
    app.update();
    let initial = app.world().resource::<WorldRenderStats>().clone();

    for _ in 0..90 {
        app.update();
    }

    let after = app.world().resource::<WorldRenderStats>().clone();
    assert_eq!(initial.star_count, after.star_count);
    assert_eq!(initial.debris_count, after.debris_count);
}

fn marker_input(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("input");
}

fn marker_gameplay(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("gameplay");
}

fn marker_music(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("music");
}

fn marker_persistence(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("persistence");
}

fn marker_world(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("world");
}

fn marker_camera(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("camera");
}

fn marker_ui(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("ui");
}

fn marker_debug(mut trace: ResMut<ExecutionTrace>) {
    trace.0.push("debug");
}
