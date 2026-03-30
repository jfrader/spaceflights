use crate::Seed;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGameplaySnapshot {
    pub user_id: String,
    pub sessions_completed: u64,
    pub total_runtime_seconds: u64,
    pub last_seed: Option<Seed>,
    pub updated_unix_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceError {
    BackendUnavailable(String),
    Storage(String),
    CorruptedData(String),
}

impl std::fmt::Display for PersistenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BackendUnavailable(message) => write!(f, "backend unavailable: {message}"),
            Self::Storage(message) => write!(f, "storage error: {message}"),
            Self::CorruptedData(message) => write!(f, "corrupted data: {message}"),
        }
    }
}

impl std::error::Error for PersistenceError {}

pub trait GameplayPersistenceStrategy {
    /// Returns a stable backend identifier for diagnostics and telemetry.
    fn backend_name(&self) -> &'static str;

    /// Ensures backend-specific storage schema exists.
    ///
    /// # Errors
    /// Returns [`PersistenceError`] when schema initialization fails.
    fn init_schema(&mut self) -> Result<(), PersistenceError>;

    /// Loads the latest snapshot for a user if present.
    ///
    /// # Errors
    /// Returns [`PersistenceError`] when storage access or decoding fails.
    fn load_user_snapshot(
        &mut self,
        user_id: &str,
    ) -> Result<Option<UserGameplaySnapshot>, PersistenceError>;

    /// Persists a full snapshot for a user.
    ///
    /// # Errors
    /// Returns [`PersistenceError`] when write or serialization fails.
    fn save_user_snapshot(
        &mut self,
        snapshot: &UserGameplaySnapshot,
    ) -> Result<(), PersistenceError>;
}
