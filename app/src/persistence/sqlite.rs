use std::convert::TryFrom;

use rusqlite::{params, Connection};
use spaceflights_core::{
    GameplayPersistenceStrategy, PersistenceError, Seed, UserGameplaySnapshot,
};

pub struct SqliteGameplayPersistence {
    connection: Connection,
}

impl SqliteGameplayPersistence {
    /// Creates a SQLite-backed persistence strategy.
    ///
    /// # Errors
    /// Returns [`PersistenceError`] when the `SQLite` connection cannot be opened.
    pub fn new(path: &str) -> Result<Self, PersistenceError> {
        let connection =
            Connection::open(path).map_err(|error| PersistenceError::Storage(error.to_string()))?;

        Ok(Self { connection })
    }
}

impl GameplayPersistenceStrategy for SqliteGameplayPersistence {
    fn backend_name(&self) -> &'static str {
        "sqlite"
    }

    fn init_schema(&mut self) -> Result<(), PersistenceError> {
        self.connection
            .execute_batch(
                "
                CREATE TABLE IF NOT EXISTS user_gameplay_snapshot (
                    user_id TEXT PRIMARY KEY,
                    sessions_completed INTEGER NOT NULL,
                    total_runtime_seconds INTEGER NOT NULL,
                    last_seed_hex TEXT,
                    updated_unix_seconds INTEGER NOT NULL
                );
                ",
            )
            .map_err(|error| PersistenceError::Storage(error.to_string()))
    }

    fn load_user_snapshot(
        &mut self,
        user_id: &str,
    ) -> Result<Option<UserGameplaySnapshot>, PersistenceError> {
        let mut statement = self
            .connection
            .prepare(
                "
                SELECT sessions_completed, total_runtime_seconds, last_seed_hex, updated_unix_seconds
                FROM user_gameplay_snapshot
                WHERE user_id = ?1
                ",
            )
            .map_err(|error| PersistenceError::Storage(error.to_string()))?;

        let snapshot_result = statement.query_row(params![user_id], |row| {
            let sessions_completed_i64: i64 = row.get(0)?;
            let total_runtime_seconds_i64: i64 = row.get(1)?;
            let last_seed_hex: Option<String> = row.get(2)?;
            let updated_unix_seconds_i64: i64 = row.get(3)?;

            let sessions_completed = u64::try_from(sessions_completed_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "sessions_completed is negative",
                    )),
                )
            })?;

            let total_runtime_seconds = u64::try_from(total_runtime_seconds_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "total_runtime_seconds is negative",
                    )),
                )
            })?;

            let updated_unix_seconds = u64::try_from(updated_unix_seconds_i64).map_err(|_| {
                rusqlite::Error::FromSqlConversionFailure(
                    8,
                    rusqlite::types::Type::Integer,
                    Box::new(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "updated_unix_seconds is negative",
                    )),
                )
            })?;

            let last_seed = match last_seed_hex {
                Some(value) => {
                    let parsed = u64::from_str_radix(&value, 16).map_err(|error| {
                        rusqlite::Error::FromSqlConversionFailure(
                            0,
                            rusqlite::types::Type::Text,
                            Box::new(error),
                        )
                    })?;
                    Some(Seed::new(parsed))
                }
                None => None,
            };

            Ok(UserGameplaySnapshot {
                user_id: String::from(user_id),
                sessions_completed,
                total_runtime_seconds,
                last_seed,
                updated_unix_seconds,
            })
        });

        match snapshot_result {
            Ok(value) => Ok(Some(value)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(error) => Err(PersistenceError::Storage(error.to_string())),
        }
    }

    fn save_user_snapshot(
        &mut self,
        snapshot: &UserGameplaySnapshot,
    ) -> Result<(), PersistenceError> {
        let sessions_completed = i64::try_from(snapshot.sessions_completed).map_err(|_| {
            PersistenceError::CorruptedData(String::from("sessions_completed overflow"))
        })?;
        let total_runtime_seconds =
            i64::try_from(snapshot.total_runtime_seconds).map_err(|_| {
                PersistenceError::CorruptedData(String::from("total_runtime_seconds overflow"))
            })?;
        let updated_unix_seconds = i64::try_from(snapshot.updated_unix_seconds).map_err(|_| {
            PersistenceError::CorruptedData(String::from("updated_unix_seconds overflow"))
        })?;
        let last_seed_hex = snapshot
            .last_seed
            .map(|seed| format!("{:016x}", seed.value()));

        self.connection
            .execute(
                "
                INSERT INTO user_gameplay_snapshot (
                    user_id,
                    sessions_completed,
                    total_runtime_seconds,
                    last_seed_hex,
                    updated_unix_seconds
                ) VALUES (?1, ?2, ?3, ?4, ?5)
                ON CONFLICT(user_id)
                DO UPDATE SET
                    sessions_completed = excluded.sessions_completed,
                    total_runtime_seconds = excluded.total_runtime_seconds,
                    last_seed_hex = excluded.last_seed_hex,
                    updated_unix_seconds = excluded.updated_unix_seconds
                ",
                params![
                    &snapshot.user_id,
                    sessions_completed,
                    total_runtime_seconds,
                    last_seed_hex,
                    updated_unix_seconds
                ],
            )
            .map(|_| ())
            .map_err(|error| PersistenceError::Storage(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use spaceflights_core::{GameplayPersistenceStrategy, Seed, UserGameplaySnapshot};

    use super::SqliteGameplayPersistence;

    #[test]
    fn sqlite_strategy_round_trip() {
        let mut strategy = match SqliteGameplayPersistence::new(":memory:") {
            Ok(value) => value,
            Err(error) => panic!("sqlite in-memory creation failed: {error}"),
        };
        if let Err(error) = strategy.init_schema() {
            panic!("schema init failed: {error}");
        }

        let snapshot = UserGameplaySnapshot {
            user_id: String::from("user-1"),
            sessions_completed: 3,
            total_runtime_seconds: 245,
            last_seed: Some(Seed::new(42)),
            updated_unix_seconds: 1_716_000_000,
        };

        if let Err(error) = strategy.save_user_snapshot(&snapshot) {
            panic!("save failed: {error}");
        }
        let loaded = match strategy.load_user_snapshot("user-1") {
            Ok(Some(value)) => value,
            Ok(None) => panic!("expected persisted snapshot"),
            Err(error) => panic!("load failed: {error}"),
        };

        assert_eq!(loaded, snapshot);
    }
}
