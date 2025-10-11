export function Viewport3D() {
  return (
    <div className="flex h-full flex-col bg-background">
      {/* Toolbar */}
      <div className="flex items-center gap-2 border-b border-border bg-muted/40 px-3 py-2">
        <span className="text-sm text-muted-foreground">3D Viewport</span>
      </div>

      {/* Viewport Content */}
      <div className="relative flex-1">
        <div className="absolute inset-0 flex items-center justify-center bg-muted/20">
          <div className="text-center">
            <div className="mb-2 text-4xl text-muted-foreground/50">
              <svg
                xmlns="http://www.w3.org/2000/svg"
                width="64"
                height="64"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                strokeWidth="2"
                strokeLinecap="round"
                strokeLinejoin="round"
              >
                <path d="M12 2L2 7l10 5 10-5-10-5z" />
                <path d="M2 17l10 5 10-5" />
                <path d="M2 12l10 5 10-5" />
              </svg>
            </div>
            <p className="text-sm text-muted-foreground">
              3D Viewport (Rendering will be integrated with wgpu)
            </p>
            <p className="mt-2 text-xs text-muted-foreground/75">
              WASD to move • Mouse to look • Scroll to zoom
            </p>
          </div>
        </div>

        {/* Grid */}
        <svg
          className="absolute inset-0 h-full w-full opacity-10"
          xmlns="http://www.w3.org/2000/svg"
        >
          <defs>
            <pattern
              id="grid"
              width="40"
              height="40"
              patternUnits="userSpaceOnUse"
            >
              <path
                d="M 40 0 L 0 0 0 40"
                fill="none"
                stroke="currentColor"
                strokeWidth="1"
              />
            </pattern>
          </defs>
          <rect width="100%" height="100%" fill="url(#grid)" />
        </svg>

        {/* Corner Info */}
        <div className="absolute bottom-2 right-2 rounded bg-background/90 px-2 py-1 text-xs text-muted-foreground">
          Camera: Perspective
        </div>
      </div>
    </div>
  );
}
