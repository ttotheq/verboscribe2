#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

cargo run -p verboscribe-transcription --example whisper_cpp_smoke
