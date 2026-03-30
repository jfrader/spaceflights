use spaceflights_core::{GameplayPersistenceStrategy, PersistenceError, UserGameplaySnapshot};

pub struct SpaceTimeDbGameplayPersistence {
    endpoint_uri: String,
}

impl SpaceTimeDbGameplayPersistence {
    #[must_use]
    pub fn new(endpoint_uri: &str) -> Self {
        Self {
            endpoint_uri: String::from(endpoint_uri),
        }
    }

    fn unavailable(&self) -> PersistenceError {
        PersistenceError::BackendUnavailable(format!(
            "SpaceTimeDB adapter is not implemented yet. endpoint={}",
            self.endpoint_uri
        ))
    }
}

impl GameplayPersistenceStrategy for SpaceTimeDbGameplayPersistence {
    fn backend_name(&self) -> &'static str {
        "spacetimedb"
    }

    fn init_schema(&mut self) -> Result<(), PersistenceError> {
        Err(self.unavailable())
    }

    fn load_user_snapshot(
        &mut self,
        _user_id: &str,
    ) -> Result<Option<UserGameplaySnapshot>, PersistenceError> {
        Err(self.unavailable())
    }

    fn save_user_snapshot(
        &mut self,
        _snapshot: &UserGameplaySnapshot,
    ) -> Result<(), PersistenceError> {
        Err(self.unavailable())
    }
}
