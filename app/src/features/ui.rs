use bevy::prelude::*;
use spaceflights_core::{ControlMode, KeyBindingCode, Seed};

use super::camera::MouseLookState;
use super::gameplay::GameplayRuntime;
use super::input::{InputBindings, InputIntentBuffer};
use super::music::MusicRenderStats;
use super::world::{RequestWorldReset, WorldRenderStats};
use crate::plugins::persistence::PersistenceHealth;
use crate::plugins::scene::{CurrentSeed, SceneRoots};
use crate::plugins::schedule::GameSet;

pub struct UiFeaturePlugin;

#[derive(Resource, Debug, Clone, Copy)]
pub struct MainMenuState {
    pub open: bool,
}

impl Default for MainMenuState {
    fn default() -> Self {
        Self { open: true }
    }
}

#[derive(Component)]
struct MainMenuPanel;

#[derive(Component)]
struct MainMenuText;

#[derive(Component)]
struct HudPanel;

#[derive(Component)]
struct HudText;

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
    pub menu_open: bool,
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
            menu_open: true,
        }
    }
}

impl Plugin for UiFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<UiOverlayModel>()
            .init_resource::<MainMenuState>()
            .add_systems(Startup, spawn_world_menu_ui)
            .add_systems(
                Update,
                (
                    maintain_ui_overlay_model,
                    handle_world_menu_input,
                    sync_main_menu_ui,
                    sync_hud_ui,
                    sync_ui_with_runtime,
                )
                    .in_set(GameSet::Ui),
            );
    }
}

fn maintain_ui_overlay_model(mut overlay: ResMut<UiOverlayModel>) {
    overlay.fps_placeholder = overlay.fps_placeholder.max(0.0);
}

fn spawn_world_menu_ui(mut commands: Commands, roots: Option<Res<SceneRoots>>) {
    let main_menu_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Percent(24.0),
                    left: Val::Percent(32.0),
                    width: Val::Px(560.0),
                    padding: UiRect::all(Val::Px(16.0)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                ..Default::default()
            },
            MainMenuPanel,
            Name::new("MainMenuPanel"),
        ))
        .id();

    let main_menu_text = commands
        .spawn((
            TextBundle::from_section(
                "SPACEFLIGHTS",
                TextStyle {
                    font: Handle::default(),
                    font_size: 18.0,
                    color: Color::srgb(0.90, 0.95, 1.0),
                },
            ),
            MainMenuText,
            Name::new("MainMenuText"),
        ))
        .id();

    let hud_panel = commands
        .spawn((
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    top: Val::Px(14.0),
                    left: Val::Px(14.0),
                    width: Val::Px(360.0),
                    padding: UiRect::all(Val::Px(10.0)),
                    flex_direction: FlexDirection::Column,
                    ..Default::default()
                },
                background_color: BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
                visibility: Visibility::Hidden,
                ..Default::default()
            },
            HudPanel,
            Name::new("HudPanel"),
        ))
        .id();

    let hud_text = commands
        .spawn((
            TextBundle::from_section(
                "",
                TextStyle {
                    font: Handle::default(),
                    font_size: 14.0,
                    color: Color::srgb(0.85, 0.90, 0.96),
                },
            ),
            HudText,
            Name::new("HudText"),
        ))
        .id();

    commands.entity(main_menu_panel).add_child(main_menu_text);
    commands.entity(hud_panel).add_child(hud_text);
    if let Some(roots) = roots {
        commands.entity(roots.ui_root).add_child(main_menu_panel);
        commands.entity(roots.ui_root).add_child(hud_panel);
    }
}

fn handle_world_menu_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Res<InputBindings>,
    mut menu: ResMut<MainMenuState>,
    current_seed: Option<Res<CurrentSeed>>,
    mut reset_writer: EventWriter<RequestWorldReset>,
) {
    let Some(keyboard) = keyboard else {
        return;
    };

    if menu.open {
        if bindings.is_just_pressed(&keyboard, bindings.controls.menu.confirm) {
            menu.open = false;
        }

        if bindings.is_just_pressed(&keyboard, bindings.controls.menu.new_world) {
            let new_seed = generate_runtime_seed();
            reset_writer.send(RequestWorldReset { seed: new_seed });
            menu.open = false;
        }

        if bindings.is_just_pressed(&keyboard, bindings.controls.menu.restart_world) {
            let seed = current_seed.map_or(Seed::new(0), |seed| seed.0);
            reset_writer.send(RequestWorldReset { seed });
            menu.open = false;
        }

        return;
    }

    if bindings.is_just_pressed(&keyboard, bindings.controls.menu.toggle) {
        menu.open = true;
    }
}

fn sync_main_menu_ui(
    menu: Res<MainMenuState>,
    bindings: Res<InputBindings>,
    runtime: Option<Res<GameplayRuntime>>,
    current_seed: Option<Res<CurrentSeed>>,
    mut panel_query: Query<&mut Visibility, With<MainMenuPanel>>,
    mut text_query: Query<&mut Text, With<MainMenuText>>,
) {
    if let Ok(mut visibility) = panel_query.get_single_mut() {
        *visibility = if menu.open {
            Visibility::Visible
        } else {
            Visibility::Hidden
        };
    }

    let speed = runtime.map_or(0.0, |runtime| runtime.state.ship_speed_km_s);
    let seed_label = current_seed.map_or(String::from("unknown"), |seed| seed.0.to_string());
    let toggle = binding_label(bindings.controls.menu.toggle);
    let confirm = binding_label(bindings.controls.menu.confirm);
    let new_world = binding_label(bindings.controls.menu.new_world);
    let restart = binding_label(bindings.controls.menu.restart_world);
    let speed_up = binding_label(bindings.controls.gameplay.speed_up);
    let speed_down = binding_label(bindings.controls.gameplay.speed_down);
    let mode_ship = binding_label(bindings.controls.gameplay.mode_ship);
    let mode_module = binding_label(bindings.controls.gameplay.mode_module);
    let mode_eva = binding_label(bindings.controls.gameplay.mode_eva);
    let yaw_left = binding_label(bindings.controls.gameplay.yaw_left);
    let yaw_right = binding_label(bindings.controls.gameplay.yaw_right);
    let pitch_up = binding_label(bindings.controls.gameplay.pitch_up);
    let pitch_down = binding_label(bindings.controls.gameplay.pitch_down);

    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!(
            "SPACEFLIGHTS\n\n\
             MAIN MENU\n\n\
             {confirm}: start\n\
             {new_world}: start new seeded world\n\
             {restart}: restart current seed\n\n\
             IN-GAME\n\
             {toggle}: open menu\n\
             {speed_down}/{speed_up}: speed\n\
             {mode_ship}/{mode_module}/{mode_eva}: mode\n\
             {yaw_left}/{yaw_right}/{pitch_up}/{pitch_down}: steer\n\n\
             Current seed: {seed_label}\n\
             Current speed: {speed:.1} km/s"
        );
    }
}

fn sync_hud_ui(
    menu: Res<MainMenuState>,
    overlay: Res<UiOverlayModel>,
    mut panel_query: Query<&mut Visibility, With<HudPanel>>,
    mut text_query: Query<&mut Text, With<HudText>>,
) {
    if let Ok(mut visibility) = panel_query.get_single_mut() {
        *visibility = if menu.open {
            Visibility::Hidden
        } else {
            Visibility::Visible
        };
    }

    if let Ok(mut text) = text_query.get_single_mut() {
        text.sections[0].value = format!(
            "MODE: {}\nSPEED: {:.1} km/s\nDIST: {:.0} km\nSEED: {}\nSTARS: {}",
            overlay.mode_label,
            overlay.ship_speed_km_s,
            overlay.ship_distance_km,
            overlay.seed_label,
            overlay.stars_count
        );
    }
}

#[allow(
    clippy::too_many_arguments,
    reason = "UI model sync intentionally aggregates multiple feature resources into one view model."
)]
pub fn sync_ui_with_runtime(
    runtime: Option<Res<GameplayRuntime>>,
    intents: Option<Res<InputIntentBuffer>>,
    persistence: Option<Res<PersistenceHealth>>,
    world_stats: Option<Res<WorldRenderStats>>,
    mouse_look: Option<Res<MouseLookState>>,
    music_stats: Option<Res<MusicRenderStats>>,
    menu_state: Res<MainMenuState>,
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

    overlay.menu_open = menu_state.open;
}

fn mode_label(mode: ControlMode) -> &'static str {
    match mode {
        ControlMode::Ship => "ship",
        ControlMode::Module => "module",
        ControlMode::Eva => "eva",
    }
}

fn generate_runtime_seed() -> Seed {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    Seed::from_text(&format!("runtime-seed-{now}"))
}

fn binding_label(binding: KeyBindingCode) -> &'static str {
    match binding {
        KeyBindingCode::BracketLeft => "[",
        KeyBindingCode::BracketRight => "]",
        KeyBindingCode::Digit1 => "1",
        KeyBindingCode::Digit2 => "2",
        KeyBindingCode::Digit3 => "3",
        KeyBindingCode::KeyW => "W",
        KeyBindingCode::KeyA => "A",
        KeyBindingCode::KeyS => "S",
        KeyBindingCode::KeyD => "D",
        KeyBindingCode::ArrowLeft => "Left",
        KeyBindingCode::ArrowRight => "Right",
        KeyBindingCode::ArrowUp => "Up",
        KeyBindingCode::ArrowDown => "Down",
        KeyBindingCode::Escape => "Esc",
        KeyBindingCode::Enter => "Enter",
        KeyBindingCode::KeyN => "N",
        KeyBindingCode::KeyR => "R",
    }
}
