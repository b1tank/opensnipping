import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri API core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_state") return Promise.resolve("idle");
    if (cmd === "ping") return Promise.resolve("Pong from Rust!");
    if (cmd === "take_screenshot") {
      return Promise.resolve({
        path: "/tmp/test-screenshot.png",
        width: 800,
        height: 600,
      });
    }
    return Promise.resolve(null);
  }),
  convertFileSrc: vi.fn().mockImplementation((path: string) => {
    // Return a data URL for testing (simulates asset protocol)
    return `asset://localhost/${path}`;
  }),
}));

// Store event listeners for testing
type EventCallback = (event: { payload: unknown }) => void;
const eventListeners: Map<string, EventCallback[]> = new Map();

// Helper to emit events in tests
export function emitMockEvent(eventName: string, payload: unknown) {
  const listeners = eventListeners.get(eventName) || [];
  listeners.forEach((callback) => callback({ payload }));
}

// Helper to clear all listeners between tests
export function clearMockEventListeners() {
  eventListeners.clear();
}

// Mock Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockImplementation((eventName: string, callback: EventCallback) => {
    // Store the callback
    if (!eventListeners.has(eventName)) {
      eventListeners.set(eventName, []);
    }
    eventListeners.get(eventName)!.push(callback);

    // Return unlisten function
    return Promise.resolve(() => {
      const listeners = eventListeners.get(eventName);
      if (listeners) {
        const index = listeners.indexOf(callback);
        if (index > -1) {
          listeners.splice(index, 1);
        }
      }
    });
  }),
}));
