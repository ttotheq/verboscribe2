import { invoke } from "@tauri-apps/api/core";
import "./styles.css";

type StatusModel = {
  appStatus: string;
  engineState: "Idle" | "Starting" | "Recording" | "Transcribing" | "Succeeded" | "Failed";
  provider: string;
  hotkey: string;
  recovery: string;
  lastTranscript: string;
};

const fallbackStatus: StatusModel = {
  appStatus: "Desktop shell ready",
  engineState: "Idle",
  provider: "whisper.cpp",
  hotkey: "Control+Option+Space",
  recovery: "No recovery needed",
  lastTranscript: "",
};

function render(model: StatusModel) {
  const app = document.querySelector<HTMLDivElement>("#app");
  if (!app) {
    return;
  }

  app.innerHTML = `
    <div class="shell">
      <header class="topbar">
        <div>
          <h1>VerboScribe 2</h1>
          <p>${model.appStatus}</p>
        </div>
        <strong class="state">${model.engineState}</strong>
      </header>

      <section class="status-grid" aria-label="Dictation status">
        <article>
          <span>Provider</span>
          <strong>${model.provider}</strong>
        </article>
        <article>
          <span>Hotkey</span>
          <strong>${model.hotkey}</strong>
        </article>
        <article>
          <span>Recovery</span>
          <strong>${model.recovery}</strong>
        </article>
      </section>

      <section class="transcript" aria-label="Last transcript">
        <span>Last transcript</span>
        <p>${model.lastTranscript || "None yet"}</p>
      </section>
    </div>
  `;
}

async function loadStatus() {
  try {
    const status = await invoke<StatusModel>("app_status");
    render(status);
  } catch {
    render(fallbackStatus);
  }
}

void loadStatus();
window.setInterval(() => {
  void loadStatus();
}, 1000);
