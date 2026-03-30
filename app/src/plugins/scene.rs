use bevy::prelude::*;
use spaceflights_core::Seed;

use crate::AppConfigResource;

#[derive(Component)]
pub struct WorldRoot;

#[derive(Component)]
pub struct UiRoot;

#[derive(Resource, Debug, Clone, Copy)]
pub struct CurrentSeed(pub Seed);

#[derive(Resource, Debug, Clone, Copy)]
pub struct SceneRoots {
    pub world_root: Entity,
    pub ui_root: Entity,
}

pub fn bootstrap_scene(mut commands: Commands, config: Res<AppConfigResource>) {
    let world_root = commands.spawn((WorldRoot, Name::new("WorldRoot"))).id();
    let ui_root = commands.spawn((UiRoot, Name::new("UiRoot"))).id();

    commands.insert_resource(SceneRoots {
        world_root,
        ui_root,
    });
    commands.insert_resource(CurrentSeed(Seed::new(config.0.seed.initial_seed)));
}
