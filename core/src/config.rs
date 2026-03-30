use crate::music::ScaleMode;
use crate::worldgen::WorldProfilePreset;

#[derive(Debug, Clone, PartialEq)]
pub struct AppConfig {
    pub window: WindowConfig,
    pub debug: DebugConfig,
    pub seed: SeedConfig,
    pub persistence: PersistenceConfig,
    pub world: WorldConfig,
    pub mouse_look: MouseLookConfig,
    pub music: MusicConfig,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WindowConfig {
    pub title: String,
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DebugConfig {
    pub show_overlay: bool,
    pub enable_tracing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SeedConfig {
    pub initial_seed: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PersistenceConfig {
    pub backend: PersistenceBackend,
    pub sqlite_path: String,
    pub spacetimedb_uri: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceBackend {
    Sqlite,
    SpaceTimeDb,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorldConfig {
    pub profile: WorldProfilePreset,
    pub star_density_percent: u16,
    pub debris_density_percent: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MouseLookConfig {
    pub sensitivity: f32,
    pub pitch_limit_deg: f32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicConfig {
    pub enabled: bool,
    pub backend: MusicBackend,
    pub bpm: u16,
    pub scale_mode: ScaleMode,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MusicBackend {
    Silent,
    DebugLog,
    Rodio,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            window: WindowConfig {
                title: String::from("SPACEFLIGHTS"),
                width: 1600,
                height: 900,
            },
            debug: DebugConfig {
                show_overlay: true,
                enable_tracing: true,
            },
            seed: SeedConfig {
                initial_seed: 0x5F37_59DF,
            },
            persistence: PersistenceConfig {
                backend: PersistenceBackend::Sqlite,
                sqlite_path: String::from(":memory:"),
                spacetimedb_uri: String::from("http://localhost:3000"),
            },
            world: WorldConfig {
                profile: WorldProfilePreset::Main,
                star_density_percent: 100,
                debris_density_percent: 100,
            },
            mouse_look: MouseLookConfig {
                sensitivity: 0.0022,
                pitch_limit_deg: 80.0,
            },
            music: MusicConfig {
                enabled: true,
                backend: MusicBackend::Silent,
                bpm: 104,
                scale_mode: ScaleMode::Minor,
            },
        }
    }
}

impl AppConfig {
    #[must_use]
    pub fn with_env_overrides<I>(mut self, pairs: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        for (key, value) in pairs {
            match key.as_str() {
                "SPACEFLIGHTS_WINDOW_TITLE" => {
                    self.window.title = value;
                }
                "SPACEFLIGHTS_WINDOW_WIDTH" => {
                    if let Ok(width) = value.parse::<u32>() {
                        self.window.width = width.max(1);
                    }
                }
                "SPACEFLIGHTS_WINDOW_HEIGHT" => {
                    if let Ok(height) = value.parse::<u32>() {
                        self.window.height = height.max(1);
                    }
                }
                "SPACEFLIGHTS_DEBUG_OVERLAY" => {
                    if let Some(parsed) = parse_bool(&value) {
                        self.debug.show_overlay = parsed;
                    }
                }
                "SPACEFLIGHTS_DEBUG_TRACING" => {
                    if let Some(parsed) = parse_bool(&value) {
                        self.debug.enable_tracing = parsed;
                    }
                }
                "SPACEFLIGHTS_INITIAL_SEED" => {
                    if let Ok(seed) = value.parse::<u64>() {
                        self.seed.initial_seed = seed;
                    }
                }
                "SPACEFLIGHTS_PERSISTENCE_BACKEND" => {
                    if let Some(backend) = parse_persistence_backend(&value) {
                        self.persistence.backend = backend;
                    }
                }
                "SPACEFLIGHTS_SQLITE_PATH" => {
                    self.persistence.sqlite_path = value;
                }
                "SPACEFLIGHTS_SPACETIMEDB_URI" => {
                    self.persistence.spacetimedb_uri = value;
                }
                "SPACEFLIGHTS_WORLD_PROFILE" => {
                    if let Some(profile) = parse_world_profile_preset(&value) {
                        self.world.profile = profile;
                    }
                }
                "SPACEFLIGHTS_WORLD_STAR_DENSITY" => {
                    if let Ok(percent) = value.parse::<u16>() {
                        self.world.star_density_percent = percent.max(10);
                    }
                }
                "SPACEFLIGHTS_WORLD_DEBRIS_DENSITY" => {
                    if let Ok(percent) = value.parse::<u16>() {
                        self.world.debris_density_percent = percent.max(10);
                    }
                }
                "SPACEFLIGHTS_MOUSE_SENSITIVITY" => {
                    if let Ok(sensitivity) = value.parse::<f32>() {
                        self.mouse_look.sensitivity = sensitivity.max(0.0001);
                    }
                }
                "SPACEFLIGHTS_MOUSE_PITCH_LIMIT_DEG" => {
                    if let Ok(limit) = value.parse::<f32>() {
                        self.mouse_look.pitch_limit_deg = limit.clamp(10.0, 89.0);
                    }
                }
                "SPACEFLIGHTS_MUSIC_ENABLED" => {
                    if let Some(enabled) = parse_bool(&value) {
                        self.music.enabled = enabled;
                    }
                }
                "SPACEFLIGHTS_MUSIC_BACKEND" => {
                    if let Some(backend) = parse_music_backend(&value) {
                        self.music.backend = backend;
                    }
                }
                "SPACEFLIGHTS_MUSIC_BPM" => {
                    if let Ok(bpm) = value.parse::<u16>() {
                        self.music.bpm = bpm.clamp(40, 220);
                    }
                }
                "SPACEFLIGHTS_MUSIC_SCALE" => {
                    if let Some(scale_mode) = parse_scale_mode(&value) {
                        self.music.scale_mode = scale_mode;
                    }
                }
                _ => {}
            }
        }

        self
    }
}

#[must_use]
pub fn parse_bool(input: &str) -> Option<bool> {
    match input.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

#[must_use]
pub fn parse_persistence_backend(input: &str) -> Option<PersistenceBackend> {
    match input.trim().to_ascii_lowercase().as_str() {
        "sqlite" => Some(PersistenceBackend::Sqlite),
        "spacetimedb" | "space-time-db" | "space_time_db" => Some(PersistenceBackend::SpaceTimeDb),
        _ => None,
    }
}

#[must_use]
pub fn parse_world_profile_preset(input: &str) -> Option<WorldProfilePreset> {
    match input.trim().to_ascii_lowercase().as_str() {
        "main" => Some(WorldProfilePreset::Main),
        "debug" | "dev" | "dev_debug" => Some(WorldProfilePreset::DevDebug),
        _ => None,
    }
}

#[must_use]
pub fn parse_music_backend(input: &str) -> Option<MusicBackend> {
    match input.trim().to_ascii_lowercase().as_str() {
        "silent" | "none" => Some(MusicBackend::Silent),
        "debug" | "debug_log" => Some(MusicBackend::DebugLog),
        "rodio" => Some(MusicBackend::Rodio),
        _ => None,
    }
}

#[must_use]
pub fn parse_scale_mode(input: &str) -> Option<ScaleMode> {
    match input.trim().to_ascii_lowercase().as_str() {
        "minor" => Some(ScaleMode::Minor),
        "major" => Some(ScaleMode::Major),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{
        parse_bool, parse_music_backend, parse_persistence_backend, parse_scale_mode,
        parse_world_profile_preset, AppConfig, MusicBackend, PersistenceBackend,
    };
    use crate::music::ScaleMode;
    use crate::worldgen::WorldProfilePreset;

    fn assert_close(actual: f32, expected: f32) {
        let diff = (actual - expected).abs();
        assert!(diff <= f32::EPSILON, "actual={actual} expected={expected}");
    }

    #[test]
    fn default_config_is_stable() {
        let cfg = AppConfig::default();

        assert_eq!(cfg.window.title, "SPACEFLIGHTS");
        assert_eq!(cfg.window.width, 1600);
        assert_eq!(cfg.window.height, 900);
        assert!(cfg.debug.show_overlay);
        assert!(cfg.debug.enable_tracing);
        assert_eq!(cfg.seed.initial_seed, 0x5F37_59DF);
        assert_eq!(cfg.persistence.backend, PersistenceBackend::Sqlite);
        assert_eq!(cfg.persistence.sqlite_path, ":memory:");
        assert_eq!(cfg.world.profile, WorldProfilePreset::Main);
        assert_eq!(cfg.world.star_density_percent, 100);
        assert_eq!(cfg.world.debris_density_percent, 100);
        assert_close(cfg.mouse_look.sensitivity, 0.0022);
        assert_close(cfg.mouse_look.pitch_limit_deg, 80.0);
        assert!(cfg.music.enabled);
        assert_eq!(cfg.music.backend, MusicBackend::Silent);
        assert_eq!(cfg.music.bpm, 104);
        assert_eq!(cfg.music.scale_mode, ScaleMode::Minor);
    }

    #[test]
    fn env_overrides_are_applied_when_valid() {
        let cfg = AppConfig::default().with_env_overrides([
            (
                String::from("SPACEFLIGHTS_WINDOW_TITLE"),
                String::from("TEST"),
            ),
            (
                String::from("SPACEFLIGHTS_WINDOW_WIDTH"),
                String::from("1920"),
            ),
            (
                String::from("SPACEFLIGHTS_WINDOW_HEIGHT"),
                String::from("1080"),
            ),
            (
                String::from("SPACEFLIGHTS_DEBUG_OVERLAY"),
                String::from("false"),
            ),
            (
                String::from("SPACEFLIGHTS_DEBUG_TRACING"),
                String::from("0"),
            ),
            (
                String::from("SPACEFLIGHTS_INITIAL_SEED"),
                String::from("42"),
            ),
            (
                String::from("SPACEFLIGHTS_PERSISTENCE_BACKEND"),
                String::from("spacetimedb"),
            ),
            (
                String::from("SPACEFLIGHTS_SQLITE_PATH"),
                String::from("./spaceflights.db"),
            ),
            (
                String::from("SPACEFLIGHTS_SPACETIMEDB_URI"),
                String::from("https://spacetime.local"),
            ),
            (
                String::from("SPACEFLIGHTS_WORLD_PROFILE"),
                String::from("dev_debug"),
            ),
            (
                String::from("SPACEFLIGHTS_WORLD_STAR_DENSITY"),
                String::from("135"),
            ),
            (
                String::from("SPACEFLIGHTS_WORLD_DEBRIS_DENSITY"),
                String::from("65"),
            ),
            (
                String::from("SPACEFLIGHTS_MOUSE_SENSITIVITY"),
                String::from("0.0031"),
            ),
            (
                String::from("SPACEFLIGHTS_MOUSE_PITCH_LIMIT_DEG"),
                String::from("75"),
            ),
            (
                String::from("SPACEFLIGHTS_MUSIC_ENABLED"),
                String::from("true"),
            ),
            (
                String::from("SPACEFLIGHTS_MUSIC_BACKEND"),
                String::from("debug"),
            ),
            (String::from("SPACEFLIGHTS_MUSIC_BPM"), String::from("128")),
            (
                String::from("SPACEFLIGHTS_MUSIC_SCALE"),
                String::from("major"),
            ),
        ]);

        assert_eq!(cfg.window.title, "TEST");
        assert_eq!(cfg.window.width, 1920);
        assert_eq!(cfg.window.height, 1080);
        assert!(!cfg.debug.show_overlay);
        assert!(!cfg.debug.enable_tracing);
        assert_eq!(cfg.seed.initial_seed, 42);
        assert_eq!(cfg.persistence.backend, PersistenceBackend::SpaceTimeDb);
        assert_eq!(cfg.persistence.sqlite_path, "./spaceflights.db");
        assert_eq!(cfg.persistence.spacetimedb_uri, "https://spacetime.local");
        assert_eq!(cfg.world.profile, WorldProfilePreset::DevDebug);
        assert_eq!(cfg.world.star_density_percent, 135);
        assert_eq!(cfg.world.debris_density_percent, 65);
        assert_close(cfg.mouse_look.sensitivity, 0.0031);
        assert_close(cfg.mouse_look.pitch_limit_deg, 75.0);
        assert_eq!(cfg.music.backend, MusicBackend::DebugLog);
        assert_eq!(cfg.music.bpm, 128);
        assert_eq!(cfg.music.scale_mode, ScaleMode::Major);
    }

    #[test]
    fn invalid_env_values_fall_back_to_defaults() {
        let cfg = AppConfig::default().with_env_overrides([
            (
                String::from("SPACEFLIGHTS_WINDOW_WIDTH"),
                String::from("abc"),
            ),
            (
                String::from("SPACEFLIGHTS_WINDOW_HEIGHT"),
                String::from("0"),
            ),
            (
                String::from("SPACEFLIGHTS_DEBUG_OVERLAY"),
                String::from("maybe"),
            ),
            (
                String::from("SPACEFLIGHTS_INITIAL_SEED"),
                String::from("nan"),
            ),
            (
                String::from("SPACEFLIGHTS_PERSISTENCE_BACKEND"),
                String::from("bogus"),
            ),
            (
                String::from("SPACEFLIGHTS_WORLD_PROFILE"),
                String::from("sandbox"),
            ),
            (
                String::from("SPACEFLIGHTS_MOUSE_SENSITIVITY"),
                String::from("nope"),
            ),
            (
                String::from("SPACEFLIGHTS_MUSIC_BACKEND"),
                String::from("loud"),
            ),
        ]);

        assert_eq!(cfg.window.width, 1600);
        assert_eq!(cfg.window.height, 1);
        assert!(cfg.debug.show_overlay);
        assert_eq!(cfg.seed.initial_seed, 0x5F37_59DF);
        assert_eq!(cfg.persistence.backend, PersistenceBackend::Sqlite);
        assert_eq!(cfg.world.profile, WorldProfilePreset::Main);
        assert_close(cfg.mouse_look.sensitivity, 0.0022);
        assert_eq!(cfg.music.backend, MusicBackend::Silent);
    }

    #[test]
    fn parse_bool_supports_common_variants() {
        assert_eq!(parse_bool("true"), Some(true));
        assert_eq!(parse_bool("YES"), Some(true));
        assert_eq!(parse_bool("0"), Some(false));
        assert_eq!(parse_bool("off"), Some(false));
        assert_eq!(parse_bool("unknown"), None);
    }

    #[test]
    fn parse_backend_supports_known_values() {
        assert_eq!(
            parse_persistence_backend("sqlite"),
            Some(PersistenceBackend::Sqlite)
        );
        assert_eq!(
            parse_persistence_backend("space_time_db"),
            Some(PersistenceBackend::SpaceTimeDb)
        );
        assert_eq!(parse_persistence_backend("mongo"), None);
    }

    #[test]
    fn parse_world_profile_supports_known_values() {
        assert_eq!(
            parse_world_profile_preset("main"),
            Some(WorldProfilePreset::Main)
        );
        assert_eq!(
            parse_world_profile_preset("dev"),
            Some(WorldProfilePreset::DevDebug)
        );
        assert_eq!(parse_world_profile_preset("prod"), None);
    }

    #[test]
    fn parse_music_backend_supports_known_values() {
        assert_eq!(parse_music_backend("silent"), Some(MusicBackend::Silent));
        assert_eq!(parse_music_backend("debug"), Some(MusicBackend::DebugLog));
        assert_eq!(parse_music_backend("rodio"), Some(MusicBackend::Rodio));
        assert_eq!(parse_music_backend("midi"), None);
    }

    #[test]
    fn parse_scale_supports_known_values() {
        assert_eq!(parse_scale_mode("minor"), Some(ScaleMode::Minor));
        assert_eq!(parse_scale_mode("major"), Some(ScaleMode::Major));
        assert_eq!(parse_scale_mode("dorian"), None);
    }
}
