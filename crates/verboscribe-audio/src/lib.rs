//! Cross-platform audio recording facade.

use std::{
    fs,
    path::{Path, PathBuf},
    sync::{mpsc, Arc, Mutex},
    thread,
    time::{Instant, SystemTime, UNIX_EPOCH},
};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use verboscribe_core::{AudioCapture, AudioRecorder, DictationError};

pub const TARGET_SAMPLE_RATE: u32 = 16_000;
pub const TARGET_CHANNELS: u16 = 1;
pub const TARGET_BITS_PER_SAMPLE: u16 = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WavInfo {
    pub path: PathBuf,
    pub duration_ms: u64,
    pub sample_rate: u32,
    pub channels: u16,
    pub bits_per_sample: u16,
    pub sample_count: u32,
}

#[derive(Debug, thiserror::Error)]
pub enum AudioError {
    #[error("audio file not found at {0}")]
    MissingFile(PathBuf),
    #[error("could not read WAV file {path}: {source}")]
    ReadWav { path: PathBuf, source: hound::Error },
    #[error("could not write WAV file {path}: {source}")]
    WriteWav { path: PathBuf, source: hound::Error },
    #[error("could not create recording directory {path}: {source}")]
    CreateDirectory {
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("WAV must be mono, got {0} channel(s)")]
    UnsupportedChannels(u16),
    #[error("WAV must be 16000 Hz, got {0} Hz")]
    UnsupportedSampleRate(u32),
    #[error("WAV must be 16-bit PCM, got {bits_per_sample}-bit {sample_format:?}")]
    UnsupportedSampleFormat {
        bits_per_sample: u16,
        sample_format: hound::SampleFormat,
    },
}

pub struct CpalAudioRecorder {
    output_dir: PathBuf,
    state: RecorderState,
}

enum RecorderState {
    Idle,
    Recording(RecordingSession),
}

struct RecordingSession {
    controller: Box<dyn RecordingController + Send>,
    started_at: Instant,
    capture_config: CaptureConfig,
    samples: Arc<Mutex<Vec<f32>>>,
}

#[derive(Debug, Clone)]
struct CaptureConfig {
    path: PathBuf,
    sample_rate: u32,
    channels: u16,
}

trait RecordingController {
    fn stop(self: Box<Self>) -> Result<(), DictationError>;
    fn cancel(self: Box<Self>) -> Result<(), DictationError>;
}

struct ThreadedRecordingController {
    command_tx: mpsc::Sender<RecorderThreadCommand>,
    join_handle: thread::JoinHandle<()>,
}

enum RecorderThreadCommand {
    Stop,
    Cancel,
}

impl RecordingController for ThreadedRecordingController {
    fn stop(self: Box<Self>) -> Result<(), DictationError> {
        self.command_tx
            .send(RecorderThreadCommand::Stop)
            .map_err(|_| {
                DictationError::Recording("recording thread stopped unexpectedly".to_string())
            })?;
        self.join_handle
            .join()
            .map_err(|_| DictationError::Recording("recording thread panicked".to_string()))?;
        Ok(())
    }

    fn cancel(self: Box<Self>) -> Result<(), DictationError> {
        let _ = self.command_tx.send(RecorderThreadCommand::Cancel);
        self.join_handle
            .join()
            .map_err(|_| DictationError::Recording("recording thread panicked".to_string()))?;
        Ok(())
    }
}

impl Default for CpalAudioRecorder {
    fn default() -> Self {
        Self::new(std::env::temp_dir().join("verboscribe2"))
    }
}

impl CpalAudioRecorder {
    pub fn new(output_dir: impl Into<PathBuf>) -> Self {
        Self {
            output_dir: output_dir.into(),
            state: RecorderState::Idle,
        }
    }

    pub fn output_dir(&self) -> &Path {
        &self.output_dir
    }

    fn default_input_device(&self) -> Result<cpal::Device, DictationError> {
        cpal::default_host().default_input_device().ok_or_else(|| {
            DictationError::Recording("no default input device available".to_string())
        })
    }

    fn supported_input_config(
        &self,
        device: &cpal::Device,
    ) -> Result<cpal::SupportedStreamConfig, DictationError> {
        device
            .default_input_config()
            .map_err(map_default_input_config_error)
    }

    fn begin_session(&self) -> Result<RecordingSession, DictationError> {
        fs::create_dir_all(&self.output_dir).map_err(|source| {
            DictationError::Recording(
                AudioError::CreateDirectory {
                    path: self.output_dir.clone(),
                    source,
                }
                .to_string(),
            )
        })?;

        let samples = Arc::new(Mutex::new(Vec::new()));
        let path = next_recording_path(&self.output_dir);
        let (command_tx, command_rx) = mpsc::channel();
        let (ready_tx, ready_rx) = mpsc::sync_channel(1);
        let thread_samples = Arc::clone(&samples);
        let thread_path = path.clone();
        let join_handle = thread::spawn(move || {
            let ready = setup_recording_thread(thread_samples, thread_path);
            match ready {
                Ok((capture_config, stream)) => {
                    let _ = ready_tx.send(Ok(capture_config));
                    match command_rx.recv() {
                        Ok(RecorderThreadCommand::Stop) => {
                            let _ = stream.pause();
                        }
                        Ok(RecorderThreadCommand::Cancel) | Err(_) => {}
                    }
                }
                Err(error) => {
                    let _ = ready_tx.send(Err(error));
                }
            }
        });
        let capture_config = ready_rx.recv().map_err(|_| {
            DictationError::Recording("recording thread exited before setup".to_string())
        })??;

        Ok(RecordingSession {
            controller: Box::new(ThreadedRecordingController {
                command_tx,
                join_handle,
            }),
            started_at: Instant::now(),
            capture_config,
            samples,
        })
    }
}

impl AudioRecorder for CpalAudioRecorder {
    fn request_permission(&mut self) -> Result<(), DictationError> {
        let device = self.default_input_device()?;
        let _config = self.supported_input_config(&device)?;
        Ok(())
    }

    fn start(&mut self) -> Result<(), DictationError> {
        if matches!(self.state, RecorderState::Recording(_)) {
            return Err(DictationError::Recording(
                "recording already active".to_string(),
            ));
        }

        self.request_permission()?;
        let session = self.begin_session()?;
        self.state = RecorderState::Recording(session);
        Ok(())
    }

    fn stop(&mut self) -> Result<AudioCapture, DictationError> {
        let session = match std::mem::replace(&mut self.state, RecorderState::Idle) {
            RecorderState::Idle => {
                return Err(DictationError::Recording(
                    "recording was not active".to_string(),
                ))
            }
            RecorderState::Recording(session) => session,
        };

        let RecordingSession {
            controller,
            started_at,
            capture_config,
            samples,
        } = session;
        controller.stop()?;

        let captured_samples = samples
            .lock()
            .map_err(|_| {
                DictationError::Recording("recording buffer lock was poisoned".to_string())
            })?
            .clone();
        let normalized_samples = normalize_captured_samples(
            &captured_samples,
            capture_config.channels,
            capture_config.sample_rate,
        );
        let did_capture_audio = !normalized_samples.is_empty();
        let wav = write_mono_i16_wav(
            &capture_config.path,
            &normalized_samples,
            TARGET_SAMPLE_RATE,
        )
        .map_err(map_audio_error)?;
        let duration_ms = if did_capture_audio {
            wav.duration_ms
                .max(started_at.elapsed().as_millis() as u64)
                .max(1)
        } else {
            0
        };

        Ok(AudioCapture {
            path: wav.path,
            duration_ms,
        })
    }

    fn cancel(&mut self) {
        if let RecorderState::Recording(session) =
            std::mem::replace(&mut self.state, RecorderState::Idle)
        {
            let _ = session.controller.cancel();
        }
    }
}

pub fn write_mono_i16_wav(
    path: impl AsRef<Path>,
    samples: &[i16],
    sample_rate: u32,
) -> Result<WavInfo, AudioError> {
    let path = path.as_ref();
    let spec = hound::WavSpec {
        channels: TARGET_CHANNELS,
        sample_rate,
        bits_per_sample: TARGET_BITS_PER_SAMPLE,
        sample_format: hound::SampleFormat::Int,
    };
    let mut writer =
        hound::WavWriter::create(path, spec).map_err(|source| AudioError::WriteWav {
            path: path.to_path_buf(),
            source,
        })?;

    for sample in samples {
        writer
            .write_sample(*sample)
            .map_err(|source| AudioError::WriteWav {
                path: path.to_path_buf(),
                source,
            })?;
    }

    writer.finalize().map_err(|source| AudioError::WriteWav {
        path: path.to_path_buf(),
        source,
    })?;

    validate_wav_for_transcription(path)
}

pub fn validate_wav_for_transcription(path: impl AsRef<Path>) -> Result<WavInfo, AudioError> {
    let path = path.as_ref();
    if !path.is_file() {
        return Err(AudioError::MissingFile(path.to_path_buf()));
    }

    let reader = hound::WavReader::open(path).map_err(|source| AudioError::ReadWav {
        path: path.to_path_buf(),
        source,
    })?;
    let spec = reader.spec();

    if spec.channels != TARGET_CHANNELS {
        return Err(AudioError::UnsupportedChannels(spec.channels));
    }
    if spec.sample_rate != TARGET_SAMPLE_RATE {
        return Err(AudioError::UnsupportedSampleRate(spec.sample_rate));
    }
    if spec.bits_per_sample != TARGET_BITS_PER_SAMPLE
        || spec.sample_format != hound::SampleFormat::Int
    {
        return Err(AudioError::UnsupportedSampleFormat {
            bits_per_sample: spec.bits_per_sample,
            sample_format: spec.sample_format,
        });
    }

    let sample_count = reader.duration();
    Ok(WavInfo {
        path: path.to_path_buf(),
        duration_ms: duration_ms(sample_count, spec.sample_rate),
        sample_rate: spec.sample_rate,
        channels: spec.channels,
        bits_per_sample: spec.bits_per_sample,
        sample_count,
    })
}

pub fn f32_sample_to_i16(sample: f32) -> i16 {
    let clamped = sample.clamp(-1.0, 1.0);
    if clamped >= 0.0 {
        (clamped * i16::MAX as f32).round() as i16
    } else {
        (clamped * -(i16::MIN as f32)).round() as i16
    }
}

fn build_input_stream(
    device: cpal::Device,
    config: &cpal::SupportedStreamConfig,
    samples: Arc<Mutex<Vec<f32>>>,
) -> Result<cpal::Stream, DictationError> {
    let stream_config: cpal::StreamConfig = config.clone().into();
    let error_callback = |error| eprintln!("audio input stream error: {error}");

    match config.sample_format() {
        cpal::SampleFormat::F32 => device
            .build_input_stream(
                &stream_config,
                move |data: &[f32], _| append_f32_samples(&samples, data),
                error_callback,
                None,
            )
            .map_err(map_build_stream_error),
        cpal::SampleFormat::I16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[i16], _| append_i16_samples(&samples, data),
                error_callback,
                None,
            )
            .map_err(map_build_stream_error),
        cpal::SampleFormat::U16 => device
            .build_input_stream(
                &stream_config,
                move |data: &[u16], _| append_u16_samples(&samples, data),
                error_callback,
                None,
            )
            .map_err(map_build_stream_error),
        unsupported => Err(DictationError::Recording(format!(
            "unsupported input sample format: {unsupported:?}"
        ))),
    }
}

fn setup_recording_thread(
    samples: Arc<Mutex<Vec<f32>>>,
    path: PathBuf,
) -> Result<(CaptureConfig, cpal::Stream), DictationError> {
    let host = cpal::default_host();
    let device = host.default_input_device().ok_or_else(|| {
        DictationError::Recording("no default input device available".to_string())
    })?;
    let config = device
        .default_input_config()
        .map_err(map_default_input_config_error)?;
    let capture_config = CaptureConfig {
        path,
        sample_rate: config.sample_rate().0,
        channels: config.channels(),
    };
    let stream = build_input_stream(device, &config, samples)?;
    stream.play().map_err(map_play_stream_error)?;
    Ok((capture_config, stream))
}

fn append_f32_samples(shared: &Arc<Mutex<Vec<f32>>>, data: &[f32]) {
    if let Ok(mut samples) = shared.lock() {
        samples.extend_from_slice(data);
    }
}

fn append_i16_samples(shared: &Arc<Mutex<Vec<f32>>>, data: &[i16]) {
    if let Ok(mut samples) = shared.lock() {
        samples.extend(data.iter().map(|sample| *sample as f32 / i16::MAX as f32));
    }
}

fn append_u16_samples(shared: &Arc<Mutex<Vec<f32>>>, data: &[u16]) {
    if let Ok(mut samples) = shared.lock() {
        samples.extend(
            data.iter()
                .map(|sample| (*sample as f32 / u16::MAX as f32) * 2.0 - 1.0),
        );
    }
}

fn normalize_captured_samples(samples: &[f32], channels: u16, source_rate: u32) -> Vec<i16> {
    let mono = downmix_to_mono(samples, channels);
    let resampled = if source_rate == TARGET_SAMPLE_RATE || mono.is_empty() {
        mono
    } else {
        resample_linear(&mono, source_rate, TARGET_SAMPLE_RATE)
    };
    resampled.into_iter().map(f32_sample_to_i16).collect()
}

fn downmix_to_mono(samples: &[f32], channels: u16) -> Vec<f32> {
    if channels <= 1 {
        return samples.to_vec();
    }

    let channels = channels as usize;
    samples
        .chunks(channels)
        .map(|frame| frame.iter().copied().sum::<f32>() / frame.len() as f32)
        .collect()
}

fn resample_linear(samples: &[f32], source_rate: u32, target_rate: u32) -> Vec<f32> {
    if samples.is_empty() || source_rate == 0 || target_rate == 0 {
        return Vec::new();
    }
    if source_rate == target_rate || samples.len() == 1 {
        return samples.to_vec();
    }

    let output_len =
        ((samples.len() as u64 * target_rate as u64) / source_rate as u64).max(1) as usize;
    let ratio = source_rate as f64 / target_rate as f64;

    let mut resampled = Vec::with_capacity(output_len);
    for output_index in 0..output_len {
        let source_position = output_index as f64 * ratio;
        let lower_index = source_position.floor() as usize;
        let upper_index = lower_index
            .saturating_add(1)
            .min(samples.len().saturating_sub(1));
        let fraction = (source_position - lower_index as f64) as f32;
        let lower = samples[lower_index.min(samples.len() - 1)];
        let upper = samples[upper_index];
        resampled.push(lower + (upper - lower) * fraction);
    }

    resampled
}

fn duration_ms(sample_count: u32, sample_rate: u32) -> u64 {
    if sample_rate == 0 {
        0
    } else {
        (u64::from(sample_count) * 1_000) / u64::from(sample_rate)
    }
}

fn next_recording_path(output_dir: &Path) -> PathBuf {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    let pid = std::process::id();
    output_dir.join(format!("recording-{timestamp}-{pid}.wav"))
}

fn map_audio_error(error: AudioError) -> DictationError {
    DictationError::Recording(error.to_string())
}

fn map_default_input_config_error(error: cpal::DefaultStreamConfigError) -> DictationError {
    map_cpal_message(error.to_string())
}

fn map_build_stream_error(error: cpal::BuildStreamError) -> DictationError {
    map_cpal_message(error.to_string())
}

fn map_play_stream_error(error: cpal::PlayStreamError) -> DictationError {
    map_cpal_message(error.to_string())
}

fn map_cpal_message(message: String) -> DictationError {
    let lowered = message.to_ascii_lowercase();
    if lowered.contains("permission")
        || lowered.contains("denied")
        || lowered.contains("not authorized")
        || lowered.contains("not permitted")
        || lowered.contains("unauthorized")
    {
        DictationError::MicrophonePermissionDenied
    } else {
        DictationError::Recording(message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_mono_i16_wav_creates_valid_transcription_wav() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("recording.wav");
        let samples = vec![0_i16; TARGET_SAMPLE_RATE as usize];

        let info = write_mono_i16_wav(&path, &samples, TARGET_SAMPLE_RATE).unwrap();

        assert_eq!(info.path, path);
        assert_eq!(info.sample_rate, TARGET_SAMPLE_RATE);
        assert_eq!(info.channels, TARGET_CHANNELS);
        assert_eq!(info.bits_per_sample, TARGET_BITS_PER_SAMPLE);
        assert_eq!(info.sample_count, TARGET_SAMPLE_RATE);
        assert_eq!(info.duration_ms, 1_000);
    }

    #[test]
    fn validate_rejects_missing_file() {
        let error = validate_wav_for_transcription("/missing/audio.wav").unwrap_err();

        assert!(matches!(error, AudioError::MissingFile(_)));
    }

    #[test]
    fn validate_rejects_stereo_wav() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("stereo.wav");
        write_test_wav(
            &path,
            2,
            TARGET_SAMPLE_RATE,
            TARGET_BITS_PER_SAMPLE,
            hound::SampleFormat::Int,
        );

        let error = validate_wav_for_transcription(&path).unwrap_err();

        assert!(matches!(error, AudioError::UnsupportedChannels(2)));
    }

    #[test]
    fn validate_rejects_wrong_sample_rate() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("rate.wav");
        write_test_wav(
            &path,
            TARGET_CHANNELS,
            48_000,
            TARGET_BITS_PER_SAMPLE,
            hound::SampleFormat::Int,
        );

        let error = validate_wav_for_transcription(&path).unwrap_err();

        assert!(matches!(error, AudioError::UnsupportedSampleRate(48_000)));
    }

    #[test]
    fn validate_rejects_float_wav() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("float.wav");
        let spec = hound::WavSpec {
            channels: TARGET_CHANNELS,
            sample_rate: TARGET_SAMPLE_RATE,
            bits_per_sample: 32,
            sample_format: hound::SampleFormat::Float,
        };
        let mut writer = hound::WavWriter::create(&path, spec).unwrap();
        writer.write_sample(0.0_f32).unwrap();
        writer.finalize().unwrap();

        let error = validate_wav_for_transcription(&path).unwrap_err();

        assert!(matches!(error, AudioError::UnsupportedSampleFormat { .. }));
    }

    #[test]
    fn f32_sample_conversion_clamps_without_overflow() {
        assert_eq!(f32_sample_to_i16(2.0), i16::MAX);
        assert_eq!(f32_sample_to_i16(-2.0), i16::MIN);
        assert_eq!(f32_sample_to_i16(0.0), 0);
    }

    #[test]
    fn downmix_to_mono_averages_each_frame() {
        let mono = downmix_to_mono(&[0.0, 1.0, 0.5, -0.5], 2);

        assert_eq!(mono, vec![0.5, 0.0]);
    }

    #[test]
    fn resample_linear_changes_sample_count_for_target_rate() {
        let resampled = resample_linear(&[0.0, 1.0, 0.0, -1.0], 8_000, 16_000);

        assert_eq!(resampled.len(), 8);
        assert!((resampled[1] - 0.5).abs() < 0.001);
    }

    #[test]
    fn normalize_capture_downmixes_and_resamples_to_target_contract() {
        let normalized = normalize_captured_samples(&[0.0, 1.0, 1.0, 0.0], 2, 8_000);

        assert_eq!(normalized.len(), 4);
        assert_eq!(normalized[0], f32_sample_to_i16(0.5));
    }

    #[test]
    fn stop_without_start_returns_recording_error() {
        let directory = tempfile::tempdir().unwrap();
        let mut recorder = CpalAudioRecorder::new(directory.path());

        let error = recorder.stop().unwrap_err();

        assert!(matches!(error, DictationError::Recording(_)));
    }

    #[test]
    fn duplicate_start_returns_recording_error_when_session_is_active() {
        let directory = tempfile::tempdir().unwrap();
        let mut recorder = CpalAudioRecorder::new(directory.path());
        recorder.state = RecorderState::Recording(fake_recording_session(
            directory.path().join("active.wav"),
            Vec::new(),
            TARGET_SAMPLE_RATE,
            TARGET_CHANNELS,
        ));

        let error = recorder.start().unwrap_err();

        assert!(matches!(error, DictationError::Recording(_)));
    }

    #[test]
    fn stop_writes_transcription_ready_wav_from_captured_samples() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("captured.wav");
        let mut recorder = CpalAudioRecorder::new(directory.path());
        recorder.state = RecorderState::Recording(fake_recording_session(
            path.clone(),
            vec![0.0, 1.0, 1.0, 0.0],
            8_000,
            2,
        ));

        let capture = recorder.stop().unwrap();
        let info = validate_wav_for_transcription(&capture.path).unwrap();

        assert_eq!(capture.path, path);
        assert_eq!(info.sample_rate, TARGET_SAMPLE_RATE);
        assert_eq!(info.channels, TARGET_CHANNELS);
        assert!(capture.duration_ms > 0);
    }

    #[test]
    fn permission_like_messages_map_to_permission_denied() {
        let error = map_cpal_message("permission denied by host".to_string());

        assert_eq!(error, DictationError::MicrophonePermissionDenied);
    }

    #[test]
    fn cancel_resets_active_state() {
        let directory = tempfile::tempdir().unwrap();
        let mut recorder = CpalAudioRecorder::new(directory.path());
        recorder.state = RecorderState::Recording(fake_recording_session(
            directory.path().join("cancel.wav"),
            Vec::new(),
            TARGET_SAMPLE_RATE,
            TARGET_CHANNELS,
        ));

        recorder.cancel();

        assert!(matches!(recorder.state, RecorderState::Idle));
    }

    fn write_test_wav(
        path: &Path,
        channels: u16,
        sample_rate: u32,
        bits_per_sample: u16,
        sample_format: hound::SampleFormat,
    ) {
        let spec = hound::WavSpec {
            channels,
            sample_rate,
            bits_per_sample,
            sample_format,
        };
        let mut writer = hound::WavWriter::create(path, spec).unwrap();
        for _ in 0..channels {
            writer.write_sample(0_i16).unwrap();
        }
        writer.finalize().unwrap();
    }

    fn fake_recording_session(
        path: PathBuf,
        samples: Vec<f32>,
        sample_rate: u32,
        channels: u16,
    ) -> RecordingSession {
        RecordingSession {
            controller: Box::new(NoopController),
            started_at: Instant::now(),
            capture_config: CaptureConfig {
                path,
                sample_rate,
                channels,
            },
            samples: Arc::new(Mutex::new(samples)),
        }
    }

    struct NoopController;

    impl RecordingController for NoopController {
        fn stop(self: Box<Self>) -> Result<(), DictationError> {
            Ok(())
        }

        fn cancel(self: Box<Self>) -> Result<(), DictationError> {
            Ok(())
        }
    }
}
