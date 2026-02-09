#![forbid(unsafe_code)]
//! # AstraWeave Audio
//!
//! Spatial audio engine with dialogue runtime and text-to-speech adapter.
//!
//! Built on **rodio**, this crate provides:
//!
//! - **[`engine::AudioEngine`]** — 4-bus mixer (master, music, SFX, voice) with 3D
//!   spatial panning and distance attenuation.
//! - **[`dialogue_runtime::DialoguePlayer`]** — Dialogue audio playback with
//!   character-mapped audio banks ([`DialogueAudioMap`]).
//! - **[`voice::VoiceBank`]** — Voice sample management and TTS adapter trait.
//!
//! # Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `mock_tts` | Enables `SimpleSineTts` for testing without real TTS |

pub mod dialogue_runtime;
pub mod engine;
pub mod voice;

#[cfg(test)]
mod mutation_tests;

pub use dialogue_runtime::{load_dialogue_audio_map, DialogueAudioMap, DialoguePlayer};
pub use engine::{AudioEngine, EmitterId, ListenerPose, MusicTrack, PanMode};
#[cfg(feature = "mock_tts")]
pub use voice::SimpleSineTts;
pub use voice::{load_voice_bank, TtsAdapter, VoiceBank, VoiceSpec};
