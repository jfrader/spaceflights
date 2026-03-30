use bevy::prelude::*;

#[derive(SystemSet, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameSet {
    Persistence,
    Input,
    Gameplay,
    Music,
    World,
    Camera,
    Ui,
    Debug,
}
