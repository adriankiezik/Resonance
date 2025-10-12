import { useEffect, useRef, useState } from "react";
import init, { Viewport } from "../../wasm-viewport/ferrite_viewport_wasm";

interface CameraInfo {
  position: [number, number, number];
  fov: number;
}

export function Viewport3D() {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const containerRef = useRef<HTMLDivElement>(null);
  const viewportRef = useRef<Viewport | null>(null);
  const [isInitialized, setIsInitialized] = useState(false);
  const [cameraInfo, setCameraInfo] = useState<CameraInfo | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [fps, setFps] = useState(0);

  // Mouse state for camera controls
  const mouseState = useRef({
    isOrbiting: false, // Alt + Left mouse
    isPanning: false, // Middle mouse
    lastX: 0,
    lastY: 0,
  });

  // Animation frame ID
  const animationFrameRef = useRef<number | null>(null);
  const lastFrameTimeRef = useRef<number>(0);
  const frameCountRef = useRef<number>(0);

  // Initialize viewport
  useEffect(() => {
    const initViewport = async () => {
      try {
        if (!canvasRef.current || !containerRef.current) return;

        console.log("Initializing WebGPU viewport...");

        // Initialize WASM module
        await init();

        const canvas = canvasRef.current;
        const rect = containerRef.current.getBoundingClientRect();

        // Set canvas size
        canvas.width = Math.floor(rect.width);
        canvas.height = Math.floor(rect.height);

        // Create viewport
        const viewport = await new Viewport(canvas);
        viewportRef.current = viewport;

        setIsInitialized(true);
        setError(null);
        console.log("WebGPU viewport initialized successfully");

        // Start render loop
        startRenderLoop();
      } catch (err) {
        console.error("Failed to initialize viewport:", err);
        setError(String(err));
      }
    };

    initViewport();

    // Cleanup
    return () => {
      if (animationFrameRef.current) {
        cancelAnimationFrame(animationFrameRef.current);
      }
    };
  }, []);

  // Render loop
  const startRenderLoop = () => {
    const render = () => {
      try {
        const viewport = viewportRef.current;
        if (!viewport) {
          animationFrameRef.current = requestAnimationFrame(render);
          return;
        }

        // Render frame (native 60 FPS WebGPU rendering!)
        viewport.render();

        // Update FPS
        frameCountRef.current++;
        const now = performance.now();
        if (now - lastFrameTimeRef.current >= 1000) {
          setFps(frameCountRef.current);
          frameCountRef.current = 0;
          lastFrameTimeRef.current = now;

          // Update camera info
          const position = viewport.get_camera_position();
          const fov = viewport.get_camera_fov();
          setCameraInfo({
            position: [position[0], position[1], position[2]],
            fov,
          });
        }
      } catch (err) {
        console.error("Render error:", err);
      }

      // Continue loop
      animationFrameRef.current = requestAnimationFrame(render);
    };

    render();
  };

  // Handle resize
  useEffect(() => {
    if (!isInitialized) return;

    const handleResize = () => {
      if (!containerRef.current || !canvasRef.current || !viewportRef.current)
        return;

      const rect = containerRef.current.getBoundingClientRect();
      const width = Math.floor(rect.width);
      const height = Math.floor(rect.height);

      // Update canvas size
      canvasRef.current.width = width;
      canvasRef.current.height = height;

      // Resize viewport
      viewportRef.current.resize(width, height);
    };

    const resizeObserver = new ResizeObserver(() => {
      handleResize();
    });

    if (containerRef.current) {
      resizeObserver.observe(containerRef.current);
    }

    return () => {
      resizeObserver.disconnect();
    };
  }, [isInitialized]);

  // Mouse controls
  const handleMouseDown = (e: React.MouseEvent) => {
    const state = mouseState.current;

    if ((e.button === 0 && e.altKey) || e.button === 1) {
      // Alt + Left click or Middle click = Orbit/Pan
      state.isOrbiting = e.button === 0 && e.altKey;
      state.isPanning = e.button === 1;
      state.lastX = e.clientX;
      state.lastY = e.clientY;
      e.preventDefault();
    }
  };

  const handleMouseMove = (e: React.MouseEvent) => {
    const state = mouseState.current;

    if (!isInitialized || !viewportRef.current) return;
    if (!state.isOrbiting && !state.isPanning) return;

    const deltaX = e.clientX - state.lastX;
    const deltaY = e.clientY - state.lastY;

    try {
      if (state.isOrbiting) {
        viewportRef.current.orbit_camera(deltaX, deltaY);
      } else if (state.isPanning) {
        viewportRef.current.pan_camera(-deltaX, deltaY);
      }
    } catch (err) {
      console.error("Camera control error:", err);
    }

    state.lastX = e.clientX;
    state.lastY = e.clientY;
  };

  const handleMouseUp = () => {
    const state = mouseState.current;
    state.isOrbiting = false;
    state.isPanning = false;
  };

  const handleWheel = (e: React.WheelEvent) => {
    if (!isInitialized || !viewportRef.current) return;

    e.preventDefault();

    try {
      viewportRef.current.zoom_camera(e.deltaY * 0.01);
    } catch (err) {
      console.error("Zoom error:", err);
    }
  };

  const formatVec3 = (v: [number, number, number]) => {
    return `(${v[0].toFixed(2)}, ${v[1].toFixed(2)}, ${v[2].toFixed(2)})`;
  };

  return (
    <div className="flex h-full flex-col bg-background">
      {/* Toolbar */}
      <div className="flex items-center gap-2 border-b border-border bg-muted/40 px-3 py-2">
        <span className="text-sm font-medium">3D Viewport (WebGPU)</span>
        <div className="mx-2 h-4 w-px bg-border" />
        <span className="text-xs text-muted-foreground">
          {isInitialized ? (
            <span className="text-green-500">● Rendering @ {fps} FPS</span>
          ) : error ? (
            <span className="text-red-500">● Error</span>
          ) : (
            "Initializing..."
          )}
        </span>
      </div>

      {/* Viewport Content */}
      <div
        ref={containerRef}
        className="relative flex-1 cursor-grab active:cursor-grabbing overflow-hidden"
        onMouseDown={handleMouseDown}
        onMouseMove={handleMouseMove}
        onMouseUp={handleMouseUp}
        onMouseLeave={handleMouseUp}
        onWheel={handleWheel}
      >
        <canvas
          ref={canvasRef}
          className="w-full h-full"
          style={{ display: isInitialized ? "block" : "none" }}
        />

        {/* Error Message */}
        {error && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="rounded-lg border border-destructive bg-destructive/10 p-6 text-center">
              <p className="font-semibold text-destructive">Viewport Error</p>
              <p className="mt-2 text-sm text-muted-foreground">{error}</p>
              <p className="mt-2 text-xs text-muted-foreground">
                Note: WebGPU requires a modern browser (Chrome 113+, Edge 113+,
                Safari 16.4+)
              </p>
            </div>
          </div>
        )}

        {/* Loading State */}
        {!isInitialized && !error && (
          <div className="absolute inset-0 flex items-center justify-center">
            <div className="text-center">
              <div className="mb-4 inline-block h-8 w-8 animate-spin rounded-full border-4 border-border border-t-primary"></div>
              <p className="text-sm text-muted-foreground">
                Initializing WebGPU viewport...
              </p>
            </div>
          </div>
        )}

        {/* Camera Info Overlay */}
        {isInitialized && cameraInfo && (
          <div className="absolute bottom-3 right-3 space-y-1 rounded-lg border border-border bg-background/95 px-3 py-2 text-xs font-mono backdrop-blur">
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">Position:</span>
              <span>{formatVec3(cameraInfo.position)}</span>
            </div>
            <div className="flex items-center gap-2">
              <span className="text-muted-foreground">FOV:</span>
              <span>{cameraInfo.fov.toFixed(1)}°</span>
            </div>
          </div>
        )}

        {/* Controls Help */}
        {isInitialized && (
          <div className="absolute left-3 top-3 space-y-1 rounded-lg border border-border bg-background/95 px-3 py-2 text-xs backdrop-blur">
            <div className="mb-1 font-semibold">Controls:</div>
            <div className="text-muted-foreground">Alt + Drag: Orbit</div>
            <div className="text-muted-foreground">Middle Drag: Pan</div>
            <div className="text-muted-foreground">Scroll: Zoom</div>
          </div>
        )}
      </div>
    </div>
  );
}
