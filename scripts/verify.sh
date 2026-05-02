#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "${ROOT}"

if ! command -v cargo >/dev/null 2>&1; then
  echo "cargo is required but was not found on PATH." >&2
  exit 1
fi

cargo test --workspace

if command -v npm >/dev/null 2>&1; then
  npm --workspace apps/desktop run build
else
  echo "npm not found; skipping desktop frontend build." >&2
fi
