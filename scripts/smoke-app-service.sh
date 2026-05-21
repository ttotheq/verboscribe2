#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

echo "== App-service dictation smoke =="
cargo test -p verboscribe2-desktop smoke_dictation_cycle_

echo "App-service smoke checks passed."
