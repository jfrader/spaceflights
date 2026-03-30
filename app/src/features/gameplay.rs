use bevy::prelude::*;
use spaceflights_core::{step, ControlMode, GameCommand, GameplayState, GameplayTuning};

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
            .add_systems(Update, collect_gameplay_input.in_set(GameSet::Input))
            .add_systems(FixedUpdate, run_gameplay_step.in_set(GameSet::Gameplay));
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
    mut queue: ResMut<GameplayCommandQueue>,
) {
    let Some(keyboard) = keyboard else {
        return;
    };

    if keyboard.just_pressed(KeyCode::BracketRight) {
        queue.push(GameCommand::IncreaseShipSpeed);
    }
    if keyboard.just_pressed(KeyCode::BracketLeft) {
        queue.push(GameCommand::DecreaseShipSpeed);
    }

    if keyboard.just_pressed(KeyCode::Digit1) {
        queue.push(GameCommand::SetMode(ControlMode::Ship));
    }
    if keyboard.just_pressed(KeyCode::Digit2) {
        queue.push(GameCommand::SetMode(ControlMode::Module));
    }
    if keyboard.just_pressed(KeyCode::Digit3) {
        queue.push(GameCommand::SetMode(ControlMode::Eva));
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
