import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type DictationMode = "pressAndHold" | "toggle";
type EngineState = "Idle" | "Starting" | "Recording" | "Transcribing" | "Succeeded" | "Failed";
type RuntimePhase = "idle" | "starting" | "recording" | "transcribing" | "succeeded" | "failed";
type DictationCommand =
  | "start_dictation"
  | "stop_dictation"
  | "cancel_dictation"
  | "paste_last_transcript";
type NoticeTone = "success" | "warning" | "error";

type StatusModel = {
  appStatus: string;
  engineState: EngineState;
  provider: string;
  dictationMode: DictationMode;
  hotkey: string;
  toggleHotkey: string;
  pasteLastHotkey: string;
  usageHint: string;
  recovery: string;
  lastTranscript: string;
};

type SettingsPayload = {
  provider: string;
  whisperCppBinaryPath?: string | null;
  whisperCppModelPath?: string | null;
  language: string;
  whisperCppPromptContext: string;
  whisperCppPinnedTerms: string;
  dictationMode: DictationMode;
  minRecordingMs: number;
  hotkey: string;
  toggleHotkey: string;
  pasteLastHotkey: string;
};

type SettingsModel = {
  provider: "whisperCpp";
  whisperCppBinaryPath: string;
  whisperCppModelPath: string;
  language: string;
  whisperCppPromptContext: string;
  whisperCppPinnedTerms: string;
  dictationMode: DictationMode;
  minRecordingMs: number;
  hotkey: string;
  toggleHotkey: string;
  pasteLastHotkey: string;
};

type RecoveryModel = {
  title: string;
  detail: string;
  nextStep: string;
};

type RuntimeModel = {
  phase: RuntimePhase;
  message: string;
  recovery?: RecoveryModel | null;
  transcript?: string | null;
};

type NoticeModel = {
  tone: NoticeTone;
  message: string;
};

type UiState = {
  status: StatusModel;
  runtime: RuntimeModel;
  savedSettings: SettingsModel;
  draftSettings: SettingsModel;
  previewMode: boolean;
  dirty: boolean;
  saving: boolean;
  busyCommand: DictationCommand | null;
  notice: NoticeModel | null;
};

type ShellElements = {
  appStatusCopy: HTMLParagraphElement;
  engineStateBadge: HTMLSpanElement;
  notice: HTMLDivElement;
  providerSelect: HTMLSelectElement;
  languageInput: HTMLInputElement;
  binaryPathInput: HTMLInputElement;
  modelPathInput: HTMLInputElement;
  hotkeyInput: HTMLInputElement;
  toggleHotkeyInput: HTMLInputElement;
  pasteLastHotkeyInput: HTMLInputElement;
  modeSelect: HTMLSelectElement;
  promptContextInput: HTMLTextAreaElement;
  pinnedTermsInput: HTMLTextAreaElement;
  saveButton: HTMLButtonElement;
  reloadButton: HTMLButtonElement;
  startButton: HTMLButtonElement;
  stopButton: HTMLButtonElement;
  cancelButton: HTMLButtonElement;
  pasteLastButton: HTMLButtonElement;
  prototypeGap: HTMLParagraphElement;
  providerMetric: HTMLParagraphElement;
  modeMetric: HTMLParagraphElement;
  hotkeyMetric: HTMLParagraphElement;
  toggleHotkeyMetric: HTMLParagraphElement;
  pasteLastHotkeyMetric: HTMLParagraphElement;
  healthMetric: HTMLParagraphElement;
  usageHint: HTMLParagraphElement;
  runtimePhase: HTMLParagraphElement;
  runtimeMessage: HTMLParagraphElement;
  recoverySummary: HTMLParagraphElement;
  recoveryDetails: HTMLDivElement;
  recoveryTitle: HTMLParagraphElement;
  recoveryDetail: HTMLParagraphElement;
  recoveryNext: HTMLParagraphElement;
  transcriptBody: HTMLPreElement;
};

const fallbackStatus: StatusModel = {
  appStatus: "Desktop shell ready",
  engineState: "Idle",
  provider: "whisper.cpp",
  dictationMode: "pressAndHold",
  hotkey: "Control+Option+Space",
  toggleHotkey: "Control+Option+D",
  pasteLastHotkey: "Control+Option+V",
  usageHint: "Hold Control+Option+Space while speaking, then release to transcribe.",
  recovery: "No recovery needed",
  lastTranscript: "",
};

const fallbackSettings: SettingsModel = {
  provider: "whisperCpp",
  whisperCppBinaryPath: "",
  whisperCppModelPath: "",
  language: "en",
  whisperCppPromptContext: "",
  whisperCppPinnedTerms: "",
  dictationMode: "pressAndHold",
  minRecordingMs: 1_000,
  hotkey: "Control+Option+Space",
  toggleHotkey: "Control+Option+D",
  pasteLastHotkey: "Control+Option+V",
};

const fallbackRuntime: RuntimeModel = {
  phase: "idle",
  message: "Preview mode outside the Tauri shell.",
  recovery: null,
  transcript: null,
};

const uiState: UiState = {
  status: fallbackStatus,
  runtime: fallbackRuntime,
  savedSettings: fallbackSettings,
  draftSettings: cloneSettings(fallbackSettings),
  previewMode: false,
  dirty: false,
  saving: false,
  busyCommand: null,
  notice: null,
};

let shellElements: ShellElements | null = null;

function cloneSettings(settings: SettingsModel): SettingsModel {
  return { ...settings };
}

function normalizeSettings(payload?: SettingsPayload | null): SettingsModel {
  return {
    provider: "whisperCpp",
    whisperCppBinaryPath: payload?.whisperCppBinaryPath ?? "",
    whisperCppModelPath: payload?.whisperCppModelPath ?? "",
    language: payload?.language ?? fallbackSettings.language,
    whisperCppPromptContext:
      payload?.whisperCppPromptContext ?? fallbackSettings.whisperCppPromptContext,
    whisperCppPinnedTerms:
      payload?.whisperCppPinnedTerms ?? fallbackSettings.whisperCppPinnedTerms,
    dictationMode: payload?.dictationMode ?? fallbackSettings.dictationMode,
    minRecordingMs: payload?.minRecordingMs ?? fallbackSettings.minRecordingMs,
    hotkey: payload?.hotkey ?? fallbackSettings.hotkey,
    toggleHotkey: payload?.toggleHotkey ?? fallbackSettings.toggleHotkey,
    pasteLastHotkey: payload?.pasteLastHotkey ?? fallbackSettings.pasteLastHotkey,
  };
}

function serializeSettings(settings: SettingsModel): SettingsPayload {
  return {
    provider: settings.provider,
    whisperCppBinaryPath: normalizeOptionalInput(settings.whisperCppBinaryPath),
    whisperCppModelPath: normalizeOptionalInput(settings.whisperCppModelPath),
    language: settings.language.trim(),
    whisperCppPromptContext: settings.whisperCppPromptContext.trim(),
    whisperCppPinnedTerms: settings.whisperCppPinnedTerms.trim(),
    dictationMode: settings.dictationMode,
    minRecordingMs: settings.minRecordingMs,
    hotkey: settings.hotkey.trim(),
    toggleHotkey: settings.toggleHotkey.trim(),
    pasteLastHotkey: settings.pasteLastHotkey.trim(),
  };
}

function normalizeOptionalInput(value: string): string | null {
  const trimmed = value.trim();
  return trimmed.length > 0 ? trimmed : null;
}

function mountShell() {
  const app = document.querySelector<HTMLDivElement>("#app");
  if (!app) {
    throw new Error("app container not found");
  }

  app.innerHTML = `
    <div class="shell">
      <header class="hero">
        <div class="hero-copy">
          <p class="eyebrow">Desktop dictation stack</p>
          <h1>VerboScribe 2</h1>
          <p id="app-status-copy" class="hero-summary"></p>
        </div>
        <div class="hero-badge-wrap">
          <span id="engine-state-badge" class="state-badge"></span>
          <p class="hero-meta">Local-first whisper.cpp pipeline with live recovery.</p>
        </div>
      </header>

      <div id="notice" class="notice" hidden></div>

      <main class="workspace">
        <section class="panel settings-panel" aria-label="Desktop settings">
          <div class="section-heading">
            <div>
              <p class="eyebrow">Settings</p>
              <h2>Desktop configuration</h2>
            </div>
            <p class="section-copy">
              Configure the current dictation path without editing <code>settings.json</code>.
            </p>
          </div>

          <form id="settings-form" class="settings-form">
            <div class="field-grid">
              <label class="field">
                <span>Provider</span>
                <select name="provider">
                  <option value="whisperCpp">whisper.cpp</option>
                </select>
                <small>Groq and provider switching are still deferred.</small>
              </label>

              <label class="field">
                <span>Language</span>
                <input name="language" type="text" autocomplete="off" spellcheck="false" />
                <small>Use the backend language code, such as <code>en</code>.</small>
              </label>

              <label class="field field-wide">
                <span>whisper.cpp binary path</span>
                <input
                  name="whisperCppBinaryPath"
                  type="text"
                  autocomplete="off"
                  spellcheck="false"
                />
                <small>Leave blank if you only want to save non-provider settings right now.</small>
              </label>

              <label class="field field-wide">
                <span>whisper.cpp model path</span>
                <input
                  name="whisperCppModelPath"
                  type="text"
                  autocomplete="off"
                  spellcheck="false"
                />
                <small>The live dictation path still expects a valid local model file.</small>
              </label>

              <label class="field">
                <span>Dictation mode</span>
                <select name="dictationMode">
                  <option value="pressAndHold">Press and hold</option>
                  <option value="toggle">Toggle</option>
                </select>
                <small>Save settings to apply mode changes to the live hotkey path.</small>
              </label>

              <label class="field">
                <span>Dictation hotkey</span>
                <input name="hotkey" type="text" autocomplete="off" spellcheck="false" />
                <small>Follows the dictation mode above (press and hold by default).</small>
              </label>

              <label class="field">
                <span>Toggle hotkey</span>
                <input name="toggleHotkey" type="text" autocomplete="off" spellcheck="false" />
                <small>Always toggles: tap once to start dictation, tap again to stop.</small>
              </label>

              <label class="field">
                <span>Paste-last hotkey</span>
                <input name="pasteLastHotkey" type="text" autocomplete="off" spellcheck="false" />
                <small>Retries inserting the preserved last transcript without recording again.</small>
              </label>
            </div>

            <label class="field field-wide">
              <span>Prompt context override</span>
              <textarea
                name="whisperCppPromptContext"
                rows="5"
                spellcheck="false"
              ></textarea>
              <small>
                This text is appended after the built-in VerboScribe product and app-name bias.
              </small>
            </label>

            <label class="field field-wide">
              <span>Pinned terms</span>
              <textarea
                name="whisperCppPinnedTerms"
                rows="4"
                spellcheck="false"
              ></textarea>
              <small>Use comma-separated names or jargon that should stay intact in transcripts.</small>
            </label>

            <div class="form-actions">
              <button id="save-settings" class="button button-primary" type="submit">
                Save settings
              </button>
              <button id="reload-settings" class="button button-secondary" type="button">
                Reload saved
              </button>
            </div>
          </form>

          <div class="prototype-gap">
            <span>Foundation slice notes</span>
            <p id="prototype-gap-copy"></p>
          </div>
        </section>

        <aside class="sidebar">
          <section class="panel live-panel" aria-label="Live dictation status">
            <div class="section-heading">
              <div>
                <p class="eyebrow">Live status</p>
                <h2>Current session</h2>
              </div>
              <p class="section-copy">Status, controls, and the current hotkey are visible here.</p>
            </div>

            <div class="metric-grid">
              <article class="metric">
                <span>Provider</span>
                <p id="metric-provider"></p>
              </article>
              <article class="metric">
                <span>Mode</span>
                <p id="metric-mode"></p>
              </article>
              <article class="metric metric-wide">
                <span>Hotkey</span>
                <p id="metric-hotkey"></p>
              </article>
              <article class="metric metric-wide">
                <span>Toggle hotkey</span>
                <p id="metric-toggle-hotkey"></p>
              </article>
              <article class="metric metric-wide">
                <span>Paste-last hotkey</span>
                <p id="metric-paste-last-hotkey"></p>
              </article>
              <article class="metric metric-wide">
                <span>Health</span>
                <p id="metric-health"></p>
              </article>
            </div>

            <div class="control-strip">
              <button id="start-dictation" class="button button-primary" type="button">Start</button>
              <button id="stop-dictation" class="button button-secondary" type="button">Stop</button>
              <button id="cancel-dictation" class="button button-ghost" type="button">Cancel</button>
              <button id="paste-last-transcript" class="button button-secondary" type="button">
                Paste last transcript
              </button>
            </div>

            <div class="hint-block">
              <span>How to dictate</span>
              <p id="usage-hint"></p>
            </div>
          </section>

          <section class="panel runtime-panel" aria-label="Runtime recovery">
            <div class="section-heading">
              <div>
                <p class="eyebrow">Recovery</p>
                <h2>Runtime and fallback</h2>
              </div>
              <p class="section-copy">Keep the current recovery guidance visible in the same surface.</p>
            </div>

            <div class="runtime-card">
              <p id="runtime-phase" class="runtime-phase"></p>
              <p id="runtime-message" class="runtime-message"></p>
            </div>

            <div class="summary-card">
              <span>Current summary</span>
              <p id="recovery-summary"></p>
            </div>

            <div id="recovery-details" class="recovery-details" hidden>
              <p id="recovery-title" class="recovery-title"></p>
              <p id="recovery-detail"></p>
              <p id="recovery-next" class="recovery-next"></p>
            </div>
          </section>

          <section class="panel transcript-panel" aria-label="Last transcript">
            <div class="section-heading">
              <div>
                <p class="eyebrow">Transcript</p>
                <h2>Last captured text</h2>
              </div>
              <p class="section-copy">The last transcript remains visible even after a paste failure.</p>
            </div>

            <pre id="transcript-body" class="transcript-body"></pre>
          </section>
        </aside>
      </main>
    </div>
  `;

  shellElements = {
    appStatusCopy: queryElement("#app-status-copy"),
    engineStateBadge: queryElement("#engine-state-badge"),
    notice: queryElement("#notice"),
    providerSelect: queryElement('select[name="provider"]'),
    languageInput: queryElement('input[name="language"]'),
    binaryPathInput: queryElement('input[name="whisperCppBinaryPath"]'),
    modelPathInput: queryElement('input[name="whisperCppModelPath"]'),
    hotkeyInput: queryElement('input[name="hotkey"]'),
    toggleHotkeyInput: queryElement('input[name="toggleHotkey"]'),
    pasteLastHotkeyInput: queryElement('input[name="pasteLastHotkey"]'),
    modeSelect: queryElement('select[name="dictationMode"]'),
    promptContextInput: queryElement('textarea[name="whisperCppPromptContext"]'),
    pinnedTermsInput: queryElement('textarea[name="whisperCppPinnedTerms"]'),
    saveButton: queryElement("#save-settings"),
    reloadButton: queryElement("#reload-settings"),
    startButton: queryElement("#start-dictation"),
    stopButton: queryElement("#stop-dictation"),
    cancelButton: queryElement("#cancel-dictation"),
    pasteLastButton: queryElement("#paste-last-transcript"),
    prototypeGap: queryElement("#prototype-gap-copy"),
    providerMetric: queryElement("#metric-provider"),
    modeMetric: queryElement("#metric-mode"),
    hotkeyMetric: queryElement("#metric-hotkey"),
    toggleHotkeyMetric: queryElement("#metric-toggle-hotkey"),
    pasteLastHotkeyMetric: queryElement("#metric-paste-last-hotkey"),
    healthMetric: queryElement("#metric-health"),
    usageHint: queryElement("#usage-hint"),
    runtimePhase: queryElement("#runtime-phase"),
    runtimeMessage: queryElement("#runtime-message"),
    recoverySummary: queryElement("#recovery-summary"),
    recoveryDetails: queryElement("#recovery-details"),
    recoveryTitle: queryElement("#recovery-title"),
    recoveryDetail: queryElement("#recovery-detail"),
    recoveryNext: queryElement("#recovery-next"),
    transcriptBody: queryElement("#transcript-body"),
  };

  wireEvents();
}

function wireEvents() {
  const elements = requireShellElements();

  queryElement<HTMLFormElement>("#settings-form").addEventListener("submit", (event) => {
    event.preventDefault();
    void saveSettings();
  });

  const inputHandler = (event: Event) => {
    const target = event.target;
    if (
      !(target instanceof HTMLInputElement) &&
      !(target instanceof HTMLTextAreaElement) &&
      !(target instanceof HTMLSelectElement)
    ) {
      return;
    }

    const { name, value } = target;
    if (!isSettingsField(name)) {
      return;
    }

    uiState.draftSettings = {
      ...uiState.draftSettings,
      [name]: value,
    };
    uiState.dirty = !settingsEqual(uiState.savedSettings, uiState.draftSettings);
    syncShell();
  };

  elements.providerSelect.addEventListener("change", inputHandler);
  elements.languageInput.addEventListener("input", inputHandler);
  elements.binaryPathInput.addEventListener("input", inputHandler);
  elements.modelPathInput.addEventListener("input", inputHandler);
  elements.hotkeyInput.addEventListener("input", inputHandler);
  elements.toggleHotkeyInput.addEventListener("input", inputHandler);
  elements.pasteLastHotkeyInput.addEventListener("input", inputHandler);
  elements.modeSelect.addEventListener("change", inputHandler);
  elements.promptContextInput.addEventListener("input", inputHandler);
  elements.pinnedTermsInput.addEventListener("input", inputHandler);

  elements.reloadButton.addEventListener("click", () => {
    void reloadSettings();
  });
  elements.startButton.addEventListener("click", () => {
    void runDictationCommand("start_dictation");
  });
  elements.stopButton.addEventListener("click", () => {
    void runDictationCommand("stop_dictation");
  });
  elements.cancelButton.addEventListener("click", () => {
    void runDictationCommand("cancel_dictation");
  });
  elements.pasteLastButton.addEventListener("click", () => {
    void runDictationCommand("paste_last_transcript");
  });
}

function syncShell(forceFormSync = false) {
  const elements = requireShellElements();
  const status = uiState.status;
  const runtime = uiState.runtime;
  const draftNotice =
    uiState.dirty && !uiState.saving
      ? {
          tone: "warning" as const,
          message:
            "Unsaved changes: click Save settings to apply dictation mode, hotkey, and provider updates.",
        }
      : null;
  const activeNotice =
    uiState.notice && uiState.notice.tone === "error" ? uiState.notice : draftNotice ?? uiState.notice;

  elements.appStatusCopy.textContent = status.appStatus;
  elements.engineStateBadge.textContent = status.engineState;
  elements.engineStateBadge.dataset.tone = stateTone(status, runtime);

  syncNotice(elements.notice, activeNotice);

  syncControlValue(elements.providerSelect, uiState.draftSettings.provider, forceFormSync);
  syncControlValue(elements.languageInput, uiState.draftSettings.language, forceFormSync);
  syncControlValue(
    elements.binaryPathInput,
    uiState.draftSettings.whisperCppBinaryPath,
    forceFormSync,
  );
  syncControlValue(
    elements.modelPathInput,
    uiState.draftSettings.whisperCppModelPath,
    forceFormSync,
  );
  syncControlValue(elements.hotkeyInput, uiState.draftSettings.hotkey, forceFormSync);
  syncControlValue(
    elements.toggleHotkeyInput,
    uiState.draftSettings.toggleHotkey,
    forceFormSync,
  );
  syncControlValue(
    elements.pasteLastHotkeyInput,
    uiState.draftSettings.pasteLastHotkey,
    forceFormSync,
  );
  syncControlValue(elements.modeSelect, uiState.draftSettings.dictationMode, forceFormSync);
  syncControlValue(
    elements.promptContextInput,
    uiState.draftSettings.whisperCppPromptContext,
    forceFormSync,
  );
  syncControlValue(
    elements.pinnedTermsInput,
    uiState.draftSettings.whisperCppPinnedTerms,
    forceFormSync,
  );

  elements.prototypeGap.textContent =
    `Minimum recording duration remains backend-only at ${uiState.savedSettings.minRecordingMs} ms. ` +
    "Cancel hotkeys, retry-last-failed-transcript, preview-before-insert, cleanup and style controls, snippets, dictionary management, model install and refresh, history, insights, and launch behavior are still outside this first desktop settings pass.";

  elements.providerMetric.textContent = status.provider;
  elements.modeMetric.textContent = formatMode(status.dictationMode);
  elements.hotkeyMetric.textContent = status.hotkey;
  elements.toggleHotkeyMetric.textContent = status.toggleHotkey;
  elements.pasteLastHotkeyMetric.textContent = status.pasteLastHotkey;
  elements.healthMetric.textContent =
    status.recovery === "No recovery needed" ? "No recovery needed" : status.recovery;
  elements.usageHint.textContent = status.usageHint;

  elements.runtimePhase.textContent = `${formatPhase(runtime.phase)} status`;
  elements.runtimeMessage.textContent = runtime.message;
  elements.recoverySummary.textContent = status.recovery;

  if (runtime.recovery) {
    elements.recoveryDetails.hidden = false;
    elements.recoveryTitle.textContent = runtime.recovery.title;
    elements.recoveryDetail.textContent = runtime.recovery.detail;
    elements.recoveryNext.textContent = `Next: ${runtime.recovery.nextStep}`;
  } else {
    elements.recoveryDetails.hidden = true;
    elements.recoveryTitle.textContent = "";
    elements.recoveryDetail.textContent = "";
    elements.recoveryNext.textContent = "";
  }

  const transcript = runtime.transcript ?? status.lastTranscript;
  elements.transcriptBody.textContent = transcript || "No transcript captured yet.";

  elements.saveButton.disabled = uiState.previewMode || uiState.saving || !uiState.dirty;
  elements.reloadButton.disabled = uiState.previewMode || uiState.saving;
  elements.saveButton.textContent = uiState.saving
    ? "Saving..."
    : uiState.dirty
      ? "Save settings to apply"
      : "Save settings";
  elements.reloadButton.textContent = uiState.dirty ? "Discard draft" : "Reload saved";

  const commandBusy = uiState.busyCommand !== null || uiState.saving || uiState.previewMode;
  elements.startButton.disabled = commandBusy || status.engineState !== "Idle";
  elements.stopButton.disabled =
    commandBusy || (status.engineState !== "Starting" && status.engineState !== "Recording");
  elements.cancelButton.disabled = commandBusy || status.engineState === "Idle";
  elements.pasteLastButton.disabled = commandBusy || !(runtime.transcript ?? status.lastTranscript);

  elements.startButton.textContent =
    uiState.busyCommand === "start_dictation" ? "Starting..." : "Start";
  elements.stopButton.textContent =
    uiState.busyCommand === "stop_dictation" ? "Stopping..." : "Stop";
  elements.cancelButton.textContent =
    uiState.busyCommand === "cancel_dictation" ? "Cancelling..." : "Cancel";
  elements.pasteLastButton.textContent =
    uiState.busyCommand === "paste_last_transcript"
      ? "Pasting last transcript..."
      : "Paste last transcript";
}

function syncNotice(element: HTMLDivElement, notice: NoticeModel | null) {
  element.hidden = notice === null;
  element.className = notice ? `notice notice-${notice.tone}` : "notice";
  element.textContent = notice?.message ?? "";
}

function syncControlValue(
  element: HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement,
  value: string,
  force = false,
) {
  if (force || document.activeElement !== element) {
    if (element.value !== value) {
      element.value = value;
    }
  }
}

function stateTone(status: StatusModel, runtime: RuntimeModel): "ready" | "busy" | "warning" {
  if (status.recovery !== "No recovery needed" || runtime.phase === "failed") {
    return "warning";
  }
  if (status.engineState === "Recording" || status.engineState === "Starting") {
    return "busy";
  }
  return "ready";
}

function formatMode(mode: DictationMode) {
  return mode === "toggle" ? "Toggle" : "Press and hold";
}

function formatPhase(phase: RuntimePhase) {
  switch (phase) {
    case "idle":
      return "Idle";
    case "starting":
      return "Preparing";
    case "recording":
      return "Recording";
    case "transcribing":
      return "Transcribing";
    case "succeeded":
      return "Succeeded";
    case "failed":
      return "Attention needed";
  }
}

function settingsEqual(left: SettingsModel, right: SettingsModel) {
  return JSON.stringify(left) === JSON.stringify(right);
}

function isSettingsField(value: string): value is keyof SettingsModel {
  return (
    value === "provider" ||
    value === "whisperCppBinaryPath" ||
    value === "whisperCppModelPath" ||
    value === "language" ||
    value === "whisperCppPromptContext" ||
    value === "whisperCppPinnedTerms" ||
    value === "dictationMode" ||
    value === "hotkey" ||
    value === "toggleHotkey" ||
    value === "pasteLastHotkey"
  );
}

async function bootstrap() {
  const [statusResult, runtimeResult, settingsResult] = await Promise.allSettled([
    invoke<StatusModel>("app_status"),
    invoke<RuntimeModel>("runtime_status"),
    invoke<SettingsPayload>("settings"),
  ]);

  const allFailed =
    statusResult.status === "rejected" &&
    runtimeResult.status === "rejected" &&
    settingsResult.status === "rejected";

  if (statusResult.status === "fulfilled") {
    uiState.status = statusResult.value;
  }
  if (runtimeResult.status === "fulfilled") {
    uiState.runtime = runtimeResult.value;
  }
  if (settingsResult.status === "fulfilled") {
    const settings = normalizeSettings(settingsResult.value);
    uiState.savedSettings = settings;
    uiState.draftSettings = cloneSettings(settings);
    uiState.dirty = false;
  }

  if (allFailed) {
    uiState.previewMode = true;
    uiState.notice = {
      tone: "warning",
      message:
        "Preview mode: Tauri commands are unavailable in a normal browser, so the UI is showing fallback desktop data.",
    };
  } else {
    uiState.previewMode = false;
  }

  syncShell(true);

  if (!uiState.previewMode) {
    window.setInterval(() => {
      void refreshLiveState();
    }, 1000);
  }
}

async function refreshLiveState() {
  if (uiState.previewMode) {
    return;
  }

  const [statusResult, runtimeResult] = await Promise.allSettled([
    invoke<StatusModel>("app_status"),
    invoke<RuntimeModel>("runtime_status"),
  ]);

  if (statusResult.status === "fulfilled") {
    uiState.status = statusResult.value;
  }
  if (runtimeResult.status === "fulfilled") {
    uiState.runtime = runtimeResult.value;
  }

  syncShell();
}

async function reloadSettings() {
  if (uiState.previewMode) {
    return;
  }

  try {
    const payload = await invoke<SettingsPayload>("settings");
    const settings = normalizeSettings(payload);
    uiState.savedSettings = settings;
    uiState.draftSettings = cloneSettings(settings);
    uiState.dirty = false;
    uiState.notice = {
      tone: "success",
      message: "Saved settings reloaded from disk.",
    };
  } catch (error) {
    uiState.notice = {
      tone: "error",
      message: `Could not reload settings: ${formatError(error)}`,
    };
  }

  syncShell(true);
}

async function saveSettings() {
  if (uiState.previewMode) {
    return;
  }

  const validationError = validateSettings(uiState.draftSettings);
  if (validationError) {
    uiState.notice = { tone: "error", message: validationError };
    syncShell();
    return;
  }

  uiState.saving = true;
  syncShell();

  try {
    const savedPayload = await invoke<SettingsPayload>("save_settings", {
      settings: serializeSettings(uiState.draftSettings),
    });
    const savedSettings = normalizeSettings(savedPayload);
    uiState.savedSettings = savedSettings;
    uiState.draftSettings = cloneSettings(savedSettings);
    uiState.dirty = false;

    try {
      await invoke("register_dictation_hotkey");
      uiState.notice = {
        tone: "success",
        message: "Settings saved and the dictation hotkey was reapplied.",
      };
    } catch (error) {
      uiState.notice = {
        tone: "warning",
        message: `Settings saved, but hotkey registration failed: ${formatError(error)}`,
      };
    }
  } catch (error) {
    uiState.notice = {
      tone: "error",
      message: `Could not save settings: ${formatError(error)}`,
    };
  } finally {
    uiState.saving = false;
    await refreshLiveState();
    syncShell(true);
  }
}

function validateSettings(settings: SettingsModel): string | null {
  if (!settings.language.trim()) {
    return "Language is required before settings can be saved.";
  }
  if (!settings.hotkey.trim()) {
    return "A dictation hotkey is required before settings can be saved.";
  }
  if (!settings.toggleHotkey.trim()) {
    return "A toggle hotkey is required before settings can be saved.";
  }
  if (!settings.pasteLastHotkey.trim()) {
    return "A paste-last hotkey is required before settings can be saved.";
  }
  return null;
}

async function runDictationCommand(command: DictationCommand) {
  if (uiState.previewMode) {
    return;
  }

  uiState.busyCommand = command;
  syncShell();

  try {
    await invoke(command);
    uiState.notice = null;
  } catch (error) {
    uiState.notice = {
      tone: "warning",
      message: `Dictation action reported: ${formatError(error)}`,
    };
  } finally {
    uiState.busyCommand = null;
    await refreshLiveState();
    syncShell();
  }
}

function formatError(error: unknown): string {
  if (typeof error === "string") {
    return error;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return "Unknown error";
}

function queryElement<T extends Element>(selector: string): T {
  const element = document.querySelector<T>(selector);
  if (!element) {
    throw new Error(`missing element: ${selector}`);
  }
  return element;
}

function requireShellElements(): ShellElements {
  if (!shellElements) {
    throw new Error("shell elements are not mounted");
  }
  return shellElements;
}

mountShell();
syncShell(true);
void bootstrap();
