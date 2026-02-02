/**
 * Centralized Tauri command wrappers.
 *
 * All invoke("...") calls should go through this module.
 * UI components should import these typed functions instead of
 * calling invoke directly.
 */

import { invoke } from "@tauri-apps/api/core";
import { CaptureConfig, CaptureState } from "../types";

// ─────────────────────────────────────────────────────────────
// Command names (must match Rust #[tauri::command] names)
// ─────────────────────────────────────────────────────────────

export const CMD_PING = "ping";
export const CMD_GET_STATE = "get_state";
export const CMD_START_CAPTURE = "start_capture";
export const CMD_PAUSE_RECORDING = "pause_recording";
export const CMD_RESUME_RECORDING = "resume_recording";
export const CMD_FINALIZE_COMPLETE = "finalize_complete";
export const CMD_RESET_ERROR = "reset_error";
export const CMD_TAKE_SCREENSHOT = "take_screenshot";
export const CMD_START_RECORDING_VIDEO = "start_recording_video";
export const CMD_STOP_RECORDING_VIDEO = "stop_recording_video";
export const CMD_PAUSE_RECORDING_VIDEO = "pause_recording_video";
export const CMD_RESUME_RECORDING_VIDEO = "resume_recording_video";

// ─────────────────────────────────────────────────────────────
// Typed command wrappers
// ─────────────────────────────────────────────────────────────

/** Ping the Rust backend (health check). */
export function ping(): Promise<string> {
  return invoke<string>(CMD_PING);
}

/** Get the current capture state. */
export function getState(): Promise<CaptureState> {
  return invoke<CaptureState>(CMD_GET_STATE);
}

/** Start capture with the given config. Returns the new state. */
export function startCapture(config: CaptureConfig): Promise<CaptureState> {
  return invoke<CaptureState>(CMD_START_CAPTURE, { config });
}

/** Pause the current recording (state machine transition). */
export function pauseRecording(): Promise<void> {
  return invoke(CMD_PAUSE_RECORDING);
}

/** Resume a paused recording (state machine transition). */
export function resumeRecording(): Promise<void> {
  return invoke(CMD_RESUME_RECORDING);
}

/** Mark finalize as complete (state → idle). */
export function finalizeComplete(): Promise<void> {
  return invoke(CMD_FINALIZE_COMPLETE);
}

/** Reset from error state back to idle. */
export function resetError(): Promise<void> {
  return invoke(CMD_RESET_ERROR);
}

/** Take a screenshot with the given config. */
export function takeScreenshot(config: CaptureConfig): Promise<void> {
  return invoke(CMD_TAKE_SCREENSHOT, { config });
}

/** Start actual video recording (after portal selection). */
export function startRecordingVideo(): Promise<void> {
  return invoke(CMD_START_RECORDING_VIDEO);
}

/** Stop video recording and finalize output. */
export function stopRecordingVideo(): Promise<void> {
  return invoke(CMD_STOP_RECORDING_VIDEO);
}

/** Pause the video recording pipeline. */
export function pauseRecordingVideo(): Promise<void> {
  return invoke(CMD_PAUSE_RECORDING_VIDEO);
}

/** Resume the video recording pipeline. */
export function resumeRecordingVideo(): Promise<void> {
  return invoke(CMD_RESUME_RECORDING_VIDEO);
}
