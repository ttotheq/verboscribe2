#!/usr/bin/env bash
set -euo pipefail

MODE="${1:-run}"
APP_NAME="VerboScribe 2"
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
APP_BUNDLE="$ROOT_DIR/target/debug/bundle/macos/$APP_NAME.app"
DEFAULT_SIGNING_IDENTITIES=(
  "VerboScribe Local Code Signing"
  "Whisper Dictation Local Code Signing"
)

resolve_app_executable() {
  if [[ -f "$APP_BUNDLE/Contents/Info.plist" ]]; then
    /usr/libexec/PlistBuddy -c "Print :CFBundleExecutable" \
      "$APP_BUNDLE/Contents/Info.plist"
  else
    printf '%s\n' "verboscribe2-desktop"
  fi
}

has_codesign_identity() {
  local identity="$1"
  security find-identity -v -p codesigning | grep -Fq "\"$identity\""
}

resolve_signing_identity() {
  if [[ "${VERBOSCRIBE_CODESIGN_IDENTITY:-}" == "-" ]]; then
    return 0
  fi

  if [[ -n "${VERBOSCRIBE_CODESIGN_IDENTITY:-}" ]]; then
    if has_codesign_identity "$VERBOSCRIBE_CODESIGN_IDENTITY"; then
      printf '%s\n' "$VERBOSCRIBE_CODESIGN_IDENTITY"
      return 0
    fi

    echo "Signing identity '${VERBOSCRIBE_CODESIGN_IDENTITY}' is not available." >&2
    exit 1
  fi

  local identity
  for identity in "${DEFAULT_SIGNING_IDENTITIES[@]}"; do
    if has_codesign_identity "$identity"; then
      printf '%s\n' "$identity"
      return 0
    fi
  done
}

sign_app_bundle() {
  local identity="$1"
  if [[ -z "$identity" ]]; then
    echo "No stable signing identity found; leaving app ad-hoc signed." >&2
    return 0
  fi

  echo "Signing app with '$identity'..."
  codesign --force --deep --sign "$identity" "$APP_BUNDLE"
}

APP_EXECUTABLE="$(resolve_app_executable)"

pkill -x "$APP_NAME" >/dev/null 2>&1 || true
pkill -x "$APP_EXECUTABLE" >/dev/null 2>&1 || true

cd "$ROOT_DIR"
npm --workspace apps/desktop run tauri build -- --debug

if [[ ! -d "$APP_BUNDLE" ]]; then
  echo "expected app bundle at $APP_BUNDLE" >&2
  exit 1
fi

APP_EXECUTABLE="$(resolve_app_executable)"
APP_BINARY="$APP_BUNDLE/Contents/MacOS/$APP_EXECUTABLE"

if [[ ! -x "$APP_BINARY" ]]; then
  echo "expected app executable at $APP_BINARY" >&2
  exit 1
fi

SIGNING_IDENTITY="$(resolve_signing_identity)"
sign_app_bundle "$SIGNING_IDENTITY"

open_app() {
  /usr/bin/open -n "$APP_BUNDLE"
}

case "$MODE" in
  run)
    open_app
    ;;
  --debug|debug)
    lldb -- "$APP_BINARY"
    ;;
  --logs|logs)
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --telemetry|telemetry)
    open_app
    /usr/bin/log stream --info --style compact --predicate "process == \"$APP_NAME\""
    ;;
  --verify|verify)
    open_app
    sleep 2
    pgrep -x "$APP_EXECUTABLE" >/dev/null
    ;;
  *)
    echo "usage: $0 [run|--debug|--logs|--telemetry|--verify]" >&2
    exit 2
    ;;
esac
