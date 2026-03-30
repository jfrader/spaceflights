use spaceflights_app::{build_app, EnabledFeatures, RuntimeMode};
use spaceflights_core::AppConfig;

fn main() {
    let config = AppConfig::default().with_env_overrides(std::env::vars());
    let mut app = build_app(config, RuntimeMode::Desktop, EnabledFeatures::default());
    app.run();
}
