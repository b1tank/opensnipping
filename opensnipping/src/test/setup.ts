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

// Mock Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
