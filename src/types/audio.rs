//! Audio types for Voice Type
//!
//! Contains types related to audio recording state.

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::audio::SAMPLE_RATE;

/// Audio recording state
///
/// Shared between the audio callback and the control logic.
/// Uses atomic operations for lock-free coordination.
#[derive(Debug)]
pub struct RecordingState {
    /// Set to false to stop recording
    pub running: AtomicBool,
    /// Collected audio samples (interleaved if stereo)
    pub samples: Mutex<Vec<i16>>,
}

impl RecordingState {
    /// Create a new recording state
    pub fn new() -> Self {
        Self {
            running: AtomicBool::new(true),
            samples: Mutex::new(Vec::with_capacity(SAMPLE_RATE as usize * 60)),
        }
    }

    /// Check if recording should continue
    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::Relaxed)
    }

    /// Stop the recording
    pub fn stop(&self) {
        self.running.store(false, Ordering::Relaxed);
    }
}

impl Default for RecordingState {
    fn default() -> Self {
        Self::new()
    }
}
