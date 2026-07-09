# Paste-Last Hotkey Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Add a dedicated, configurable paste-last hotkey so users can retry inserting the preserved transcript without recording again.

**Architecture:** Extend the existing three-layer hotkey flow already used by the dictation and toggle hotkeys: persisted settings in `verboscribe-storage`, status/command handling in `app_service.rs`, and registration/routing in `hotkeys.rs`. Surface the new hotkey in the desktop shell so the shipped slice is configurable and visible.

**Tech Stack:** Rust workspace (`cargo test`, `cargo fmt`), Tauri 2 backend, TypeScript/Vite desktop UI.

---

### Task 1: Add the backend paste-last hotkey slice

**Objective:** Persist a paste-last hotkey and make the runtime invoke `paste_last_transcript` from a dedicated global shortcut.

**Files:**
- Modify: `crates/verboscribe-storage/src/lib.rs`
- Modify: `apps/desktop/src-tauri/src/app_service.rs`
- Modify: `apps/desktop/src-tauri/src/hotkeys.rs`
- Test: `crates/verboscribe-storage/src/lib.rs`
- Test: `apps/desktop/src-tauri/src/app_service.rs`
- Test: `apps/desktop/src-tauri/src/hotkeys.rs`

**Step 1: Write failing test**

Add a new app-service test that proves a dedicated paste-last hotkey can retry a preserved transcript after the initial paste failure.

**Step 2: Run test to verify failure**

Run: `cargo test -p verboscribe2-desktop paste_last_hotkey_retries_a_preserved_transcript -- --nocapture`
Expected: FAIL because the paste-last hotkey role/handler does not exist yet.

**Step 3: Write minimal implementation**

Add a `PasteLast` hotkey role, runtime state tracking, settings plumbed through `SettingsDto`, and hotkey registration/dispatch that calls `AppService::paste_last_transcript()` on the key press.

**Step 4: Run test to verify pass**

Run: `cargo test -p verboscribe2-desktop paste_last_hotkey_retries_a_preserved_transcript -- --nocapture`
Expected: PASS

**Step 5: Add persistence coverage**

Add storage tests that prove the new hotkey gets a default value and older settings files still load with a backward-compatible fallback.

**Step 6: Run targeted regression checks**

Run:
- `cargo test -p verboscribe-storage defaults_document_provider_language_mode_and_hotkey -- --nocapture`
- `cargo test -p verboscribe-storage load_legacy_settings_without_prompt_fields_uses_defaults -- --nocapture`
- `cargo test -p verboscribe2-desktop app_status_returns_shell_defaults -- --nocapture`
- `cargo test -p verboscribe2-desktop app_status_reports_toggle_hotkey_registration_independently -- --nocapture`
- `cargo test -p verboscribe2-desktop normalize_default_hotkey_for_plugin_registration -- --nocapture`

**Step 7: Commit**

```bash
git add crates/verboscribe-storage/src/lib.rs apps/desktop/src-tauri/src/app_service.rs apps/desktop/src-tauri/src/hotkeys.rs
git commit -m "feat(desktop): add paste-last hotkey"
```

### Task 2: Surface the slice in the desktop shell and docs

**Objective:** Make the new hotkey visible, editable, and documented.

**Files:**
- Modify: `apps/desktop/src/main.ts`
- Modify: `docs/FEATURE_LIST.md`
- Modify: `docs/MANUAL_QA.md`
- Modify: `docs/BACKLOG.md`
- Modify: `docs/SPRINTS.md`
- Modify: `HANDOFF.md`

**Step 1: Write failing test**

Reuse the backend-facing settings/status tests so the UI has a real persisted field to bind to before changing frontend code.

**Step 2: Run test to verify failure**

Run: `cargo test -p verboscribe2-desktop app_status_returns_shell_defaults -- --nocapture`
Expected: FAIL before the new status/settings field exists.

**Step 3: Write minimal implementation**

Add the paste-last hotkey field to the settings form, live-status metrics, and relevant copy. Update docs to mark the dedicated paste-last hotkey as implemented and leave retry-last-failed-transcript as still missing.

**Step 4: Run test/build verification**

Run:
- `cargo test -p verboscribe2-desktop app_status_returns_shell_defaults -- --nocapture`
- `./scripts/verify.sh`

Expected: PASS

**Step 5: Commit**

```bash
git add apps/desktop/src/main.ts docs/FEATURE_LIST.md docs/MANUAL_QA.md docs/BACKLOG.md docs/SPRINTS.md HANDOFF.md docs/plans/2026-07-04-paste-last-hotkey.md
git commit -m "docs: record paste-last hotkey slice"
```