use std::sync::Mutex;

use bevy::prelude::*;
use rodio::buffer::SamplesBuffer;
use rodio::{OutputStream, OutputStreamHandle, Sink};
use spaceflights_core::{generate_phrase, MusicBackend, MusicProfile, NoteEvent, Seed, Waveform};

use crate::plugins::scene::CurrentSeed;
use crate::plugins::schedule::GameSet;
use crate::AppConfigResource;

pub struct MusicFeaturePlugin;

pub trait ProceduralSynthBackend: 'static {
    fn backend_name(&self) -> &'static str;
    fn emit_notes(&mut self, notes: &[NoteEvent]);
}

pub struct SilentBackend;

impl ProceduralSynthBackend for SilentBackend {
    fn backend_name(&self) -> &'static str {
        "silent"
    }

    fn emit_notes(&mut self, _notes: &[NoteEvent]) {}
}

#[derive(Default)]
pub struct DebugLogBackend;

impl ProceduralSynthBackend for DebugLogBackend {
    fn backend_name(&self) -> &'static str {
        "debug_log"
    }

    fn emit_notes(&mut self, notes: &[NoteEvent]) {
        std::hint::black_box(notes.len());
    }
}

pub struct RodioBackend {
    stream: Mutex<OutputStream>,
    handle: Mutex<OutputStreamHandle>,
    sink: Mutex<Sink>,
}

impl RodioBackend {
    /// Create an audio output backend using the system default output device.
    ///
    /// # Errors
    /// Returns an error when no default output stream or sink can be created.
    pub fn try_new() -> Result<Self, String> {
        let (stream, handle) = OutputStream::try_default()
            .map_err(|error| format!("failed to open default output stream: {error}"))?;
        let sink = Sink::try_new(&handle)
            .map_err(|error| format!("failed to build sink for output stream: {error}"))?;

        Ok(Self {
            stream: Mutex::new(stream),
            handle: Mutex::new(handle),
            sink: Mutex::new(sink),
        })
    }
}

impl ProceduralSynthBackend for RodioBackend {
    fn backend_name(&self) -> &'static str {
        "rodio"
    }

    fn emit_notes(&mut self, notes: &[NoteEvent]) {
        let Ok(_stream_guard) = self.stream.lock() else {
            return;
        };
        let Ok(_handle_guard) = self.handle.lock() else {
            return;
        };
        let Ok(sink) = self.sink.lock() else {
            return;
        };

        let source = render_lofi_phrase(notes);
        sink.append(source);
    }
}

pub struct MusicBackendResource {
    backend: Box<dyn ProceduralSynthBackend>,
}

#[derive(Resource, Debug, Clone)]
pub struct MusicRuntime {
    pub profile: MusicProfile,
    pub seed: Seed,
    pub phrase_index: u64,
    pub notes_emitted: u64,
    pub next_phrase_at_seconds: f64,
    pub phrase_duration_seconds: f64,
}

#[derive(Resource, Debug, Clone)]
pub struct MusicRenderStats {
    pub backend_name: String,
    pub phrase_index: u64,
    pub notes_emitted: u64,
}

impl Plugin for MusicFeaturePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, initialize_music)
            .add_systems(FixedUpdate, emit_next_phrase.in_set(GameSet::Music));
    }
}

fn initialize_music(world: &mut World) {
    let Some(config) = world.get_resource::<AppConfigResource>() else {
        world.insert_resource(MusicRenderStats {
            backend_name: String::from("config_missing"),
            phrase_index: 0,
            notes_emitted: 0,
        });
        return;
    };

    let music_cfg = config.0.music;
    let seed = world
        .get_resource::<CurrentSeed>()
        .map_or(config.0.seed.initial_seed, |resource| resource.0.value());

    if !music_cfg.enabled {
        world.insert_resource(MusicRenderStats {
            backend_name: String::from("disabled"),
            phrase_index: 0,
            notes_emitted: 0,
        });
        return;
    }

    let profile = MusicProfile {
        bpm: music_cfg.bpm,
        scale_mode: music_cfg.scale_mode,
        ..MusicProfile::default()
    };

    let phrase_duration = phrase_duration_seconds(profile);

    let (backend, backend_name): (Box<dyn ProceduralSynthBackend>, String) =
        build_backend(music_cfg.backend);

    world.insert_non_send_resource(MusicBackendResource { backend });
    world.insert_resource(MusicRuntime {
        profile,
        seed: Seed::new(seed),
        phrase_index: 0,
        notes_emitted: 0,
        next_phrase_at_seconds: 0.0,
        phrase_duration_seconds: phrase_duration,
    });
    world.insert_resource(MusicRenderStats {
        backend_name,
        phrase_index: 0,
        notes_emitted: 0,
    });
}

fn build_backend(preferred: MusicBackend) -> (Box<dyn ProceduralSynthBackend>, String) {
    match preferred {
        MusicBackend::Silent => (Box::new(SilentBackend), String::from("silent")),
        MusicBackend::DebugLog => (Box::new(DebugLogBackend), String::from("debug_log")),
        MusicBackend::Rodio => match RodioBackend::try_new() {
            Ok(backend) => (Box::new(backend), String::from("rodio")),
            Err(error) => {
                bevy::log::warn!("rodio backend unavailable, falling back to silent: {error}");
                (Box::new(SilentBackend), String::from("rodio_unavailable"))
            }
        },
    }
}

fn emit_next_phrase(
    time: Res<Time>,
    mut backend: Option<NonSendMut<MusicBackendResource>>,
    mut runtime: Option<ResMut<MusicRuntime>>,
    mut stats: Option<ResMut<MusicRenderStats>>,
) {
    let (Some(backend), Some(runtime), Some(stats)) =
        (backend.as_mut(), runtime.as_mut(), stats.as_mut())
    else {
        return;
    };

    let now = time.elapsed_seconds_f64();
    if now < runtime.next_phrase_at_seconds {
        return;
    }

    let phrase = generate_phrase(runtime.seed, runtime.phrase_index, runtime.profile);
    backend.backend.emit_notes(&phrase);

    runtime.phrase_index = runtime.phrase_index.saturating_add(1);
    runtime.notes_emitted = runtime
        .notes_emitted
        .saturating_add(u64::try_from(phrase.len()).unwrap_or(u64::MAX));
    runtime.next_phrase_at_seconds += runtime.phrase_duration_seconds;

    stats.phrase_index = runtime.phrase_index;
    stats.notes_emitted = runtime.notes_emitted;
    stats.backend_name = String::from(backend.backend.backend_name());
}

fn phrase_duration_seconds(profile: MusicProfile) -> f64 {
    let step_duration =
        60.0_f64 / f64::from(profile.bpm) / (f64::from(profile.notes_per_bar) / 4.0_f64);
    step_duration * f64::from(profile.bars_per_phrase) * f64::from(profile.notes_per_bar)
}

const SAMPLE_RATE: u32 = 44_100;

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    reason = "Audio sample generation intentionally uses f32 output for rodio buffers."
)]
fn render_lofi_phrase(notes: &[NoteEvent]) -> SamplesBuffer<f32> {
    if notes.is_empty() {
        return SamplesBuffer::new(2, SAMPLE_RATE, Vec::new());
    }

    let step_duration = notes[0].duration_seconds.max(0.03);
    let phrase_duration = notes
        .iter()
        .map(|note| note.duration_seconds.max(0.03))
        .sum::<f32>()
        .max(step_duration);
    let total_samples = (phrase_duration * SAMPLE_RATE as f32).ceil() as usize;
    let mut stereo = Vec::with_capacity(total_samples.saturating_mul(2));

    for sample_index in 0..total_samples {
        let t = sample_index as f32 / SAMPLE_RATE as f32;
        let step = ((t / step_duration).floor() as usize).min(notes.len().saturating_sub(1));
        let local_t = t - step as f32 * step_duration;
        let step_phase = (local_t / step_duration).clamp(0.0, 1.0);
        let note = notes[step];

        let melody = melody_voice(note, local_t, step_phase);
        let bass = bass_voice(notes, step, local_t, step_duration, t);
        let pad = pad_voice(notes, step, t);
        let drums = drum_bus(step, local_t, step_duration);
        let air = vinyl_air(step, sample_index);

        let mono = (melody + bass + pad + drums + air).clamp(-1.0, 1.0) * 0.55;
        let pan = pan_lfo(t);
        let left = mono * (1.0 - pan * 0.25);
        let right = mono * (1.0 + pan * 0.25);

        stereo.push(left);
        stereo.push(right);
    }

    SamplesBuffer::new(2, SAMPLE_RATE, stereo)
}

fn melody_voice(note: NoteEvent, local_t: f32, step_phase: f32) -> f32 {
    let frequency = note.frequency_hz.clamp(20.0, 4_000.0);
    let amp = note.velocity.clamp(0.0, 1.0) * adsr(step_phase, 0.04, 0.25, 0.45, 0.18);
    let detune = osc_sine(frequency * 1.005, local_t) * 0.35 + osc_sine(frequency * 0.998, local_t);
    let voiced = match note.waveform {
        Waveform::Sine => osc_sine(frequency, local_t),
        Waveform::Triangle => osc_triangle(frequency, local_t),
        Waveform::Saw => osc_saw(frequency, local_t),
        Waveform::Square => osc_square(frequency, local_t),
    };
    (0.55 * voiced + 0.45 * detune) * amp * 0.36
}

fn bass_voice(
    notes: &[NoteEvent],
    step: usize,
    local_t: f32,
    step_duration: f32,
    absolute_t: f32,
) -> f32 {
    let degree_anchor = (step / 2) * 2;
    let index = degree_anchor.min(notes.len().saturating_sub(1));
    let frequency = (notes[index].frequency_hz * 0.5).clamp(20.0, 220.0);
    let beat_phase = (local_t / step_duration).clamp(0.0, 1.0);
    let movement = osc_sine(0.12, absolute_t) * 0.015;
    let osc = 0.78 * osc_sine(frequency * (1.0 + movement), local_t)
        + 0.22 * osc_square(frequency, local_t);
    osc * adsr(beat_phase, 0.03, 0.20, 0.55, 0.12) * 0.33
}

fn pad_voice(notes: &[NoteEvent], step: usize, absolute_t: f32) -> f32 {
    let chord_step = (step / 8) * 8;
    let i0 = chord_step.min(notes.len().saturating_sub(1));
    let i1 = (chord_step + 2).min(notes.len().saturating_sub(1));
    let i2 = (chord_step + 4).min(notes.len().saturating_sub(1));
    let f0 = notes[i0].frequency_hz * 0.5;
    let f1 = notes[i1].frequency_hz * 0.5;
    let f2 = notes[i2].frequency_hz * 0.5;

    let shimmer = 1.0 + osc_sine(0.08, absolute_t) * 0.01;
    let mix = osc_triangle(f0 * shimmer, absolute_t) * 0.46
        + osc_triangle(f1 * shimmer, absolute_t) * 0.34
        + osc_triangle(f2 * shimmer, absolute_t) * 0.20;
    let swell = 0.55 + 0.45 * osc_sine(0.035, absolute_t);
    mix * swell * 0.22
}

#[allow(
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss,
    reason = "Drum noise indexing intentionally quantizes time into deterministic hash steps."
)]
fn drum_bus(step: usize, local_t: f32, step_duration: f32) -> f32 {
    let beat_in_bar = step % 8;
    let phase = (local_t / step_duration).clamp(0.0, 1.0);

    let kick = if beat_in_bar == 0 || beat_in_bar == 4 {
        let freq = 95.0 - 55.0 * phase;
        osc_sine(freq.max(28.0), local_t) * (1.0 - phase).powf(3.2) * 0.70
    } else {
        0.0
    };

    let snare = if beat_in_bar == 2 || beat_in_bar == 6 {
        let noise = white_noise(hash_u64(step as u64 * 97 + (local_t * 10_000.0) as u64));
        noise * (1.0 - phase).powf(2.5) * 0.26
    } else {
        0.0
    };

    let hat = {
        let noise = white_noise(hash_u64(step as u64 * 3_311 + (local_t * 60_000.0) as u64));
        let amp = (1.0 - phase).powf(9.0);
        noise * amp * 0.12
    };

    kick + snare + hat
}

fn vinyl_air(step: usize, sample_index: usize) -> f32 {
    let n = white_noise(hash_u64(step as u64 * 1_315_423_911 + sample_index as u64));
    n * 0.018
}

fn pan_lfo(t: f32) -> f32 {
    osc_sine(0.05, t).clamp(-1.0, 1.0)
}

fn adsr(phase: f32, attack: f32, decay: f32, sustain: f32, release: f32) -> f32 {
    if phase < attack {
        return (phase / attack.max(0.0001)).clamp(0.0, 1.0);
    }

    let decay_end = (attack + decay).min(1.0);
    if phase < decay_end {
        let x = (phase - attack) / decay.max(0.0001);
        return (1.0 - x * (1.0 - sustain)).clamp(0.0, 1.0);
    }

    let release_start = (1.0 - release).max(0.0);
    if phase >= release_start {
        let x = (phase - release_start) / release.max(0.0001);
        return (sustain * (1.0 - x)).clamp(0.0, 1.0);
    }

    sustain.clamp(0.0, 1.0)
}

fn osc_sine(freq: f32, t: f32) -> f32 {
    (std::f32::consts::TAU * freq * t).sin()
}

fn osc_square(freq: f32, t: f32) -> f32 {
    if osc_sine(freq, t) >= 0.0 {
        1.0
    } else {
        -1.0
    }
}

fn osc_saw(freq: f32, t: f32) -> f32 {
    let phase = (freq * t).fract();
    phase * 2.0 - 1.0
}

fn osc_triangle(freq: f32, t: f32) -> f32 {
    let phase = (freq * t).fract();
    2.0 * (2.0 * phase - 1.0).abs() - 1.0
}

fn hash_u64(mut x: u64) -> u64 {
    x ^= x >> 30;
    x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
    x ^ (x >> 31)
}

#[allow(
    clippy::cast_precision_loss,
    reason = "Noise normalization intentionally maps integer hash space into f32 audio space."
)]
fn white_noise(hash: u64) -> f32 {
    let normalized = (hash >> 11) as f32 / ((1_u64 << 53) as f32);
    normalized * 2.0 - 1.0
}
