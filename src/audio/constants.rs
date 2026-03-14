//! Audio-related constants

/// Sample rate for recording (16kHz is optimal for Whisper)
pub const SAMPLE_RATE: u32 = 16000;

/// Number of audio channels (mono)
pub const CHANNELS: u16 = 1;

/// Bits per sample (16-bit PCM)
pub const BITS_PER_SAMPLE: u16 = 16;
