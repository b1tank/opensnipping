import { useRef, useState, useEffect, useCallback } from "react";
import { convertFileSrc } from "@tauri-apps/api/core";

interface Point {
  x: number;
  y: number;
}

interface Stroke {
  points: Point[];
  color: string;
  width: number;
}

interface AnnotationCanvasProps {
  imagePath: string;
  onExport: (dataUrl: string) => void;
  onCancel: () => void;
}

const PEN_COLOR = "#ff0000";
const PEN_WIDTH = 3;

export function AnnotationCanvas({
  imagePath,
  onExport,
  onCancel,
}: AnnotationCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const [strokes, setStrokes] = useState<Stroke[]>([]);
  const [currentStroke, setCurrentStroke] = useState<Stroke | null>(null);
  const [isDrawing, setIsDrawing] = useState(false);
  const [image, setImage] = useState<HTMLImageElement | null>(null);
  const [imageLoaded, setImageLoaded] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load the image from local path
  useEffect(() => {
    const img = new Image();
    // Convert local file path to asset URL
    const assetUrl = convertFileSrc(imagePath);

    img.onload = () => {
      setImage(img);
      setImageLoaded(true);
      setError(null);
    };

    img.onerror = () => {
      setError(`Failed to load image: ${imagePath}`);
      setImageLoaded(false);
    };

    img.src = assetUrl;

    return () => {
      img.onload = null;
      img.onerror = null;
    };
  }, [imagePath]);

  // Redraw canvas when image loads or strokes change
  const redrawCanvas = useCallback(() => {
    const canvas = canvasRef.current;
    if (!canvas || !image) return;

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    // Set canvas size to match image
    canvas.width = image.width;
    canvas.height = image.height;

    // Draw image
    ctx.drawImage(image, 0, 0);

    // Draw all completed strokes
    for (const stroke of strokes) {
      drawStroke(ctx, stroke);
    }

    // Draw current stroke if drawing
    if (currentStroke) {
      drawStroke(ctx, currentStroke);
    }
  }, [image, strokes, currentStroke]);

  useEffect(() => {
    if (imageLoaded) {
      redrawCanvas();
    }
  }, [imageLoaded, redrawCanvas]);

  function drawStroke(ctx: CanvasRenderingContext2D, stroke: Stroke) {
    if (stroke.points.length < 2) return;

    ctx.strokeStyle = stroke.color;
    ctx.lineWidth = stroke.width;
    ctx.lineCap = "round";
    ctx.lineJoin = "round";

    ctx.beginPath();
    ctx.moveTo(stroke.points[0].x, stroke.points[0].y);

    for (let i = 1; i < stroke.points.length; i++) {
      ctx.lineTo(stroke.points[i].x, stroke.points[i].y);
    }

    ctx.stroke();
  }

  function getCanvasPoint(
    e: React.MouseEvent<HTMLCanvasElement> | React.TouchEvent<HTMLCanvasElement>
  ): Point | null {
    const canvas = canvasRef.current;
    if (!canvas) return null;

    const rect = canvas.getBoundingClientRect();
    const scaleX = canvas.width / rect.width;
    const scaleY = canvas.height / rect.height;

    let clientX: number, clientY: number;

    if ("touches" in e) {
      if (e.touches.length === 0) return null;
      clientX = e.touches[0].clientX;
      clientY = e.touches[0].clientY;
    } else {
      clientX = e.clientX;
      clientY = e.clientY;
    }

    return {
      x: (clientX - rect.left) * scaleX,
      y: (clientY - rect.top) * scaleY,
    };
  }

  function handlePointerDown(
    e: React.MouseEvent<HTMLCanvasElement> | React.TouchEvent<HTMLCanvasElement>
  ) {
    const point = getCanvasPoint(e);
    if (!point) return;

    setIsDrawing(true);
    setCurrentStroke({
      points: [point],
      color: PEN_COLOR,
      width: PEN_WIDTH,
    });
  }

  function handlePointerMove(
    e: React.MouseEvent<HTMLCanvasElement> | React.TouchEvent<HTMLCanvasElement>
  ) {
    if (!isDrawing || !currentStroke) return;

    const point = getCanvasPoint(e);
    if (!point) return;

    setCurrentStroke((prev) => {
      if (!prev) return null;
      return {
        ...prev,
        points: [...prev.points, point],
      };
    });
  }

  function handlePointerUp() {
    if (!isDrawing || !currentStroke) return;

    // Only add stroke if it has multiple points
    if (currentStroke.points.length >= 2) {
      setStrokes((prev) => [...prev, currentStroke]);
    }

    setCurrentStroke(null);
    setIsDrawing(false);
  }

  function handleUndo() {
    setStrokes((prev) => prev.slice(0, -1));
  }

  function handleClear() {
    setStrokes([]);
  }

  function handleExport() {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const dataUrl = canvas.toDataURL("image/png");
    onExport(dataUrl);
  }

  if (error) {
    return (
      <div className="annotation-overlay">
        <div className="annotation-error">
          <p>{error}</p>
          <button onClick={onCancel} className="btn">
            Close
          </button>
        </div>
      </div>
    );
  }

  if (!imageLoaded) {
    return (
      <div className="annotation-overlay">
        <div className="annotation-loading">Loading screenshot...</div>
      </div>
    );
  }

  return (
    <div className="annotation-overlay">
      <div className="annotation-toolbar">
        <button onClick={handleUndo} className="btn" disabled={strokes.length === 0}>
          Undo
        </button>
        <button onClick={handleClear} className="btn" disabled={strokes.length === 0}>
          Clear
        </button>
        <button onClick={handleExport} className="btn btn-primary">
          Export
        </button>
        <button onClick={onCancel} className="btn">
          Cancel
        </button>
      </div>
      <div className="annotation-canvas-container">
        <canvas
          ref={canvasRef}
          onMouseDown={handlePointerDown}
          onMouseMove={handlePointerMove}
          onMouseUp={handlePointerUp}
          onMouseLeave={handlePointerUp}
          onTouchStart={handlePointerDown}
          onTouchMove={handlePointerMove}
          onTouchEnd={handlePointerUp}
          className="annotation-canvas"
        />
      </div>
    </div>
  );
}
