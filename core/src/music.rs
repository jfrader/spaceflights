use crate::Seed;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Waveform {
    Sine,
    Triangle,
    Saw,
    Square,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScaleMode {
    Minor,
    Major,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MusicProfile {
    pub bpm: u16,
    pub bars_per_phrase: u8,
    pub notes_per_bar: u8,
    pub root_midi: u8,
    pub scale_mode: ScaleMode,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NoteEvent {
    pub midi_note: u8,
    pub frequency_hz: f32,
    pub duration_seconds: f32,
    pub velocity: f32,
    pub waveform: Waveform,
}

impl Default for MusicProfile {
    fn default() -> Self {
        Self {
            bpm: 104,
            bars_per_phrase: 4,
            notes_per_bar: 8,
            root_midi: 45,
            scale_mode: ScaleMode::Minor,
        }
    }
}

#[must_use]
pub fn generate_phrase(seed: Seed, phrase_index: u64, profile: MusicProfile) -> Vec<NoteEvent> {
    let step_duration =
        60.0_f32 / f32::from(profile.bpm) / (f32::from(profile.notes_per_bar) / 4.0);
    let total_steps = usize::from(profile.bars_per_phrase) * usize::from(profile.notes_per_bar);
    let mut phrase: Vec<NoteEvent> = Vec::with_capacity(total_steps);

    for step in 0..total_steps {
        let noise_index = phrase_index
            .saturating_mul(4096)
            .saturating_add(u64::try_from(step).unwrap_or(u64::MAX));
        let noise = seed.nth_value(noise_index);

        let degree = degree_from_noise(noise, profile.scale_mode);
        let octave_offset = octave_offset_from_noise(noise);
        let midi = i32::from(profile.root_midi) + i32::from(degree) + octave_offset;
        let midi_clamped = u8::try_from(midi.clamp(24, 96)).unwrap_or(96);

        let frequency_hz = midi_to_hz(midi_clamped);
        let velocity = velocity_from_noise(noise);
        let waveform = waveform_from_noise(noise);

        phrase.push(NoteEvent {
            midi_note: midi_clamped,
            frequency_hz,
            duration_seconds: step_duration,
            velocity,
            waveform,
        });
    }

    phrase
}

fn degree_from_noise(noise: u64, mode: ScaleMode) -> i8 {
    let degrees_minor: [i8; 7] = [0, 2, 3, 5, 7, 8, 10];
    let degrees_major: [i8; 7] = [0, 2, 4, 5, 7, 9, 11];

    let index = (noise % 7) as usize;
    match mode {
        ScaleMode::Minor => degrees_minor[index],
        ScaleMode::Major => degrees_major[index],
    }
}

fn octave_offset_from_noise(noise: u64) -> i32 {
    match (noise >> 8) % 4 {
        0 => -12,
        1 => 0,
        2 => 12,
        _ => 24,
    }
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Frequency math uses f32 for runtime DSP-friendly payloads."
)]
fn midi_to_hz(note: u8) -> f32 {
    let exponent = (f32::from(note) - 69.0) / 12.0;
    440.0 * 2.0_f32.powf(exponent)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Normalized velocity is f32 by design."
)]
fn velocity_from_noise(noise: u64) -> f32 {
    let n = ((noise >> 16) % 10_000) as f32 / 10_000.0;
    0.35 + n * 0.6
}

fn waveform_from_noise(noise: u64) -> Waveform {
    match (noise >> 32) % 4 {
        0 => Waveform::Sine,
        1 => Waveform::Triangle,
        2 => Waveform::Saw,
        _ => Waveform::Square,
    }
}

#[cfg(test)]
mod tests {
    use super::{generate_phrase, MusicProfile, NoteEvent, ScaleMode};
    use crate::Seed;

    fn phrase_signature(phrase: &[NoteEvent]) -> Vec<(u8, u32, u32)> {
        phrase
            .iter()
            .map(|event| {
                (
                    event.midi_note,
                    event.frequency_hz.to_bits(),
                    event.velocity.to_bits(),
                )
            })
            .collect()
    }

    #[test]
    fn phrase_generation_is_stable() {
        let seed = Seed::new(777);
        let profile = MusicProfile::default();

        let a = generate_phrase(seed, 0, profile);
        let b = generate_phrase(seed, 0, profile);

        assert_eq!(phrase_signature(&a), phrase_signature(&b));
    }

    #[test]
    fn phrase_generation_changes_with_seed() {
        let profile = MusicProfile::default();
        let a = generate_phrase(Seed::new(1), 0, profile);
        let b = generate_phrase(Seed::new(2), 0, profile);

        assert_ne!(phrase_signature(&a), phrase_signature(&b));
    }

    #[test]
    fn profile_controls_phrase_length() {
        let profile = MusicProfile {
            bars_per_phrase: 2,
            notes_per_bar: 4,
            ..MusicProfile::default()
        };

        let phrase = generate_phrase(Seed::new(10), 1, profile);
        assert_eq!(phrase.len(), 8);
    }

    #[test]
    fn major_and_minor_change_note_selection() {
        let seed = Seed::new(44);
        let minor = MusicProfile {
            scale_mode: ScaleMode::Minor,
            ..MusicProfile::default()
        };
        let major = MusicProfile {
            scale_mode: ScaleMode::Major,
            ..MusicProfile::default()
        };

        let phrase_minor = generate_phrase(seed, 0, minor);
        let phrase_major = generate_phrase(seed, 0, major);

        assert_ne!(
            phrase_signature(&phrase_minor),
            phrase_signature(&phrase_major)
        );
    }
}
