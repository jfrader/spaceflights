use bevy::prelude::*;

use super::gameplay::GameplayCommandQueue;
use crate::plugins::schedule::GameSet;

pub struct InputFeaturePlugin;

#[derive(Resource, Debug, Default)]
pub struct InputIntentBuffer {
    pub queued_intents: u32,
}

impl Plugin for InputFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputIntentBuffer>()
            .add_systems(Update, sync_input_intent_count.in_set(GameSet::Input));
    }
}

pub fn sync_input_intent_count(
    gameplay_queue: Option<Res<GameplayCommandQueue>>,
    mut intents: ResMut<InputIntentBuffer>,
) {
    intents.queued_intents = gameplay_queue
        .as_ref()
        .map_or(0, |queue| u32::try_from(queue.len()).unwrap_or(u32::MAX));
}
