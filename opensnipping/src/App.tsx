import { useState, useEffect, useCallback } from "react";
import "./App.css";
import { CaptureState } from "./types";
import {
  ping,
  getState,
  startCapture,
  pauseRecording,
  resumeRecording,
  finalizeComplete,
  resetError,
  takeScreenshot,
  startRecordingVideo,
  stopRecordingVideo,
  pauseRecordingVideo,
  resumeRecordingVideo,
  useCaptureEvents,
} from "./tauri";
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

  // Handle state changes
  const handleStateChanged = useCallback((state: CaptureState) => {
    setCaptureState(state);
    if (state !== "error") {
      setError(null);
    }
  }, []);

  // Handle errors
  const handleError = useCallback((message: string) => {
    setError(message);
  }, []);

  // Handle selection complete (for logging/debugging)
  const handleSelectionComplete = useCallback((selection: unknown) => {
    console.log("Selection complete:", selection);
  }, []);

  // Handle screenshot complete
  const handleScreenshotComplete = useCallback((path: string) => {
    console.log("Screenshot complete:", path);
    setScreenshotPath(path);
    setIsCapturingScreenshot(false);
  }, []);

  // Handle recording started
  const handleRecordingStarted = useCallback((outputPath: string) => {
    console.log("Recording started:", outputPath);
    setRecordingPath(outputPath);
  }, []);

  // Handle recording stopped
  const handleRecordingStopped = useCallback((path: string, durationMs: number) => {
    console.log("Recording stopped:", path, durationMs);
    setRecordingPath(null);
    alert(`Recording saved to: ${path}\nDuration: ${Math.round(durationMs / 1000)}s`);
  }, []);

  // Subscribe to capture events
  useCaptureEvents({
    onStateChanged: handleStateChanged,
    onError: handleError,
    onSelectionComplete: handleSelectionComplete,
    onScreenshotComplete: handleScreenshotComplete,
    onRecordingStarted: handleRecordingStarted,
    onRecordingStopped: handleRecordingStopped,
  });

  // Fetch initial state
  useEffect(() => {
    getState().then(setCaptureState);
  }, []);

  async function handlePingRust() {
    const result = await ping();
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
      const newState = await startCapture({
        source: "screen",
        fps: 30,
        include_cursor: true,
        audio: { system: false, mic: false },
        container: "mp4",
        output_path: outputPath,
      });
      
      // If portal selection succeeded (state is now Recording), start actual video recording
      if (newState === "recording") {
        await startRecordingVideo();
      }
    } catch (e) {
      setError(String(e));
    }
  }

  async function handlePauseRecording() {
    try {
      await pauseRecording();
      await pauseRecordingVideo();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResumeRecording() {
    try {
      await resumeRecording();
      await resumeRecordingVideo();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleStopRecording() {
    try {
      // Stop actual video recording (this also transitions state)
      await stopRecordingVideo();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleFinalizeComplete() {
    try {
      await finalizeComplete();
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleResetError() {
    try {
      await resetError();
      setError(null);
    } catch (e) {
      setError(String(e));
    }
  }

  async function handleTakeScreenshot() {
    setIsCapturingScreenshot(true);
    setError(null);
    try {
      await takeScreenshot({
        source: "screen",
        fps: 30,
        include_cursor: true,
        audio: { system: false, mic: false },
        container: "mp4",
        output_path: "/tmp/screenshot.png",
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
