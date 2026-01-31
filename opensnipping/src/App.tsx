import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import "./App.css";
import {
  CaptureState,
  StateChangedEvent,
  ErrorEvent,
  EVENT_STATE_CHANGED,
  EVENT_ERROR,
} from "./types";

type AppMode = "screenshot" | "record";

function App() {
  const [mode, setMode] = useState<AppMode>("screenshot");
  const [captureState, setCaptureState] = useState<CaptureState>("idle");
  const [error, setError] = useState<string | null>(null);
  const [pingResult, setPingResult] = useState<string>("");

  // Listen to Rust events
  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // Listen for state changes
    listen<StateChangedEvent>(EVENT_STATE_CHANGED, (event) => {
      setCaptureState(event.payload.state);
      if (event.payload.state !== "error") {
        setError(null);
      }
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for errors
    listen<ErrorEvent>(EVENT_ERROR, (event) => {
      setError(event.payload.error.message);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Fetch initial state
    invoke<CaptureState>("get_state").then(setCaptureState);

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);

  async function handlePingRust() {
    const result = await invoke<string>("ping");
    setPingResult(result);
  }

  function handleToggleMode() {
    setMode((prev) => (prev === "screenshot" ? "record" : "screenshot"));
  }

  async function handleStartCapture() {
    try {
      await invoke("start_capture", {
        config: {
          source: "screen",
          fps: 30,
          include_cursor: true,
          audio: { system: false, mic: false },
          container: "mp4",
          output_path: "/tmp/recording.mp4",
        },
      });
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleCancelCapture() {
    try {
      await invoke("cancel_capture");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleBeginRecording() {
    try {
      await invoke("begin_recording");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handlePauseRecording() {
    try {
      await invoke("pause_recording");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResumeRecording() {
    try {
      await invoke("resume_recording");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleStopRecording() {
    try {
      await invoke("stop_recording");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleFinalizeComplete() {
    try {
      await invoke("finalize_complete");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResetError() {
    try {
      await invoke("reset_error");
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  const getStateLabel = (state: CaptureState) => {
    const labels: Record<CaptureState, string> = {
      idle: "Idle",
      selecting: "Selecting...",
      recording: "Recording",
      paused: "Paused",
      finalizing: "Finalizing...",
      error: "Error",
    };
    return labels[state];
  };

  return (
    <main className="container">
      <h1>OpenSnipping</h1>
      <p className="mode-indicator">
        Mode: <strong>{mode === "screenshot" ? "Screenshot" : "Record"}</strong>
      </p>
      <p className="state-indicator">
        State: <strong className={`state-${captureState}`}>{getStateLabel(captureState)}</strong>
      </p>

      {error && <p className="error-message">{error}</p>}

      <div className="button-row">
        <button onClick={handlePingRust} className="btn">
          Ping Rust
        </button>
        <button onClick={handleToggleMode} className="btn">
          Toggle Mode
        </button>
      </div>

      <div className="button-row">
        {captureState === "idle" && (
          <button onClick={handleStartCapture} className="btn btn-primary">
            Start Capture
          </button>
        )}
        {captureState === "selecting" && (
          <>
            <button onClick={handleBeginRecording} className="btn btn-primary">
              Begin Recording
            </button>
            <button onClick={handleCancelCapture} className="btn">
              Cancel
            </button>
          </>
        )}
        {captureState === "recording" && (
          <>
            <button onClick={handlePauseRecording} className="btn">
              Pause
            </button>
            <button onClick={handleStopRecording} className="btn btn-danger">
              Stop
            </button>
          </>
        )}
        {captureState === "paused" && (
          <>
            <button onClick={handleResumeRecording} className="btn btn-primary">
              Resume
            </button>
            <button onClick={handleStopRecording} className="btn btn-danger">
              Stop
            </button>
          </>
        )}
        {captureState === "finalizing" && (
          <button onClick={handleFinalizeComplete} className="btn btn-primary">
            Done
          </button>
        )}
        {captureState === "error" && (
          <button onClick={handleResetError} className="btn">
            Reset
          </button>
        )}
      </div>

      {pingResult && <p className="ping-result">{pingResult}</p>}
    </main>
  );
}

export default App;
