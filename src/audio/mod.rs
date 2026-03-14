//! Audio recording module
//!
//! Handles microphone input using cpal (Cross-Platform Audio Library).
//! Records audio while hotkey is held, returns WAV-encoded bytes.

// Re-export RecordingState from types
pub use crate::types::audio::RecordingState;

// Declare submodules
mod constants;
mod recorder;

// Re-export constants
pub use constants::{BITS_PER_SAMPLE, CHANNELS, SAMPLE_RATE};

// Re-export functions from recorder
pub use recorder::{is_valid_duration, list_input_devices, record_while, wav_duration_seconds};
