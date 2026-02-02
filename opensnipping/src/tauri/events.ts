/**
 * Centralized Tauri event subscription utilities.
 *
 * All listen(...) calls should go through this module.
 * Provides a React hook for subscribing to capture events.
 */

import { useEffect, useRef } from "react";
import { listen, UnlistenFn } from "@tauri-apps/api/event";
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
} from "../types";

// ─────────────────────────────────────────────────────────────
// Event handler types
// ─────────────────────────────────────────────────────────────

export interface CaptureEventHandlers {
  onStateChanged?: (state: CaptureState, previous: CaptureState) => void;
  onError?: (message: string, code: string) => void;
  onSelectionComplete?: (selection: SelectionCompleteEvent["selection"]) => void;
  onScreenshotComplete?: (path: string, width: number, height: number) => void;
  onRecordingStarted?: (outputPath: string) => void;
  onRecordingStopped?: (path: string, durationMs: number, width: number, height: number) => void;
}

// ─────────────────────────────────────────────────────────────
// React hook for capture events
// ─────────────────────────────────────────────────────────────

/**
 * Hook to subscribe to all capture-related Tauri events.
 * Automatically cleans up listeners on unmount.
 *
 * @param handlers - Object with optional callbacks for each event type
 */
export function useCaptureEvents(handlers: CaptureEventHandlers): void {
  // Use ref to avoid re-subscribing when handler references change
  const handlersRef = useRef(handlers);
  handlersRef.current = handlers;

  useEffect(() => {
    const unlisteners: UnlistenFn[] = [];

    // State changed
    listen<StateChangedEvent>(EVENT_STATE_CHANGED, (event) => {
      handlersRef.current.onStateChanged?.(
        event.payload.state,
        event.payload.previous
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    // Error
    listen<ErrorEvent>(EVENT_ERROR, (event) => {
      handlersRef.current.onError?.(
        event.payload.error.message,
        event.payload.error.code
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    // Selection complete
    listen<SelectionCompleteEvent>(EVENT_SELECTION_COMPLETE, (event) => {
      handlersRef.current.onSelectionComplete?.(event.payload.selection);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Screenshot complete
    listen<ScreenshotCompleteEvent>(EVENT_SCREENSHOT_COMPLETE, (event) => {
      handlersRef.current.onScreenshotComplete?.(
        event.payload.path,
        event.payload.width,
        event.payload.height
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    // Recording started
    listen<RecordingStartedEvent>(EVENT_RECORDING_STARTED, (event) => {
      handlersRef.current.onRecordingStarted?.(event.payload.output_path);
    }).then((unlisten) => unlisteners.push(unlisten));

    // Recording stopped
    listen<RecordingStoppedEvent>(EVENT_RECORDING_STOPPED, (event) => {
      handlersRef.current.onRecordingStopped?.(
        event.payload.path,
        event.payload.duration_ms,
        event.payload.width,
        event.payload.height
      );
    }).then((unlisten) => unlisteners.push(unlisten));

    return () => {
      unlisteners.forEach((unlisten) => unlisten());
    };
  }, []);
}

// Re-export event names for convenience
export {
  EVENT_STATE_CHANGED,
  EVENT_ERROR,
  EVENT_SELECTION_COMPLETE,
  EVENT_SCREENSHOT_COMPLETE,
  EVENT_RECORDING_STARTED,
  EVENT_RECORDING_STOPPED,
} from "../types";
