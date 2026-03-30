use bevy::math::primitives::{Annulus, Circle, Cuboid, Sphere};
use bevy::prelude::*;
use bevy::render::view::NoFrustumCulling;
use spaceflights_core::{
    generate_world, profile_from_preset, HorizonDebris, StarPoint, WorldProfile, WorldSnapshot,
};

use crate::features::gameplay::GameplayRuntime;
use crate::plugins::scene::CurrentSeed;
use crate::plugins::schedule::GameSet;
use crate::{AppConfigResource, BuildFlavor, BuildFlavorResource};

pub struct WorldFeaturePlugin;

const FLYBY_COUNT: usize = 6;
const FLYBY_RANGE_Z: f32 = 300.0;
const FLYBY_RANGE_X: f32 = 160.0;
const FLYBY_RANGE_Y: f32 = 90.0;
const RENDER_DEBRIS_STRIDE: usize = 7;
const WORLD_RENDER_SCALE: f32 = 0.03;
const PLANET_RADIUS: f32 = 72.0;
const PLANET_RING_HALF_WIDTH: f32 = 0.7;
const PLANET_OCCLUDER_RADIUS_SCALE: f32 = 0.94;
const PLANET_BASE_X: f32 = 18.0;
const PLANET_BASE_Y: f32 = -22.0;
const PLANET_FLYBY_START_Z: f32 = -1_150.0;
const PLANET_FLYBY_DURATION_S: f32 = 12.0;
const PLANET_DEPART_SPEED: f32 = 120.0;
const STAR_BACKGROUND_RADIUS_MIN: f32 = 1_500.0;
const STAR_BACKGROUND_RADIUS_RANGE: f32 = 5_000.0;
const STAR_BACKGROUND_LAYER_RANGE: f32 = 380.0;
const DEBRIS_TRAVEL_SCALE: f64 = 0.08;

#[derive(Resource, Debug, Clone)]
pub struct WorldRuntime {
    pub snapshot: WorldSnapshot,
    pub dev_flyby_demo_enabled: bool,
}

#[derive(Resource, Debug, Clone)]
pub struct WorldRenderStats {
    pub seed_label: String,
    pub profile_label: String,
    pub star_count: u32,
    pub debris_count: u32,
}

#[derive(Component)]
struct StarVisual {
    index: usize,
}

#[derive(Component)]
struct DebrisVisual {
    index: usize,
}

#[derive(Component, Clone, Copy)]
struct FlybyVisual {
    style: FlybyStyle,
    base_position: Vec3,
    drift: Vec2,
    speed_km_s: f32,
    wobble_amplitude: f32,
    wobble_frequency: f32,
    wobble_phase: f32,
    spin_rate: Vec3,
}

#[derive(Clone, Copy)]
enum FlybyStyle {
    Asteroid,
    Comet,
}

#[derive(Component)]
struct PlanetOccluder;

#[derive(Component)]
struct PlanetBorder;

impl Plugin for WorldFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_world)
            .add_systems(Update, sync_world_visuals.in_set(GameSet::World));
    }
}

fn initialize_world(
    mut commands: Commands,
    config: Res<AppConfigResource>,
    build_flavor: Option<Res<BuildFlavorResource>>,
    current_seed: Option<Res<CurrentSeed>>,
    mut meshes: Option<ResMut<Assets<Mesh>>>,
    mut materials: Option<ResMut<Assets<StandardMaterial>>>,
) {
    let seed = current_seed
        .as_ref()
        .map_or(config.0.seed.initial_seed, |seed_resource| {
            seed_resource.0.value()
        });

    let profile = profile_from_preset(
        config.0.world.profile,
        config.0.world.star_density_percent,
        config.0.world.debris_density_percent,
    );

    let snapshot = generate_world(spaceflights_core::Seed::new(seed), profile);
    let dev_flyby_demo_enabled = build_flavor
        .as_ref()
        .is_some_and(|flavor| flavor.0 == BuildFlavor::DevDebug);
    let stats = WorldRenderStats {
        seed_label: format!("{seed:016x}"),
        profile_label: String::from(profile.label),
        star_count: u32::try_from(snapshot.stars.len()).unwrap_or(u32::MAX),
        debris_count: u32::try_from(snapshot.horizon_debris.len()).unwrap_or(u32::MAX),
    };

    if let (Some(mut meshes), Some(mut materials)) = (meshes.take(), materials.take()) {
        spawn_visuals_with_meshes(
            &mut commands,
            &snapshot,
            &mut meshes,
            &mut materials,
            dev_flyby_demo_enabled,
        );
    } else {
        spawn_headless_placeholders(&mut commands, &snapshot, dev_flyby_demo_enabled);
    }

    commands.insert_resource(WorldRuntime {
        snapshot,
        dev_flyby_demo_enabled,
    });
    commands.insert_resource(stats);
}

fn spawn_visuals_with_meshes(
    commands: &mut Commands,
    snapshot: &WorldSnapshot,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    dev_flyby_demo_enabled: bool,
) {
    let point_mesh = meshes.add(Mesh::from(Sphere::new(0.5)));
    let rod_mesh = meshes.add(Mesh::from(Cuboid::from_size(Vec3::ONE)));
    let planet_fill_mesh = meshes.add(Mesh::from(Circle::new(
        PLANET_RADIUS * PLANET_OCCLUDER_RADIUS_SCALE,
    )));
    let planet_ring_mesh = meshes.add(Mesh::from(Annulus::new(
        (PLANET_RADIUS - PLANET_RING_HALF_WIDTH).max(1.0),
        PLANET_RADIUS + PLANET_RING_HALF_WIDTH,
    )));

    let star_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(2.4, 2.4, 2.4),
        base_color: Color::srgb(1.0, 1.0, 1.0),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });

    let debris_materials = [
        materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.08, 0.10, 0.14),
            base_color: Color::srgb(0.50, 0.58, 0.68),
            unlit: true,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.14, 0.10, 0.06),
            base_color: Color::srgb(0.76, 0.62, 0.46),
            unlit: true,
            ..Default::default()
        }),
    ];

    let flyby_materials = [
        materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.10, 0.14, 0.20),
            base_color: Color::srgb(0.60, 0.72, 0.90),
            unlit: true,
            ..Default::default()
        }),
        materials.add(StandardMaterial {
            emissive: LinearRgba::rgb(0.16, 0.11, 0.07),
            base_color: Color::srgb(0.80, 0.66, 0.52),
            unlit: true,
            ..Default::default()
        }),
    ];

    for (index, star) in snapshot.stars.iter().enumerate() {
        commands.spawn((
            PbrBundle {
                mesh: point_mesh.clone(),
                material: star_material.clone(),
                transform: Transform {
                    translation: to_vec3(star.position),
                    scale: Vec3::splat(star_initial_scale(*star)),
                    ..Default::default()
                },
                ..Default::default()
            },
            NoFrustumCulling,
            StarVisual { index },
            Name::new(format!("Star-{index}")),
        ));
    }

    for (index, debris) in snapshot.horizon_debris.iter().enumerate() {
        if index % RENDER_DEBRIS_STRIDE != 0 {
            continue;
        }

        commands.spawn((
            PbrBundle {
                mesh: rod_mesh.clone(),
                material: debris_materials[index % debris_materials.len()].clone(),
                transform: Transform {
                    translation: to_vec3(debris.position),
                    rotation: debris_visual_rotation(index),
                    scale: debris_visual_scale(*debris),
                },
                ..Default::default()
            },
            DebrisVisual { index },
            Name::new(format!("Debris-{index}")),
        ));
    }

    if dev_flyby_demo_enabled {
        spawn_flyby_visuals(commands, snapshot, rod_mesh, flyby_materials);
    }
    spawn_planet_border(commands, planet_fill_mesh, planet_ring_mesh, materials);
}

fn spawn_headless_placeholders(
    commands: &mut Commands,
    snapshot: &WorldSnapshot,
    dev_flyby_demo_enabled: bool,
) {
    for (index, star) in snapshot.stars.iter().enumerate() {
        commands.spawn((
            SpatialBundle::from_transform(Transform::from_translation(to_vec3(star.position))),
            StarVisual { index },
            Name::new(format!("HeadlessStar-{index}")),
        ));
    }

    for (index, debris) in snapshot.horizon_debris.iter().enumerate() {
        if index % RENDER_DEBRIS_STRIDE != 0 {
            continue;
        }

        commands.spawn((
            SpatialBundle::from_transform(Transform {
                translation: to_vec3(debris.position),
                scale: Vec3::splat(debris.scale),
                ..Default::default()
            }),
            DebrisVisual { index },
            Name::new(format!("HeadlessDebris-{index}")),
        ));
    }

    if dev_flyby_demo_enabled {
        for index in 0..FLYBY_COUNT {
            let flyby = build_flyby_visual(snapshot.seed.value(), index);
            commands.spawn((
                SpatialBundle::from_transform(Transform::from_translation(flyby.base_position)),
                flyby,
                Name::new(format!("HeadlessFlyby-{index}")),
            ));
        }
    }

    spawn_headless_planet_border(commands);
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Bevy transforms are f32 while worldgen uses f64."
)]
fn to_vec3(vec: spaceflights_core::Vec3d) -> Vec3 {
    Vec3::new(
        vec.x as f32 * WORLD_RENDER_SCALE,
        vec.y as f32 * WORLD_RENDER_SCALE,
        vec.z as f32 * WORLD_RENDER_SCALE,
    )
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::too_many_arguments,
    clippy::type_complexity,
    reason = "Bevy transforms are f32 while worldgen uses f64."
)]
fn sync_world_visuals(
    time: Res<Time>,
    config: Res<AppConfigResource>,
    runtime: Res<WorldRuntime>,
    gameplay: Option<Res<GameplayRuntime>>,
    camera_query: Query<(&Camera, &Projection, &Transform), With<Camera3d>>,
    mut stars: Query<
        (&StarVisual, &mut Transform),
        (
            Without<DebrisVisual>,
            Without<FlybyVisual>,
            Without<PlanetBorder>,
            Without<PlanetOccluder>,
            Without<Camera3d>,
        ),
    >,
    mut debris: Query<
        (&DebrisVisual, &mut Transform),
        (
            Without<StarVisual>,
            Without<FlybyVisual>,
            Without<PlanetBorder>,
            Without<PlanetOccluder>,
            Without<Camera3d>,
        ),
    >,
    mut flybys: Query<
        (&FlybyVisual, &mut Transform),
        (
            Without<StarVisual>,
            Without<DebrisVisual>,
            Without<PlanetBorder>,
            Without<PlanetOccluder>,
            Without<Camera3d>,
        ),
    >,
    mut planet_occluders: Query<
        &mut Transform,
        (
            With<PlanetOccluder>,
            Without<PlanetBorder>,
            Without<Camera3d>,
        ),
    >,
    mut planet_borders: Query<
        &mut Transform,
        (
            With<PlanetBorder>,
            Without<PlanetOccluder>,
            Without<Camera3d>,
        ),
    >,
) {
    let traveled_km = gameplay
        .as_ref()
        .map_or(0.0, |gameplay| gameplay.state.ship_distance_km);
    let elapsed = time.elapsed_seconds_f64();
    let profile = runtime.snapshot.profile;
    let elapsed_f32 = elapsed as f32;
    let planet_center = if runtime.dev_flyby_demo_enabled {
        planet_center_for_travel(elapsed_f32)
    } else {
        planet_center_static()
    };
    let (camera_pos, camera_fov_rad, viewport_height) = camera_query.get_single().ok().map_or(
        (
            None,
            std::f32::consts::FRAC_PI_4,
            config.0.window.height as f32,
        ),
        |(camera, projection, transform)| {
            let fov = match projection {
                Projection::Perspective(perspective) => perspective.fov,
                Projection::Orthographic(_) => std::f32::consts::FRAC_PI_4,
            };
            let viewport_height = camera
                .logical_viewport_size()
                .map_or(config.0.window.height as f32, |size| size.y.max(1.0));

            (Some(transform.translation), fov, viewport_height)
        },
    );

    for (marker, mut transform) in &mut debris {
        let data = runtime.snapshot.horizon_debris[marker.index];
        transform.translation = world_position_for_debris(data, traveled_km, elapsed, profile);
    }

    for (flyby, mut transform) in &mut flybys {
        let position = world_position_for_flyby(*flyby, traveled_km as f32, elapsed as f32);
        transform.translation = position;
        let yaw = flyby.spin_rate.y * elapsed as f32;
        let pitch = flyby.spin_rate.x * elapsed as f32;
        let roll = flyby.spin_rate.z * elapsed as f32;
        transform.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll);
    }

    let to_camera = camera_pos.map_or(Vec3::Z, |camera| {
        (camera - planet_center).normalize_or_zero()
    });
    let planet_rotation = if to_camera.length_squared() > 0.0 {
        Quat::from_rotation_arc(Vec3::Z, to_camera)
    } else {
        Quat::IDENTITY
    };
    let star_center = camera_pos.unwrap_or(Vec3::ZERO);
    let seed_value = runtime.snapshot.seed.value();

    for (marker, mut transform) in &mut stars {
        let star = runtime.snapshot.stars[marker.index];
        let radius = star_background_radius(star, seed_value, marker.index);
        let world_units_per_pixel =
            world_units_per_pixel_at_depth(radius, camera_fov_rad, viewport_height.max(1.0));
        transform.translation =
            world_position_for_star_background(star, seed_value, marker.index, star_center);
        transform.scale = Vec3::splat(star_visual_scale_world(star, world_units_per_pixel));
    }

    for mut transform in &mut planet_occluders {
        transform.translation = planet_center;
        transform.rotation = planet_rotation;
        transform.scale = Vec3::ONE;
    }

    for mut transform in &mut planet_borders {
        transform.translation = planet_center + to_camera * 0.6;
        transform.rotation = planet_rotation;
        transform.scale = Vec3::ONE;
    }
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Background stars are render-only f32 positions."
)]
fn world_position_for_star_background(
    star: StarPoint,
    seed_value: u64,
    star_index: usize,
    center_world: Vec3,
) -> Vec3 {
    let direction = star_direction_from_seed_index(seed_value, star_index);
    let radius = star_background_radius(star, seed_value, star_index);
    center_world + direction * radius
}

fn world_position_for_debris(
    debris: HorizonDebris,
    traveled_km: f64,
    elapsed_seconds: f64,
    profile: WorldProfile,
) -> Vec3 {
    let moved_x = debris.position.x + debris.drift.x * elapsed_seconds * 0.6;
    let moved_y = debris.position.y + debris.drift.y * elapsed_seconds * 0.8;
    let moved_z = debris.position.z + debris.drift.z * elapsed_seconds * 0.25;

    let z = wrap_axis(
        moved_z + traveled_km * debris.parallax * DEBRIS_TRAVEL_SCALE,
        profile.world_extent_km,
    );
    let x = wrap_axis(moved_x, profile.world_extent_km);
    let y = wrap_axis(moved_y, profile.horizon_band_km.max(1.0));

    to_vec3(spaceflights_core::Vec3d { x, y, z })
}

fn wrap_axis(value: f64, extent: f64) -> f64 {
    let width = extent * 2.0;
    (value + extent).rem_euclid(width) - extent
}

fn spawn_flyby_visuals(
    commands: &mut Commands,
    snapshot: &WorldSnapshot,
    rod_mesh: Handle<Mesh>,
    flyby_materials: [Handle<StandardMaterial>; 2],
) {
    for index in 0..FLYBY_COUNT {
        let flyby = build_flyby_visual(snapshot.seed.value(), index);
        let (material, scale) = match flyby.style {
            FlybyStyle::Asteroid => (flyby_materials[1].clone(), Vec3::new(4.8, 0.12, 0.12)),
            FlybyStyle::Comet => (flyby_materials[0].clone(), Vec3::new(7.6, 0.08, 0.08)),
        };

        commands.spawn((
            PbrBundle {
                mesh: rod_mesh.clone(),
                material,
                transform: Transform {
                    translation: flyby.base_position,
                    scale,
                    ..Default::default()
                },
                ..Default::default()
            },
            flyby,
            Name::new(format!("Flyby-{index}")),
        ));
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Seed-derived f32 values are intentional for render-only flyby params."
)]
fn build_flyby_visual(seed_value: u64, index: usize) -> FlybyVisual {
    let seed = spaceflights_core::Seed::new(seed_value);
    let base = seed.nth_value(20_000 + index as u64 * 17);
    let style = if base & 1 == 0 {
        FlybyStyle::Asteroid
    } else {
        FlybyStyle::Comet
    };

    let base_x = map_hash(
        seed.nth_value(20_001 + index as u64 * 17),
        -FLYBY_RANGE_X,
        FLYBY_RANGE_X,
    );
    let base_y = map_hash(
        seed.nth_value(20_002 + index as u64 * 17),
        -FLYBY_RANGE_Y,
        FLYBY_RANGE_Y,
    );
    let base_z = map_hash(
        seed.nth_value(20_003 + index as u64 * 17),
        -FLYBY_RANGE_Z,
        FLYBY_RANGE_Z,
    );
    let speed_km_s = map_hash(seed.nth_value(20_004 + index as u64 * 17), 14.0, 32.0);
    let drift_x = map_hash(seed.nth_value(20_005 + index as u64 * 17), -0.9, 0.9);
    let drift_y = map_hash(seed.nth_value(20_006 + index as u64 * 17), -0.4, 0.4);
    let wobble_amplitude = map_hash(seed.nth_value(20_007 + index as u64 * 17), 0.0, 4.0);
    let wobble_frequency = map_hash(seed.nth_value(20_008 + index as u64 * 17), 0.10, 0.28);
    let wobble_phase = map_hash(
        seed.nth_value(20_009 + index as u64 * 17),
        0.0,
        std::f32::consts::TAU,
    );
    let spin_x = map_hash(seed.nth_value(20_010 + index as u64 * 17), -0.5, 0.5);
    let spin_y = map_hash(seed.nth_value(20_011 + index as u64 * 17), -0.5, 0.5);
    let spin_z = map_hash(seed.nth_value(20_012 + index as u64 * 17), -0.9, 0.9);

    FlybyVisual {
        style,
        base_position: Vec3::new(base_x, base_y, base_z),
        drift: Vec2::new(drift_x, drift_y),
        speed_km_s,
        wobble_amplitude,
        wobble_frequency,
        wobble_phase,
        spin_rate: Vec3::new(spin_x, spin_y, spin_z),
    }
}

fn world_position_for_flyby(flyby: FlybyVisual, traveled_km: f32, elapsed_seconds: f32) -> Vec3 {
    let wobble = (elapsed_seconds * flyby.wobble_frequency + flyby.wobble_phase).sin()
        * flyby.wobble_amplitude;
    let x = wrap_axis_f32(
        flyby.base_position.x + flyby.drift.x * elapsed_seconds + wobble,
        FLYBY_RANGE_X,
    );
    let y = wrap_axis_f32(
        flyby.base_position.y + flyby.drift.y * elapsed_seconds + wobble * 0.2,
        FLYBY_RANGE_Y,
    );
    let z_local = wrap_axis_f32(
        flyby.base_position.z - elapsed_seconds * flyby.speed_km_s,
        FLYBY_RANGE_Z,
    );

    let ship_z = -traveled_km * 0.006;
    Vec3::new(x, y, ship_z + z_local)
}

fn wrap_axis_f32(value: f32, extent: f32) -> f32 {
    let width = extent * 2.0;
    (value + extent).rem_euclid(width) - extent
}

fn spawn_planet_border(
    commands: &mut Commands,
    fill_mesh: Handle<Mesh>,
    ring_mesh: Handle<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let ring_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(0.12, 0.22, 0.34),
        base_color: Color::srgb(0.48, 0.66, 0.88),
        unlit: true,
        cull_mode: None,
        ..Default::default()
    });
    let center = planet_center_for_travel(0.0);
    let occluder_material = materials.add(StandardMaterial {
        emissive: LinearRgba::rgb(0.01, 0.01, 0.015),
        base_color: Color::srgb(0.01, 0.01, 0.015),
        unlit: true,
        ..Default::default()
    });

    commands.spawn((
        PbrBundle {
            mesh: fill_mesh,
            material: occluder_material,
            transform: Transform {
                translation: center,
                ..Default::default()
            },
            ..Default::default()
        },
        NoFrustumCulling,
        PlanetOccluder,
        Name::new("PlanetOccluder"),
    ));

    commands.spawn((
        PbrBundle {
            mesh: ring_mesh,
            material: ring_material,
            transform: Transform::from_translation(center),
            ..Default::default()
        },
        NoFrustumCulling,
        PlanetBorder,
        Name::new("PlanetBorder"),
    ));
}

fn spawn_headless_planet_border(commands: &mut Commands) {
    let center = planet_center_for_travel(0.0);

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_translation(center)),
        PlanetOccluder,
        Name::new("HeadlessPlanetOccluder"),
    ));

    commands.spawn((
        SpatialBundle::from_transform(Transform::from_translation(center)),
        PlanetBorder,
        Name::new("HeadlessPlanetBorder"),
    ));
}

fn star_visual_scale_world(star: StarPoint, world_units_per_pixel: f32) -> f32 {
    let pixel_size = (1.30 + star.brightness * 1.25).clamp(1.25, 2.65);
    (world_units_per_pixel * pixel_size).max(0.1)
}

fn star_initial_scale(star: StarPoint) -> f32 {
    (1.30 + star.brightness * 1.25).clamp(1.25, 2.65)
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Worldgen parallax range is normalized and intentionally projected to f32 for render math."
)]
fn star_background_radius(star: StarPoint, seed_value: u64, star_index: usize) -> f32 {
    let layer = (1.0 - star.parallax as f32).clamp(0.0, 1.0);
    let depth_hash =
        seed_value.rotate_left(11) ^ (star_index as u64).wrapping_mul(0x94d0_49bb_1331_11eb);
    let radial_t = hash_unit(depth_hash);
    let radius = STAR_BACKGROUND_RADIUS_MIN
        + radial_t * STAR_BACKGROUND_RADIUS_RANGE
        + layer * STAR_BACKGROUND_LAYER_RANGE;
    radius.max(STAR_BACKGROUND_RADIUS_MIN)
}

fn world_units_per_pixel_at_depth(depth: f32, fov_rad: f32, viewport_height: f32) -> f32 {
    (2.0 * depth * (fov_rad * 0.5).tan()) / viewport_height.max(1.0)
}

fn debris_visual_scale(debris: HorizonDebris) -> Vec3 {
    let base = debris.scale.max(0.35);
    Vec3::new(base * 1.4, 0.05, 0.05)
}

fn planet_center_for_travel(elapsed_seconds: f32) -> Vec3 {
    let p0 = Vec3::new(520.0, PLANET_BASE_Y + 7.0, PLANET_FLYBY_START_Z - 420.0);
    let p1 = Vec3::new(500.0, PLANET_BASE_Y + 6.0, -980.0);
    let p2 = Vec3::new(430.0, PLANET_BASE_Y - 2.0, -380.0);
    let p3 = Vec3::new(-620.0, PLANET_BASE_Y - 10.0, 920.0);

    if elapsed_seconds <= PLANET_FLYBY_DURATION_S {
        let t = (elapsed_seconds / PLANET_FLYBY_DURATION_S).clamp(0.0, 1.0);
        cubic_bezier(p0, p1, p2, p3, t)
    } else {
        let depart_dir = (p3 - p2).normalize_or_zero();
        let dt = elapsed_seconds - PLANET_FLYBY_DURATION_S;
        p3 + depart_dir * (dt * PLANET_DEPART_SPEED)
    }
}

fn planet_center_static() -> Vec3 {
    Vec3::new(PLANET_BASE_X, PLANET_BASE_Y, PLANET_FLYBY_START_Z)
}

fn cubic_bezier(p0: Vec3, p1: Vec3, p2: Vec3, p3: Vec3, t: f32) -> Vec3 {
    let u = 1.0 - t;
    let tt = t * t;
    let uu = u * u;
    let uuu = uu * u;
    let ttt = tt * t;
    p0 * uuu + p1 * (3.0 * uu * t) + p2 * (3.0 * u * tt) + p3 * ttt
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Seeded sky direction generation intentionally maps hashes into f32 unit vectors."
)]
fn star_direction_from_seed_index(seed_value: u64, star_index: usize) -> Vec3 {
    let idx = star_index as u64;
    let hash_a = seed_value ^ idx.wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let hash_b = seed_value.rotate_left(19) ^ idx.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let u = hash_unit(hash_a);
    let v = hash_unit(hash_b);
    let theta = u * std::f32::consts::TAU;
    let z = v * 2.0 - 1.0;
    let r = (1.0 - z * z).max(0.0).sqrt();
    Vec3::new(r * theta.cos(), r * theta.sin(), z).normalize_or_zero()
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Deterministic index hash is projected to f32 angles for render transforms."
)]
fn debris_visual_rotation(index: usize) -> Quat {
    let hash = hash_index(index);
    let yaw = hash_unit(hash ^ 0x9e37_79b9_7f4a_7c15) * std::f32::consts::TAU;
    let pitch = (hash_unit(hash ^ 0xa24b_aed4_963e_e407) - 0.5) * 0.7;
    let roll = (hash_unit(hash ^ 0x3c79_ac49_2ba7_b653) - 0.5) * 1.0;
    Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll)
}

fn hash_index(index: usize) -> u64 {
    let mut x = index as u64 + 0x9e37_79b9_7f4a_7c15;
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    reason = "Normalized render hash values are intentionally converted to f32."
)]
fn hash_unit(hash: u64) -> f32 {
    let unit = (hash >> 11) as f64 / ((1_u64 << 53) as f64);
    unit as f32
}

fn map_hash(hash: u64, min: f32, max: f32) -> f32 {
    min + (max - min) * hash_unit(hash)
}
