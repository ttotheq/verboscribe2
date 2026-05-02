//! Cross-platform audio recording facade.

use std::path::{Path, PathBuf};

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

fn duration_ms(sample_count: u32, sample_rate: u32) -> u64 {
    if sample_rate == 0 {
        0
    } else {
        (u64::from(sample_count) * 1_000) / u64::from(sample_rate)
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
}
