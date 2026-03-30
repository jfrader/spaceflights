use bevy::prelude::*;
use spaceflights_core::{ControlsConfig, KeyBindingCode};

use super::gameplay::GameplayCommandQueue;
use crate::plugins::schedule::GameSet;
use crate::AppConfigResource;

pub struct InputFeaturePlugin;

#[derive(Resource, Debug, Default)]
pub struct InputIntentBuffer {
    pub queued_intents: u32,
}

#[derive(Resource, Debug, Clone, Copy)]
pub struct InputBindings {
    pub controls: ControlsConfig,
}

impl InputBindings {
    #[must_use]
    pub fn is_pressed(&self, keyboard: &ButtonInput<KeyCode>, binding: KeyBindingCode) -> bool {
        keyboard.pressed(keycode_for(binding))
    }

    #[must_use]
    pub fn is_just_pressed(
        &self,
        keyboard: &ButtonInput<KeyCode>,
        binding: KeyBindingCode,
    ) -> bool {
        keyboard.just_pressed(keycode_for(binding))
    }
}

impl Plugin for InputFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputIntentBuffer>()
            .add_systems(Startup, initialize_input_bindings)
            .add_systems(Update, sync_input_intent_count.in_set(GameSet::Input));
    }
}

fn initialize_input_bindings(mut commands: Commands, config: Res<AppConfigResource>) {
    commands.insert_resource(InputBindings {
        controls: config.0.controls,
    });
}

pub fn sync_input_intent_count(
    gameplay_queue: Option<Res<GameplayCommandQueue>>,
    mut intents: ResMut<InputIntentBuffer>,
) {
    intents.queued_intents = gameplay_queue
        .as_ref()
        .map_or(0, |queue| u32::try_from(queue.len()).unwrap_or(u32::MAX));
}

#[must_use]
pub fn keycode_for(binding: KeyBindingCode) -> KeyCode {
    match binding {
        KeyBindingCode::BracketLeft => KeyCode::BracketLeft,
        KeyBindingCode::BracketRight => KeyCode::BracketRight,
        KeyBindingCode::Digit1 => KeyCode::Digit1,
        KeyBindingCode::Digit2 => KeyCode::Digit2,
        KeyBindingCode::Digit3 => KeyCode::Digit3,
        KeyBindingCode::KeyW => KeyCode::KeyW,
        KeyBindingCode::KeyA => KeyCode::KeyA,
        KeyBindingCode::KeyS => KeyCode::KeyS,
        KeyBindingCode::KeyD => KeyCode::KeyD,
        KeyBindingCode::ArrowLeft => KeyCode::ArrowLeft,
        KeyBindingCode::ArrowRight => KeyCode::ArrowRight,
        KeyBindingCode::ArrowUp => KeyCode::ArrowUp,
        KeyBindingCode::ArrowDown => KeyCode::ArrowDown,
        KeyBindingCode::Escape => KeyCode::Escape,
        KeyBindingCode::Enter => KeyCode::Enter,
        KeyBindingCode::KeyN => KeyCode::KeyN,
        KeyBindingCode::KeyR => KeyCode::KeyR,
    }
}
