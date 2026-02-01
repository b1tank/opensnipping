import { useState, useEffect } from "react";
import { invoke } from "@tauri-apps/api/core";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
import "./App.css";
import {
  CaptureState,
  StateChangedEvent,
  ErrorEvent,
  SelectionCompleteEvent,
  ScreenshotCompleteEvent,
  RecordingStartedEvent,
  RecordingStoppedEvent,
  EVENT_STATE_CHANGED,
  EVENT_ERROR,
  EVENT_SELECTION_COMPLETE,
  EVENT_SCREENSHOT_COMPLETE,
  EVENT_RECORDING_STARTED,
  EVENT_RECORDING_STOPPED,
} from "./types";
import { AnnotationCanvas } from "./components/AnnotationCanvas";

type AppMode = "screenshot" | "record";

function App() {
  const [mode, setMode] = useState<AppMode>("screenshot");
  const [captureState, setCaptureState] = useState<CaptureState>("idle");
  const [error, setError] = useState<string | null>(null);
  const [pingResult, setPingResult] = useState<string>("");
  const [screenshotPath, setScreenshotPath] = useState<string | null>(null);
  const [isCapturingScreenshot, setIsCapturingScreenshot] = useState(false);
  const [recordingPath, setRecordingPath] = useState<string | null>(null);

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

    // Listen for selection complete (for logging/debugging)
    listen<SelectionCompleteEvent>(EVENT_SELECTION_COMPLETE, (event) => {
      console.log("Selection complete:", event.payload.selection);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for screenshot complete
    listen<ScreenshotCompleteEvent>(EVENT_SCREENSHOT_COMPLETE, (event) => {
      console.log("Screenshot complete:", event.payload);
      setScreenshotPath(event.payload.path);
      setIsCapturingScreenshot(false);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for recording started
    listen<RecordingStartedEvent>(EVENT_RECORDING_STARTED, (event) => {
      console.log("Recording started:", event.payload.output_path);
      setRecordingPath(event.payload.output_path);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Listen for recording stopped
    listen<RecordingStoppedEvent>(EVENT_RECORDING_STOPPED, (event) => {
      console.log("Recording stopped:", event.payload);
      setRecordingPath(null);
      alert(`Recording saved to: ${event.payload.path}\nDuration: ${Math.round(event.payload.duration_ms / 1000)}s`);
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
      // Generate unique output path
      const outputPath = `/tmp/opensnipping-${Date.now()}.mp4`;
      
      // First, start capture (shows portal picker, gets selection)
      const newState = await invoke<CaptureState>("start_capture", {
        config: {
          source: "screen",
          fps: 30,
          include_cursor: true,
          audio: { system: false, mic: false },
          container: "mp4",
          output_path: outputPath,
        },
      });
      
      // If portal selection succeeded (state is now Recording), start actual video recording
      if (newState === "recording") {
        await invoke("start_recording_video");
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function handlePauseRecording() {
    try {
      await invoke("pause_recording");
      await invoke("pause_recording_video");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResumeRecording() {
    try {
      await invoke("resume_recording");
      await invoke("resume_recording_video");
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleStopRecording() {
    try {
      // Stop actual video recording (this also transitions state)
      await invoke("stop_recording_video");
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

  async function handleTakeScreenshot() {
    setIsCapturingScreenshot(true);
    setError(null);
    try {
      await invoke("take_screenshot", {
        config: {
          source: "screen",
          fps: 30,
          include_cursor: true,
          audio: { system: false, mic: false },
          container: "mp4",
          output_path: "/tmp/screenshot.png",
        },
      });
    } catch (e) {
      setError(String(e));
      setIsCapturingScreenshot(false);
    }
  }

  function handleScreenshotExport(dataUrl: string) {
    // Trigger download
    const link = document.createElement("a");
    link.href = dataUrl;
    link.download = `screenshot-${Date.now()}.png`;
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    // Clear screenshot state
    setScreenshotPath(null);
  }

  function handleScreenshotCancel() {
    setScreenshotPath(null);
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
      {/* Screenshot annotation overlay */}
      {screenshotPath && (
        <AnnotationCanvas
          imagePath={screenshotPath}
          onExport={handleScreenshotExport}
          onCancel={handleScreenshotCancel}
        />
      )}

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
        {captureState === "idle" && mode === "record" && (
          <button onClick={handleStartCapture} className="btn btn-primary">
            Start Capture
          </button>
        )}
        {captureState === "idle" && mode === "screenshot" && (
          <button
            onClick={handleTakeScreenshot}
            className="btn btn-primary"
            disabled={isCapturingScreenshot}
          >
            {isCapturingScreenshot ? "Capturing..." : "Take Screenshot"}
          </button>
        )}
        {captureState === "selecting" && (
          <p className="status-text">Select a screen/window in the picker...</p>
        )}
        {captureState === "recording" && (
          <>
            {recordingPath && (
              <p className="recording-info">Recording to: {recordingPath}</p>
            )}
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
