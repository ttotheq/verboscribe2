# Feature List

This document is the current product-scope inventory for VerboScribe 2.

It exists because the feature scope was previously split across:

- `docs/PROTOTYPE_NOTES.md`
- `docs/EPICS.md`
- `docs/BACKLOG.md`
- `docs/ROADMAP.md`

Use this file as the fastest answer to:

- what the original prototype already did,
- what `verboscribe2` currently does,
- what is backend-only versus user-visible,
- what UI surface is still missing.

## Prototype Reference

The read-only macOS prototype in `~/projects/whisper` includes these product
features:

- microphone recording
- local `whisper.cpp` transcription
- Groq Whisper transcription
- press-and-hold and toggle dictation modes
- configurable dictation hotkey
- configurable cancel hotkey
- configurable paste-last hotkey
- target-app tracking
- clipboard-first paste insertion
- paste-last and paste-raw actions
- retry last failed transcript
- preview/edit-before-insert
- raw-first transcript behavior
- cleanup levels
- spoken commands
- style presets
- snippets
- personal dictionary prompt hints
- transcript history with privacy modes
- aggregate usage insights
- settings import/export excluding secrets
- launch at login
- open minimized at launch
- sound feedback
- local model selection plus install/refresh actions
- language selection
- prompt/context editing
- min/max recording duration controls
- status and recovery messaging
- accessibility shortcut/help actions
- release notes and manual QA support

## Current VerboScribe 2

### End-to-end implemented

- desktop shell launches on macOS and Windows
- desktop settings surface now exists for the current local-first stack
- app-service live dictation flow exists
- local `whisper.cpp` provider works
- live CPAL microphone recording works
- press-and-hold and toggle dictation modes work
- global dictation hotkey works
- target-app tracking works
- clipboard-first paste insertion works
- paste failure preserves transcript for manual paste
- runtime status and recovery state exist
- last transcript is retained in app state
- manual start, stop, and cancel controls exist in the desktop shell
- prompt/context plus pinned-term settings persist
- app-service smoke path exists
- local fixture smoke path exists

### Implemented in backend, only partially surfaced in UI

- minimum recording duration persistence
- transcript-processing capability in core for cleanup, spoken commands, style
  presets, snippets, and dictionary hints

### Partially implemented

- macOS real-desktop QA is largely complete, but recognition quality still
  needs hardening on short phrases and product names
- Windows adapters exist, but real-desktop QA is still pending
- app icon/bundle integration is improved, but branding exploration is not done

### Missing compared with the prototype

- cancel hotkey
- paste-last hotkey

The desktop shell now includes a `Paste last transcript` user action for
recovering a preserved transcript after paste failure, but it still lacks a
dedicated global paste-last hotkey and a separate retry-last-failed-transcript
workflow.
- paste-raw user action
- retry last failed transcript user action
- preview/edit-before-insert flow
- cleanup-level controls
- spoken-command controls
- style-preset controls
- snippets management UI
- personal-dictionary management UI
- Groq provider support
- Groq API key storage and UI
- transcript history store and UI
- usage insights store and UI
- settings import/export
- launch-at-login
- open-minimized preference
- start/stop sound feedback
- local model catalog/install/refresh UI
- min/max recording range controls
- accessibility helper actions in the UI
- hotkey reference/help surface
- release notes surface

## GUI Gap

### Current `verboscribe2` desktop UI

The Tauri UI at `apps/desktop/src/main.ts` currently shows:

- app title
- high-level state badge
- editable `whisper.cpp` binary and model paths
- editable language, dictation mode, and dictation hotkey
- editable prompt context and pinned terms
- live provider, mode, hotkey, and recovery panels
- manual start, stop, and cancel buttons
- usage hint
- last transcript

This is now a real settings-and-status foundation, but it is still much
smaller than the prototype's full operations surface. Current save semantics
are explicit: form edits are drafts until the user clicks
`Save settings to apply`.

### Prototype GUI

The macOS prototype `MainViewController.swift` had a much broader interface:

- a large settings form
- direct start, stop, and cancel buttons
- dictation, cancel, and paste-last hotkey recording
- provider, cleanup, style, command-mode, and review controls
- transcript history controls
- local model selection plus install/refresh buttons
- prompt/context, dictionary, and snippets fields
- min/max recording duration fields
- Groq key/model fields
- preview action buttons
- retry and recovery action buttons
- settings import/export actions
- launch-at-login and open-login-items actions
- transcript pane
- usage-insights pane
- a separate hotkey-reference tab

The user is correct that `verboscribe2` is not close to that interface yet.

## Recommended Product View

Treat the feature set in three layers:

### Layer 1: Vertical dictation loop

- hotkey
- record
- transcribe
- paste
- recovery

### Layer 2: Dictation usefulness

- cleanup
- spoken commands
- style presets
- snippets
- dictionary hints
- preview/edit-before-insert
- retry and paste-last recovery tools

### Layer 3: Product operations and memory

- provider switching
- model management
- settings import/export
- history
- insights
- launch behavior
- release and QA utilities

## Near-term UI Priorities

If the goal is to close the obvious product gap quickly, the next high-value UI
surfaces are:

1. a real settings screen for provider paths, language, dictation mode, hotkey,
   prompt context, and pinned terms
2. transcript action controls for paste last, retry, and preview-before-insert
3. text-processing controls for cleanup, spoken commands, style, snippets, and
   dictionary terms
4. operations controls for history, insights, import/export, and launch at
   login

## Source References

- prototype behavior summary: `docs/PROTOTYPE_NOTES.md`
- prototype macOS UI: `~/projects/whisper/Sources/VerboScribeCore/MainViewController.swift`
- current desktop UI: `apps/desktop/src/main.ts`
- current scope and epics: `docs/EPICS.md`
- current backlog: `docs/BACKLOG.md`
