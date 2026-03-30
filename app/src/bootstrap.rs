use std::sync::Once;

static TRACING_INIT: Once = Once::new();

pub fn init_tracing(enabled: bool) {
    if !enabled {
        return;
    }

    TRACING_INIT.call_once(|| {
        let filter = tracing_subscriber::EnvFilter::try_from_default_env()
            .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info"));

        let _ = tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_target(false)
            .compact()
            .try_init();
    });
}
