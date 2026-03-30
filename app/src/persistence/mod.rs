mod spacetimedb;
mod sqlite;

use spaceflights_core::{GameplayPersistenceStrategy, PersistenceBackend, PersistenceError};

use crate::AppConfigResource;

pub use spacetimedb::SpaceTimeDbGameplayPersistence;
pub use sqlite::SqliteGameplayPersistence;

pub struct PersistenceStrategyResource {
    pub strategy: Box<dyn GameplayPersistenceStrategy>,
}

#[must_use]
pub fn backend_label(backend: PersistenceBackend) -> &'static str {
    match backend {
        PersistenceBackend::Sqlite => "sqlite",
        PersistenceBackend::SpaceTimeDb => "spacetimedb",
    }
}

/// Builds a persistence strategy adapter from configuration.
///
/// # Errors
/// Returns [`PersistenceError`] when adapter construction fails.
pub fn build_persistence_strategy(
    config: &AppConfigResource,
) -> Result<Box<dyn GameplayPersistenceStrategy>, PersistenceError> {
    match config.0.persistence.backend {
        PersistenceBackend::Sqlite => {
            let strategy = SqliteGameplayPersistence::new(&config.0.persistence.sqlite_path)?;
            Ok(Box::new(strategy))
        }
        PersistenceBackend::SpaceTimeDb => {
            let strategy =
                SpaceTimeDbGameplayPersistence::new(&config.0.persistence.spacetimedb_uri);
            Ok(Box::new(strategy))
        }
    }
}
