//! Shared event namespaces for cross-feature communication contracts.
//! These are intentionally minimal during foundation phase.

pub mod app {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum AppLifecycleEvent {
        Bootstrapped,
    }
}

pub mod ui {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum UiEvent {
        OverlayToggled,
    }
}

pub mod world {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum WorldEvent {
        SeedUpdated,
    }
}
