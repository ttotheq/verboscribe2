use std::path::PathBuf;

use verboscribe_core::{AudioCapture, TranscriptionProvider};
use verboscribe_transcription::{WhisperCppConfig, WhisperCppTranscriber};

fn main() {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    let binary = std::env::var("VERBOSCRIBE_WHISPER_CPP_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(&home).join("Developer/whisper.cpp/build/bin/whisper-cli")
        });
    let model = std::env::var("VERBOSCRIBE_WHISPER_CPP_MODEL")
        .map(PathBuf::from)
        .unwrap_or_else(|_| {
            PathBuf::from(&home).join("Developer/whisper.cpp/models/ggml-base.en.bin")
        });
    let sample = std::env::var("VERBOSCRIBE_WHISPER_CPP_SAMPLE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(&home).join("Developer/whisper.cpp/samples/jfk.wav"));

    let mut provider = WhisperCppTranscriber::new(WhisperCppConfig::new(binary, model));
    let transcript = provider
        .transcribe(&AudioCapture {
            path: sample,
            duration_ms: 0,
        })
        .unwrap_or_else(|error| {
            eprintln!("{error}");
            std::process::exit(1);
        });

    println!("{transcript}");
}
