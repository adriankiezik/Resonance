import { useEffect, useState } from "react";
import { invoke } from "@tauri-apps/api/core";

interface CameraInfo {
  position: [number, number, number];
  rotation: [number, number, number, number];
  fov: number;
}

export function Toolbar() {
  const [cameraInfo, setCameraInfo] = useState<CameraInfo | null>(null);

  useEffect(() => {
    // Poll camera info periodically
    const interval = setInterval(async () => {
      try {
        const info = await invoke<CameraInfo>("get_camera_info", {
          windowLabel: "main",
        });
        setCameraInfo(info);
      } catch (err) {
        console.error("Failed to get camera info:", err);
      }
    }, 100);

    return () => clearInterval(interval);
  }, []);

  const formatVec3 = (v: [number, number, number]) => {
    return `(${v[0].toFixed(2)}, ${v[1].toFixed(2)}, ${v[2].toFixed(2)})`;
  };

  return (
    <div
      className="h-full flex items-center px-4 gap-4 bg-background/95 backdrop-blur border-b border-border"
      style={{ WebkitAppRegion: "drag" } as any}
    >
      <h1 className="text-sm font-semibold" style={{ WebkitAppRegion: "no-drag" } as any}>
        Ferrite Scene Editor
      </h1>

      <div className="h-4 w-px bg-border" />

      <div className="flex items-center gap-2 text-xs text-muted-foreground" style={{ WebkitAppRegion: "no-drag" } as any}>
        {cameraInfo && (
          <>
            <span>Camera: {formatVec3(cameraInfo.position)}</span>
            <span>FOV: {cameraInfo.fov.toFixed(1)}°</span>
          </>
        )}
      </div>

      <div className="ml-auto flex items-center gap-2" style={{ WebkitAppRegion: "no-drag" } as any}>
        <button
          className="px-3 py-1 text-xs rounded bg-primary text-primary-foreground hover:bg-primary/90"
          onClick={() => console.log("Play clicked")}
        >
          ▶ Play
        </button>
      </div>
    </div>
  );
}
