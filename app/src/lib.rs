#![allow(
    clippy::needless_pass_by_value,
    reason = "Bevy systems intentionally take ECS params by value."
)]
#![allow(
    clippy::struct_excessive_bools,
    reason = "Foundation feature toggles are explicit and readable."
)]
#![allow(
    clippy::cast_precision_loss,
    reason = "Bevy window resolution APIs use f32 dimensions."
)]

pub mod bootstrap;
pub mod features;
pub mod persistence;
pub mod plugins;

use bevy::prelude::*;
use spaceflights_core::AppConfig;

use features::camera::MouseLookPlugin;
use features::gameplay::GameplayFeaturePlugin;
use features::input::InputFeaturePlugin;
use features::music::MusicFeaturePlugin;
use features::ui::UiFeaturePlugin;
use features::world::WorldFeaturePlugin;
use plugins::dev_debug::DevGameplayShellPlugin;
use plugins::game::GamePlugin;
use plugins::schedule::GameSet;

#[derive(Resource, Debug, Clone)]
pub struct AppConfigResource(pub AppConfig);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BuildFlavor {
    Main,
    DevDebug,
}

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub struct BuildFlavorResource(pub BuildFlavor);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeMode {
    Desktop,
    Headless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EnabledFeatures {
    pub camera: bool,
    pub world: bool,
    pub ui: bool,
    pub input: bool,
    pub debug: bool,
}

impl Default for EnabledFeatures {
    fn default() -> Self {
        Self {
            camera: true,
            world: true,
            ui: true,
            input: true,
            debug: true,
        }
    }
}

#[must_use]
pub fn build_app(config: AppConfig, mode: RuntimeMode, enabled: EnabledFeatures) -> App {
    bootstrap::init_tracing(config.debug.enable_tracing);

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(Msaa::Sample4);
    app.insert_resource(AppConfigResource(config.clone()));
    app.insert_resource(BuildFlavorResource(BuildFlavor::Main));

    match mode {
        RuntimeMode::Desktop => {
            app.add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: config.window.title,
                    resolution: (config.window.width as f32, config.window.height as f32).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
        RuntimeMode::Headless => {
            app.add_plugins(MinimalPlugins);
        }
    }

    app.add_plugins(GamePlugin::new(enabled));
    app
}

#[must_use]
pub fn build_gameplay_debug_app(config: AppConfig, mode: RuntimeMode) -> App {
    bootstrap::init_tracing(config.debug.enable_tracing);
    let mut config = config;
    config.world.profile = spaceflights_core::WorldProfilePreset::DevDebug;

    let mut app = App::new();
    app.insert_resource(ClearColor(Color::BLACK));
    app.insert_resource(Msaa::Sample4);
    app.insert_resource(AppConfigResource(config.clone()));
    app.insert_resource(BuildFlavorResource(BuildFlavor::DevDebug));

    match mode {
        RuntimeMode::Desktop => {
            app.add_plugins(DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: format!("{} [DEV DEBUG]", config.window.title),
                    resolution: (config.window.width as f32, config.window.height as f32).into(),
                    ..Default::default()
                }),
                ..Default::default()
            }));
        }
        RuntimeMode::Headless => {
            app.add_plugins(MinimalPlugins);
        }
    }

    app.configure_sets(
        Update,
        (
            GameSet::Input,
            GameSet::Gameplay,
            GameSet::Music,
            GameSet::World,
            GameSet::Camera,
            GameSet::Ui,
        )
            .chain(),
    );
    app.configure_sets(
        FixedUpdate,
        (
            GameSet::Input,
            GameSet::Gameplay,
            GameSet::Music,
            GameSet::World,
            GameSet::Camera,
            GameSet::Ui,
        )
            .chain(),
    );

    app.add_plugins(InputFeaturePlugin);
    app.add_plugins(GameplayFeaturePlugin);
    app.add_plugins(MusicFeaturePlugin);
    app.add_plugins(WorldFeaturePlugin);
    app.add_plugins(MouseLookPlugin);
    app.add_plugins(UiFeaturePlugin);
    app.add_plugins(DevGameplayShellPlugin);
    app
}
