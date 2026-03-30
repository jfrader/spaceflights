use bevy::prelude::*;

use crate::features::{
    camera::CameraFeaturePlugin, debug::DebugFeaturePlugin, gameplay::GameplayFeaturePlugin,
    input::InputFeaturePlugin, music::MusicFeaturePlugin, ui::UiFeaturePlugin,
    world::WorldFeaturePlugin,
};
use crate::EnabledFeatures;

use super::{
    persistence::PersistencePlugin,
    scene::{bootstrap_scene, CurrentSeed, SceneRoots},
    schedule::GameSet,
};

pub struct GamePlugin {
    enabled: EnabledFeatures,
}

impl GamePlugin {
    #[must_use]
    pub const fn new(enabled: EnabledFeatures) -> Self {
        Self { enabled }
    }
}

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.configure_sets(
            Update,
            (
                GameSet::Persistence,
                GameSet::Input,
                GameSet::Gameplay,
                GameSet::Music,
                GameSet::World,
                GameSet::Camera,
                GameSet::Ui,
                GameSet::Debug,
            )
                .chain(),
        );

        app.configure_sets(
            FixedUpdate,
            (
                GameSet::Persistence,
                GameSet::Input,
                GameSet::Gameplay,
                GameSet::Music,
                GameSet::World,
                GameSet::Camera,
                GameSet::Ui,
                GameSet::Debug,
            )
                .chain(),
        );

        app.add_systems(Startup, bootstrap_scene);
        app.add_plugins(PersistencePlugin);

        if self.enabled.input {
            app.add_plugins(InputFeaturePlugin);
        }
        app.add_plugins(GameplayFeaturePlugin);
        app.add_plugins(MusicFeaturePlugin);
        if self.enabled.world {
            app.add_plugins(WorldFeaturePlugin);
        }
        if self.enabled.camera {
            app.add_plugins(CameraFeaturePlugin);
        }
        if self.enabled.ui {
            app.add_plugins(UiFeaturePlugin);
        }
        if self.enabled.debug {
            app.add_plugins(DebugFeaturePlugin);
        }

        app.add_systems(
            Update,
            sanity_check_foundation_resources.in_set(GameSet::Debug),
        );
    }
}

fn sanity_check_foundation_resources(seed: Res<CurrentSeed>, roots: Res<SceneRoots>) {
    std::hint::black_box(seed.0.value());
    std::hint::black_box(roots.world_root);
    std::hint::black_box(roots.ui_root);
}
