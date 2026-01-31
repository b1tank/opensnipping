import { describe, it, expect, vi, beforeEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { invoke } from "@tauri-apps/api/core";
import App from "./App";

// Mock is set up in setup.ts
const mockInvoke = invoke as ReturnType<typeof vi.fn>;

describe("App", () => {
  beforeEach(() => {
    mockInvoke.mockClear();
    mockInvoke.mockImplementation((cmd: string) => {
      if (cmd === "get_state") return Promise.resolve("idle");
      if (cmd === "ping") return Promise.resolve("Pong from Rust!");
      return Promise.resolve("idle");
    });
  });

  it("renders the app with title", () => {
    render(<App />);
    expect(screen.getByText("OpenSnipping")).toBeInTheDocument();
  });

  it("shows Screenshot mode by default", () => {
    render(<App />);
    expect(screen.getByText("Screenshot")).toBeInTheDocument();
  });

  it("toggles mode when Toggle Mode button is clicked", async () => {
    render(<App />);

    // Initially Screenshot mode
    expect(screen.getByText("Screenshot")).toBeInTheDocument();

    // Click toggle
    fireEvent.click(screen.getByText("Toggle Mode"));

    // Now Record mode
    expect(screen.getByText("Record")).toBeInTheDocument();

    // Click toggle again
    fireEvent.click(screen.getByText("Toggle Mode"));

    // Back to Screenshot mode
    expect(screen.getByText("Screenshot")).toBeInTheDocument();
  });

  it("calls invoke when Ping Rust button is clicked", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("Ping Rust"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("ping");
    });
  });

  it("displays ping result after clicking Ping Rust", async () => {
    render(<App />);
    fireEvent.click(screen.getByText("Ping Rust"));

    await waitFor(() => {
      expect(screen.getByText("Pong from Rust!")).toBeInTheDocument();
    });
  });

  it("fetches initial state on mount", async () => {
    render(<App />);

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("get_state");
    });
  });

  it("shows Start Capture button when idle", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Capture")).toBeInTheDocument();
    });
  });

  it("calls start_capture with config when Start Capture clicked", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Start Capture")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Start Capture"));

    await waitFor(() => {
      expect(mockInvoke).toHaveBeenCalledWith("start_capture", {
        config: expect.objectContaining({
          source: "screen",
          fps: 30,
          include_cursor: true,
        }),
      });
    });
  });

  it("displays state indicator", async () => {
    render(<App />);

    await waitFor(() => {
      expect(screen.getByText("Idle")).toBeInTheDocument();
    });
  });
});
