//! Async task implementations
//!
//! Contains the background async tasks that run concurrently:
//! - Audio recording task: monitors hotkey and records audio  
//! - Transcription task: sends audio to API and types result

pub mod recording;
pub mod transcription;

// Re-export the task functions
pub use recording::audio_recording_task;
pub use transcription::transcription_task;
