import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor, act } from "@testing-library/react";
import { AnnotationCanvas } from "./AnnotationCanvas";

// Mock Image class for controlled loading
class MockImage {
  width = 800;
  height = 600;
  src = "";
  onload: (() => void) | null = null;
  onerror: (() => void) | null = null;

  constructor() {
    // Simulate async image loading
    setTimeout(() => {
      if (this.onload) {
        this.onload();
      }
    }, 0);
  }
}

// Mock canvas context
const mockContext = {
  drawImage: vi.fn(),
  beginPath: vi.fn(),
  moveTo: vi.fn(),
  lineTo: vi.fn(),
  stroke: vi.fn(),
  strokeStyle: "",
  lineWidth: 0,
  lineCap: "",
  lineJoin: "",
};

// Mock canvas
const mockToDataURL = vi.fn().mockReturnValue("data:image/png;base64,mockdata");
const mockGetContext = vi.fn().mockReturnValue(mockContext);

describe("AnnotationCanvas", () => {
  const mockOnExport = vi.fn();
  const mockOnCancel = vi.fn();
  const testImagePath = "/tmp/test-screenshot.png";

  beforeEach(() => {
    vi.clearAllMocks();

    // Mock Image constructor
    vi.stubGlobal("Image", MockImage);

    // Mock HTMLCanvasElement methods
    HTMLCanvasElement.prototype.getContext = mockGetContext;
    HTMLCanvasElement.prototype.toDataURL = mockToDataURL;
    Object.defineProperty(HTMLCanvasElement.prototype, "width", {
      get: () => 800,
      set: vi.fn(),
      configurable: true,
    });
    Object.defineProperty(HTMLCanvasElement.prototype, "height", {
      get: () => 600,
      set: vi.fn(),
      configurable: true,
    });
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("shows loading state initially", () => {
    // Temporarily make Image not auto-load
    vi.stubGlobal(
      "Image",
      class {
        onload: (() => void) | null = null;
        onerror: (() => void) | null = null;
        src = "";
      }
    );

    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    expect(screen.getByText("Loading screenshot...")).toBeInTheDocument();
  });

  it("renders toolbar buttons after image loads", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Undo")).toBeInTheDocument();
      expect(screen.getByText("Clear")).toBeInTheDocument();
      expect(screen.getByText("Export")).toBeInTheDocument();
      expect(screen.getByText("Cancel")).toBeInTheDocument();
    });
  });

  it("calls onCancel when Cancel button is clicked", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Cancel")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Cancel"));

    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it("calls onExport with data URL when Export button is clicked", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Export")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Export"));

    expect(mockOnExport).toHaveBeenCalledTimes(1);
    expect(mockOnExport).toHaveBeenCalledWith("data:image/png;base64,mockdata");
    expect(mockToDataURL).toHaveBeenCalledWith("image/png");
  });

  it("has Undo and Clear buttons disabled initially (no strokes)", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      const undoButton = screen.getByText("Undo");
      const clearButton = screen.getByText("Clear");

      expect(undoButton).toBeDisabled();
      expect(clearButton).toBeDisabled();
    });
  });

  it("shows error state when image fails to load", async () => {
    // Mock Image to trigger error
    vi.stubGlobal(
      "Image",
      class {
        width = 0;
        height = 0;
        src = "";
        onload: (() => void) | null = null;
        onerror: (() => void) | null = null;

        constructor() {
          setTimeout(() => {
            if (this.onerror) {
              this.onerror();
            }
          }, 0);
        }
      }
    );

    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(
        screen.getByText(`Failed to load image: ${testImagePath}`)
      ).toBeInTheDocument();
    });

    // Should show Close button in error state
    expect(screen.getByText("Close")).toBeInTheDocument();
  });

  it("calls onCancel when Close button clicked in error state", async () => {
    // Mock Image to trigger error
    vi.stubGlobal(
      "Image",
      class {
        src = "";
        onload: (() => void) | null = null;
        onerror: (() => void) | null = null;

        constructor() {
          setTimeout(() => {
            if (this.onerror) {
              this.onerror();
            }
          }, 0);
        }
      }
    );

    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(screen.getByText("Close")).toBeInTheDocument();
    });

    fireEvent.click(screen.getByText("Close"));

    expect(mockOnCancel).toHaveBeenCalledTimes(1);
  });

  it("renders canvas element after image loads", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      const canvas = document.querySelector(".annotation-canvas");
      expect(canvas).toBeInTheDocument();
    });
  });

  it("draws image on canvas after loading", async () => {
    render(
      <AnnotationCanvas
        imagePath={testImagePath}
        onExport={mockOnExport}
        onCancel={mockOnCancel}
      />
    );

    await waitFor(() => {
      expect(mockGetContext).toHaveBeenCalledWith("2d");
      expect(mockContext.drawImage).toHaveBeenCalled();
    });
  });
});
