use bevy::prelude::*;
use spaceflights_core::UserGameplaySnapshot;

use crate::persistence::{backend_label, build_persistence_strategy, PersistenceStrategyResource};
use crate::{plugins::schedule::GameSet, AppConfigResource};

pub struct PersistencePlugin;

#[derive(Resource, Debug, Clone)]
pub struct PersistenceHealth {
    pub backend_name: String,
    pub initialized: bool,
    pub error: Option<String>,
}

impl Plugin for PersistencePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_persistence)
            .add_systems(Update, persistence_heartbeat.in_set(GameSet::Persistence));
    }
}

fn initialize_persistence(world: &mut World) {
    let Some(config) = world.get_resource::<AppConfigResource>() else {
        world.insert_resource(PersistenceHealth {
            backend_name: String::from("unknown"),
            initialized: false,
            error: Some(String::from("AppConfigResource missing")),
        });
        return;
    };

    let backend = config.0.persistence.backend;
    let backend_name = backend_label(backend);

    match build_persistence_strategy(config) {
        Ok(mut strategy) => match strategy.init_schema() {
            Ok(()) => {
                let bootstrap_snapshot = UserGameplaySnapshot {
                    user_id: String::from("local-player"),
                    sessions_completed: 0,
                    total_runtime_seconds: 0,
                    last_seed: None,
                    updated_unix_seconds: 0,
                };

                let save_result = strategy.save_user_snapshot(&bootstrap_snapshot);
                let health = if let Err(error) = save_result {
                    PersistenceHealth {
                        backend_name: String::from(backend_name),
                        initialized: false,
                        error: Some(error.to_string()),
                    }
                } else {
                    PersistenceHealth {
                        backend_name: String::from(backend_name),
                        initialized: true,
                        error: None,
                    }
                };

                world.insert_non_send_resource(PersistenceStrategyResource { strategy });
                world.insert_resource(health);
            }
            Err(error) => {
                world.insert_resource(PersistenceHealth {
                    backend_name: String::from(backend_name),
                    initialized: false,
                    error: Some(error.to_string()),
                });
            }
        },
        Err(error) => {
            world.insert_resource(PersistenceHealth {
                backend_name: String::from(backend_name),
                initialized: false,
                error: Some(error.to_string()),
            });
        }
    }
}

fn persistence_heartbeat(
    mut strategy: Option<NonSendMut<PersistenceStrategyResource>>,
    health: Option<Res<PersistenceHealth>>,
) {
    let Some(health) = health else {
        return;
    };

    if !health.initialized {
        return;
    }

    if let Some(strategy) = strategy.as_mut() {
        let load_result = strategy.strategy.load_user_snapshot("local-player");
        if let Err(error) = load_result {
            bevy::log::warn!("persistence heartbeat load failed: {error}");
        }
    }
}
