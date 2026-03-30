use bevy::prelude::*;
use spaceflights_core::{step, ControlMode, GameCommand, GameplayState, GameplayTuning};

use super::input::InputBindings;
use super::ui::MainMenuState;
use super::world::RequestWorldReset;
use crate::plugins::schedule::GameSet;

pub struct GameplayFeaturePlugin;

#[derive(Resource, Debug, Clone, Copy)]
pub struct GameplayRuntime {
    pub state: GameplayState,
    pub tuning: GameplayTuning,
}

#[derive(Resource, Debug, Default)]
pub struct GameplayCommandQueue {
    commands: Vec<GameCommand>,
}

impl GameplayCommandQueue {
    pub fn push(&mut self, command: GameCommand) {
        self.commands.push(command);
    }

    fn drain(&mut self) -> Vec<GameCommand> {
        self.commands.drain(..).collect()
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.commands.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.commands.is_empty()
    }
}

impl Plugin for GameplayFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_gameplay)
            .add_systems(
                Update,
                reset_gameplay_on_world_reset.in_set(GameSet::Gameplay),
            )
            .add_systems(FixedUpdate, collect_gameplay_input.in_set(GameSet::Input))
            .add_systems(
                FixedUpdate,
                run_gameplay_step
                    .in_set(GameSet::Gameplay)
                    .after(collect_gameplay_input),
            );
    }
}

fn initialize_gameplay(mut commands: Commands) {
    let tuning = GameplayTuning::default();
    let state = GameplayState::new(tuning);

    commands.insert_resource(GameplayRuntime { state, tuning });
    commands.insert_resource(GameplayCommandQueue::default());
}

fn collect_gameplay_input(
    keyboard: Option<Res<ButtonInput<KeyCode>>>,
    bindings: Res<InputBindings>,
    menu_state: Option<Res<MainMenuState>>,
    mut queue: ResMut<GameplayCommandQueue>,
) {
    let Some(keyboard) = keyboard else {
        return;
    };
    if menu_state.as_ref().is_some_and(|menu| menu.open) {
        return;
    }

    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.speed_up) {
        queue.push(GameCommand::IncreaseShipSpeed);
    }
    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.speed_down) {
        queue.push(GameCommand::DecreaseShipSpeed);
    }

    if bindings.is_just_pressed(&keyboard, bindings.controls.gameplay.mode_ship) {
        queue.push(GameCommand::SetMode(ControlMode::Ship));
    }
    if bindings.is_just_pressed(&keyboard, bindings.controls.gameplay.mode_module) {
        queue.push(GameCommand::SetMode(ControlMode::Module));
    }
    if bindings.is_just_pressed(&keyboard, bindings.controls.gameplay.mode_eva) {
        queue.push(GameCommand::SetMode(ControlMode::Eva));
    }

    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.yaw_left) {
        queue.push(GameCommand::YawLeft);
    }
    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.yaw_right) {
        queue.push(GameCommand::YawRight);
    }
    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.pitch_up) {
        queue.push(GameCommand::PitchUp);
    }
    if bindings.is_pressed(&keyboard, bindings.controls.gameplay.pitch_down) {
        queue.push(GameCommand::PitchDown);
    }
}

fn run_gameplay_step(
    time: Res<Time<Fixed>>,
    mut runtime: ResMut<GameplayRuntime>,
    mut queue: ResMut<GameplayCommandQueue>,
) {
    let commands = queue.drain();
    let tuning = runtime.tuning;
    step(
        &mut runtime.state,
        time.delta_seconds_f64(),
        &commands,
        tuning,
    );
}

fn reset_gameplay_on_world_reset(
    mut reset_events: EventReader<RequestWorldReset>,
    mut runtime: ResMut<GameplayRuntime>,
    mut queue: ResMut<GameplayCommandQueue>,
) {
    if reset_events.read().next().is_none() {
        return;
    }

    let tuning = runtime.tuning;
    runtime.state = GameplayState::new(tuning);
    queue.commands.clear();
}
