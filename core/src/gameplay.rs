#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Ship,
    Module,
    Eva,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameplayState {
    pub mode: ControlMode,
    pub ship_speed_km_s: f64,
    pub ship_distance_km: f64,
    pub mission_elapsed_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameplayTuning {
    pub initial_speed_km_s: f64,
    pub speed_step_km_s: f64,
    pub min_speed_km_s: f64,
    pub max_speed_km_s: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommand {
    IncreaseShipSpeed,
    DecreaseShipSpeed,
    SetMode(ControlMode),
}

impl Default for GameplayTuning {
    fn default() -> Self {
        Self {
            initial_speed_km_s: 25.0,
            speed_step_km_s: 5.0,
            min_speed_km_s: 0.0,
            max_speed_km_s: 500.0,
        }
    }
}

impl GameplayState {
    #[must_use]
    pub fn new(tuning: GameplayTuning) -> Self {
        Self {
            mode: ControlMode::Ship,
            ship_speed_km_s: tuning.initial_speed_km_s,
            ship_distance_km: 0.0,
            mission_elapsed_seconds: 0.0,
        }
    }
}

pub fn step(
    state: &mut GameplayState,
    dt_seconds: f64,
    commands: &[GameCommand],
    tuning: GameplayTuning,
) {
    for command in commands {
        match command {
            GameCommand::IncreaseShipSpeed => {
                state.ship_speed_km_s = (state.ship_speed_km_s + tuning.speed_step_km_s)
                    .clamp(tuning.min_speed_km_s, tuning.max_speed_km_s);
            }
            GameCommand::DecreaseShipSpeed => {
                state.ship_speed_km_s = (state.ship_speed_km_s - tuning.speed_step_km_s)
                    .clamp(tuning.min_speed_km_s, tuning.max_speed_km_s);
            }
            GameCommand::SetMode(mode) => {
                state.mode = *mode;
            }
        }
    }

    let dt = dt_seconds.max(0.0);
    state.ship_distance_km += state.ship_speed_km_s * dt;
    state.mission_elapsed_seconds += dt;
}

#[cfg(test)]
mod tests {
    use super::{step, ControlMode, GameCommand, GameplayState, GameplayTuning};

    fn assert_close(actual: f64, expected: f64) {
        let diff = (actual - expected).abs();
        assert!(diff <= f64::EPSILON, "actual={actual} expected={expected}");
    }

    #[test]
    fn step_advances_distance_with_current_speed() {
        let tuning = GameplayTuning::default();
        let mut state = GameplayState::new(tuning);

        step(&mut state, 2.0, &[], tuning);

        assert_close(state.ship_distance_km, 50.0);
        assert_close(state.mission_elapsed_seconds, 2.0);
    }

    #[test]
    fn speed_commands_clamp_to_bounds() {
        let tuning = GameplayTuning {
            initial_speed_km_s: 1.0,
            speed_step_km_s: 5.0,
            min_speed_km_s: 0.0,
            max_speed_km_s: 10.0,
        };
        let mut state = GameplayState::new(tuning);

        step(
            &mut state,
            0.0,
            &[
                GameCommand::IncreaseShipSpeed,
                GameCommand::IncreaseShipSpeed,
                GameCommand::IncreaseShipSpeed,
            ],
            tuning,
        );

        assert_close(state.ship_speed_km_s, 10.0);

        step(
            &mut state,
            0.0,
            &[
                GameCommand::DecreaseShipSpeed,
                GameCommand::DecreaseShipSpeed,
                GameCommand::DecreaseShipSpeed,
            ],
            tuning,
        );

        assert_close(state.ship_speed_km_s, 0.0);
    }

    #[test]
    fn mode_switch_is_applied_in_order() {
        let tuning = GameplayTuning::default();
        let mut state = GameplayState::new(tuning);

        step(
            &mut state,
            0.0,
            &[
                GameCommand::SetMode(ControlMode::Module),
                GameCommand::SetMode(ControlMode::Eva),
            ],
            tuning,
        );

        assert_eq!(state.mode, ControlMode::Eva);
    }
}
