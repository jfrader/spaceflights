pub mod config;
pub mod events;
pub mod gameplay;
pub mod music;
pub mod persistence;
pub mod seed;
pub mod worldgen;

pub use config::{
    AppConfig, MouseLookConfig, MusicBackend, MusicConfig, PersistenceBackend, PersistenceConfig,
    WorldConfig,
};
pub use gameplay::{step, ControlMode, GameCommand, GameplayState, GameplayTuning};
pub use music::{generate_phrase, MusicProfile, NoteEvent, ScaleMode, Waveform};
pub use persistence::{GameplayPersistenceStrategy, PersistenceError, UserGameplaySnapshot};
pub use seed::Seed;
pub use worldgen::{
    generate_world, profile_from_preset, HorizonDebris, StarPoint, Vec3d, WorldProfile,
    WorldProfilePreset, WorldSnapshot,
};
