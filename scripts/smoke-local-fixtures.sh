#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "== WAV validation smoke =="
cargo test -p verboscribe-audio --lib

echo "== Local whisper.cpp provider smoke =="
"$repo_root/scripts/smoke-whisper-cpp.sh"

echo "Local fixture smoke checks passed."
