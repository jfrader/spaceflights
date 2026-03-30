use spaceflights_app::{build_gameplay_debug_app, RuntimeMode};
use spaceflights_core::AppConfig;
use std::time::{SystemTime, UNIX_EPOCH};

fn main() {
    let mut config = AppConfig::default().with_env_overrides(std::env::vars());
    config.seed.initial_seed = ephemeral_dev_seed();
    let mut app = build_gameplay_debug_app(config, RuntimeMode::Desktop);
    app.run();
}

#[allow(
    clippy::cast_possible_truncation,
    reason = "Ephemeral seed intentionally folds u128 time entropy into u64 seed space."
)]
fn ephemeral_dev_seed() -> u64 {
    let now_nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_u128, |duration| duration.as_nanos());
    let pid = u128::from(std::process::id());
    let mixed = now_nanos ^ (pid << 64);
    let low = mixed as u64;
    let high = (mixed >> 64) as u64;
    low ^ high.rotate_left(23) ^ 0x9E37_79B9_7F4A_7C15
}
