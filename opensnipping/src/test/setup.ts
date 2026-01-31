import "@testing-library/jest-dom/vitest";
import { vi } from "vitest";

// Mock Tauri API core
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockImplementation((cmd: string) => {
    if (cmd === "get_state") return Promise.resolve("idle");
    if (cmd === "ping") return Promise.resolve("Pong from Rust!");
    return Promise.resolve(null);
  }),
}));

// Mock Tauri event API
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn().mockResolvedValue(() => {}),
}));
