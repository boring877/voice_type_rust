//! Audio recording functionality
//!
//! Contains functions for recording audio from microphone.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::sync::Arc;
use tokio::sync::mpsc;

use super::{BITS_PER_SAMPLE, CHANNELS, RecordingState, SAMPLE_RATE};

/// List all available input device names.
pub fn list_input_devices() -> Result<Vec<String>> {
    let host = cpal::default_host();
    let devices = host
        .input_devices()
        .context("Failed to access input devices")?;

    let mut names = Vec::new();
    for device in devices {
        names.push(
            device
                .name()
                .unwrap_or_else(|_| "Unknown input device".to_string()),
        );
    }

    Ok(names)
}

/// Get a specific input device by index
///
/// If index is None, returns default input device.
pub fn get_device(index: Option<usize>) -> Result<cpal::Device> {
    let host = cpal::default_host();

    if let Some(idx) = index {
        // Get device by index
        let mut devices = host
            .input_devices()
            .context("Failed to access input devices")?;

        devices.nth(idx).context("Invalid microphone index")
    } else {
        // Get default device
        host.default_input_device()
            .context("No default input device available")
    }
}

/// Build the audio stream configuration
///
/// Uses the device's default config. The audio will be resampled
/// to 16kHz if the device uses a different sample rate.
pub fn build_config(
    device: &cpal::Device,
) -> Result<(cpal::StreamConfig, cpal::SampleFormat, u32)> {
    let supported = device
        .default_input_config()
        .context("Failed to get default input config")?;

    tracing::info!("Default input config: {:?}", supported);

    let config: cpal::StreamConfig = supported.config();
    let actual_sample_rate = config.sample_rate.0;

    Ok((config, supported.sample_format(), actual_sample_rate))
}

/// Record audio while the predicate returns true
///
/// This function blocks the current thread while recording.
/// Audio is captured in a callback and stored in memory.
///
/// # Arguments
/// * `mic_index` - Which microphone to use (None = default)
/// * `should_record` - Closure that returns true while recording should continue
/// * `level_tx` - Channel to send audio levels for UI visualization
///
/// # Returns
/// WAV-encoded audio bytes ready for API upload
pub fn record_while<F>(
    mic_index: Option<usize>,
    mut should_record: F,
    level_tx: mpsc::Sender<f32>,
) -> Result<Vec<u8>>
where
    F: FnMut() -> bool,
{
    // Get the audio device
    let device = get_device(mic_index)?;
    tracing::info!("Using input device: {:?}", device.name());

    // Build stream configuration
    let (config, sample_format, actual_sample_rate) = build_config(&device)?;
    tracing::info!(
        "Stream config: {:?}, actual sample rate: {}, channels: {}",
        config,
        actual_sample_rate,
        config.channels
    );

    // Shared recording state
    let state = Arc::new(RecordingState::new());
    let state_clone = Arc::clone(&state);

    // Build the audio stream
    let stream = build_stream(&device, &config, sample_format, state_clone, level_tx)?;

    // Start recording
    stream.play().context("Failed to start audio stream")?;

    tracing::info!("Recording started...");

    // Block while recording
    while should_record() && state.is_running() {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // Stop recording
    state.stop();
    drop(stream); // Ensure stream is dropped before reading samples

    tracing::info!("Recording stopped");

    // Get recorded samples
    let samples = state
        .samples
        .lock()
        .map_err(|_| anyhow::anyhow!("Failed to lock samples"))?;

    tracing::info!(
        "Recorded {} samples at {} Hz",
        samples.len(),
        actual_sample_rate
    );

    // Resample to 16kHz if needed
    let resampled = if actual_sample_rate != SAMPLE_RATE {
        tracing::info!(
            "Resampling from {} Hz to {} Hz",
            actual_sample_rate,
            SAMPLE_RATE
        );
        resample_to_16k(&samples, actual_sample_rate)
    } else {
        samples.to_vec()
    };

    tracing::info!(
        "Final audio: {} samples at {} Hz",
        resampled.len(),
        SAMPLE_RATE
    );

    // Convert to WAV bytes
    let wav_bytes = encode_wav(&resampled, SAMPLE_RATE)?;

    Ok(wav_bytes)
}

/// Build the audio input stream
///
/// Sets up the cpal callback that receives audio data.
pub fn build_stream(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
    state: Arc<RecordingState>,
    level_tx: mpsc::Sender<f32>,
) -> Result<cpal::Stream> {
    let channels = config.channels as usize;

    match sample_format {
        cpal::SampleFormat::I16 => {
            build_stream_inner::<i16>(device, config, state, level_tx, channels)
        }
        cpal::SampleFormat::U16 => {
            build_stream_inner::<u16>(device, config, state, level_tx, channels)
        }
        cpal::SampleFormat::F32 => {
            build_stream_inner::<f32>(device, config, state, level_tx, channels)
        }
        _ => Err(anyhow::anyhow!(
            "Unsupported sample format: {:?}",
            sample_format
        )),
    }
}

/// Inner function to build stream with specific sample type
fn build_stream_inner<T>(
    device: &cpal::Device,
    config: &cpal::StreamConfig,
    state: Arc<RecordingState>,
    level_tx: mpsc::Sender<f32>,
    channels: usize,
) -> Result<cpal::Stream>
where
    T: cpal::SizedSample + Send + 'static,
    i16: cpal::FromSample<T>,
{
    let err_fn = |err| tracing::error!("Audio stream error: {}", err);

    let stream = device
        .build_input_stream(
            config,
            move |data: &[T], _: &cpal::InputCallbackInfo| {
                // Downmix to mono so stereo/default device layouts don't corrupt timing.
                let converted = downmix_to_mono(data, channels);

                // Calculate audio level for visualization (0.0 - 1.0)
                let max_level =
                    converted.iter().map(|&s| s.abs()).max().unwrap_or(0) as f32 / 32768.0;

                // Send level to UI (non-blocking, ignore errors if channel full)
                let _ = level_tx.try_send(max_level.min(1.0));

                // Store samples if still recording
                if state.is_running() {
                    if let Ok(mut samples) = state.samples.lock() {
                        samples.extend_from_slice(&converted);
                    }
                }
            },
            err_fn,
            None,
        )
        .context("Failed to build input stream")?;

    Ok(stream)
}

fn downmix_to_mono<T>(data: &[T], channels: usize) -> Vec<i16>
where
    T: Copy,
    i16: cpal::FromSample<T>,
{
    if channels <= 1 {
        return data
            .iter()
            .map(|&sample| cpal::Sample::from_sample(sample))
            .collect();
    }

    data.chunks(channels)
        .map(|frame| {
            let sample_sum: i32 = frame
                .iter()
                .map(|&sample| {
                    let sample_i16: i16 = cpal::Sample::from_sample(sample);
                    i32::from(sample_i16)
                })
                .sum();
            (sample_sum / frame.len() as i32) as i16
        })
        .collect()
}

/// Resample audio from source rate to target rate (16kHz)
///
/// Simple linear interpolation resampling.
pub fn resample_to_16k(input: &[i16], source_rate: u32) -> Vec<i16> {
    if input.is_empty() || source_rate == 0 || source_rate == SAMPLE_RATE {
        return input.to_vec();
    }

    let ratio = SAMPLE_RATE as f64 / source_rate as f64;
    let output_len = (input.len() as f64 * ratio) as usize;
    let mut output = Vec::with_capacity(output_len);

    for i in 0..output_len {
        let src_idx = i as f64 / ratio;
        let src_idx_floor = src_idx.floor() as usize;
        let src_idx_ceil = (src_idx_floor + 1).min(input.len() - 1);
        let frac = src_idx - src_idx_floor as f64;

        let sample1 = input[src_idx_floor] as f64;
        let sample2 = input[src_idx_ceil] as f64;
        let sample = sample1 + frac * (sample2 - sample1);

        output.push(sample as i16);
    }

    output
}

/// Encode audio samples as WAV bytes
///
/// Uses the `hound` crate for WAV encoding.
pub fn encode_wav(samples: &[i16], sample_rate: u32) -> Result<Vec<u8>> {
    let mut cursor = std::io::Cursor::new(Vec::new());

    let spec = hound::WavSpec {
        channels: CHANNELS,
        sample_rate,
        bits_per_sample: BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer =
        hound::WavWriter::new(&mut cursor, spec).context("Failed to create WAV writer")?;

    for &sample in samples {
        writer
            .write_sample(sample)
            .context("Failed to write sample")?;
    }

    writer.finalize().context("Failed to finalize WAV")?;

    Ok(cursor.into_inner())
}

/// Calculate audio duration from WAV bytes
///
/// Useful for showing recording duration to user.
pub fn wav_duration_seconds(wav_bytes: &[u8]) -> f32 {
    // WAV header is 44 bytes, each sample is 2 bytes (16-bit)
    let data_size = wav_bytes.len().saturating_sub(44);
    let sample_count = data_size / 2;
    sample_count as f32 / SAMPLE_RATE as f32
}

/// Check if recording is long enough to process
///
/// Minimum 0.5 seconds to avoid API calls on accidental presses.
pub fn is_valid_duration(wav_bytes: &[u8]) -> bool {
    wav_duration_seconds(wav_bytes) >= 0.5
}

#[cfg(test)]
mod tests {
    use super::{downmix_to_mono, resample_to_16k};

    #[test]
    fn downmixes_stereo_frames_to_mono() {
        let input = [100_i16, 300, -200, 200];
        assert_eq!(downmix_to_mono(&input, 2), vec![200, 0]);
    }

    #[test]
    fn resampling_empty_audio_is_safe() {
        assert!(resample_to_16k(&[], 48_000).is_empty());
    }
}
