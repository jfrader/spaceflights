#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    Ship,
    Module,
    Eva,
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct NavVector3Km {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameplayState {
    pub mode: ControlMode,
    pub ship_speed_km_s: f64,
    pub ship_distance_km: f64,
    pub ship_position_km: NavVector3Km,
    pub ship_yaw_rad: f64,
    pub ship_pitch_rad: f64,
    pub mission_elapsed_seconds: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GameplayTuning {
    pub initial_speed_km_s: f64,
    pub speed_step_km_s: f64,
    pub min_speed_km_s: f64,
    pub max_speed_km_s: f64,
    pub turn_rate_rad_s: f64,
    pub pitch_limit_rad: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameCommand {
    IncreaseShipSpeed,
    DecreaseShipSpeed,
    YawLeft,
    YawRight,
    PitchUp,
    PitchDown,
    SetMode(ControlMode),
}

impl Default for GameplayTuning {
    fn default() -> Self {
        Self {
            initial_speed_km_s: 25.0,
            speed_step_km_s: 20.0,
            min_speed_km_s: 0.0,
            max_speed_km_s: 5_000.0,
            turn_rate_rad_s: 0.75,
            pitch_limit_rad: 75.0_f64.to_radians(),
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
            ship_position_km: NavVector3Km::default(),
            ship_yaw_rad: 0.0,
            ship_pitch_rad: 0.0,
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
    let mut yaw_input = 0.0_f64;
    let mut pitch_input = 0.0_f64;

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
            GameCommand::YawLeft => {
                yaw_input -= 1.0;
            }
            GameCommand::YawRight => {
                yaw_input += 1.0;
            }
            GameCommand::PitchUp => {
                pitch_input += 1.0;
            }
            GameCommand::PitchDown => {
                pitch_input -= 1.0;
            }
            GameCommand::SetMode(mode) => {
                state.mode = *mode;
            }
        }
    }

    let dt = dt_seconds.max(0.0);
    if dt > 0.0 {
        state.ship_yaw_rad += yaw_input * tuning.turn_rate_rad_s * dt;
        state.ship_pitch_rad = (state.ship_pitch_rad + pitch_input * tuning.turn_rate_rad_s * dt)
            .clamp(-tuning.pitch_limit_rad, tuning.pitch_limit_rad);
    }

    let cos_pitch = state.ship_pitch_rad.cos();
    let forward_x = state.ship_yaw_rad.sin() * cos_pitch;
    let forward_y = state.ship_pitch_rad.sin();
    let forward_z = -state.ship_yaw_rad.cos() * cos_pitch;
    let moved_km = state.ship_speed_km_s * dt;

    state.ship_position_km.x += forward_x * moved_km;
    state.ship_position_km.y += forward_y * moved_km;
    state.ship_position_km.z += forward_z * moved_km;
    state.ship_distance_km += moved_km;
    state.mission_elapsed_seconds += dt;
}

#[cfg(test)]
mod tests {
    use super::{step, ControlMode, GameCommand, GameplayState, GameplayTuning, NavVector3Km};

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
        assert_eq!(
            state.ship_position_km,
            NavVector3Km {
                x: 0.0,
                y: 0.0,
                z: -50.0
            }
        );
    }

    #[test]
    fn speed_commands_clamp_to_bounds() {
        let tuning = GameplayTuning {
            initial_speed_km_s: 1.0,
            speed_step_km_s: 5.0,
            min_speed_km_s: 0.0,
            max_speed_km_s: 10.0,
            turn_rate_rad_s: 0.75,
            pitch_limit_rad: 75.0_f64.to_radians(),
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

    #[test]
    fn yaw_navigation_changes_directional_displacement() {
        let tuning = GameplayTuning {
            initial_speed_km_s: 10.0,
            speed_step_km_s: 0.0,
            min_speed_km_s: 0.0,
            max_speed_km_s: 100.0,
            turn_rate_rad_s: std::f64::consts::FRAC_PI_2,
            pitch_limit_rad: 80.0_f64.to_radians(),
        };
        let mut state = GameplayState::new(tuning);

        step(&mut state, 1.0, &[GameCommand::YawRight], tuning);

        assert_close(state.ship_yaw_rad, std::f64::consts::FRAC_PI_2);
        assert!(state.ship_position_km.x > 9.99);
        assert!(state.ship_position_km.z.abs() < 0.01);
    }
}
