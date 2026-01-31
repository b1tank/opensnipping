// Type definitions for capture API

export type CaptureSource = "screen" | "monitor" | "window" | "region";
export type ContainerFormat = "mp4" | "mkv";
export type CaptureState =
  | "idle"
  | "selecting"
  | "recording"
  | "paused"
  | "finalizing"
  | "error";

export interface AudioConfig {
  system: boolean;
  mic: boolean;
}

export interface CaptureConfig {
  source: CaptureSource;
  fps: number;
  include_cursor: boolean;
  audio: AudioConfig;
  container: ContainerFormat;
  output_path: string;
}

export type ErrorCode =
  | "permission_denied"
  | "portal_error"
  | "encoder_unavailable"
  | "pipeline_error"
  | "io_error"
  | "invalid_config"
  | "unknown";

export interface CaptureError {
  code: ErrorCode;
  message: string;
}

export interface StateChangedEvent {
  state: CaptureState;
  previous: CaptureState;
}

export type PermissionKind = "screen" | "microphone" | "system_audio";

export interface PermissionNeededEvent {
  kind: PermissionKind;
}

export interface ProgressEvent {
  duration_ms: number;
}

export interface ErrorEvent {
  error: CaptureError;
}

// Selection result from portal
export interface SelectionResult {
  node_id: number;
  stream_fd: number | null;
  width: number | null;
  height: number | null;
}

export interface SelectionCompleteEvent {
  selection: SelectionResult;
}

export interface ScreenshotCompleteEvent {
  path: string;
  width: number;
  height: number;
}

export interface RecordingStartedEvent {
  output_path: string;
}

export interface RecordingStoppedEvent {
  path: string;
  duration_ms: number;
  width: number;
  height: number;
}

// Event names
export const EVENT_STATE_CHANGED = "capture:state_changed";
export const EVENT_PERMISSION_NEEDED = "capture:permission_needed";
export const EVENT_PROGRESS = "capture:progress";
export const EVENT_ERROR = "capture:error";
export const EVENT_SELECTION_COMPLETE = "capture:selection_complete";
export const EVENT_SCREENSHOT_COMPLETE = "capture:screenshot_complete";
export const EVENT_RECORDING_STARTED = "capture:recording_started";
export const EVENT_RECORDING_STOPPED = "capture:recording_stopped";
