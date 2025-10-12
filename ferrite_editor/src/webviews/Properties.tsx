export function Properties() {
  return (
    <div className="h-full flex flex-col bg-background/95 backdrop-blur border-l border-border">
      {/* Header */}
      <div className="px-3 py-2 border-b border-border">
        <h2 className="text-sm font-semibold">Properties</h2>
      </div>

      {/* Content */}
      <div className="flex-1 overflow-y-auto p-3 space-y-4">
        {/* Transform Section */}
        <div>
          <h3 className="text-xs font-semibold text-muted-foreground uppercase mb-2">
            Transform
          </h3>
          <div className="space-y-2">
            <div>
              <label className="text-xs text-muted-foreground">Position</label>
              <div className="grid grid-cols-3 gap-2 mt-1">
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="X"
                  defaultValue={0}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Y"
                  defaultValue={0}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Z"
                  defaultValue={0}
                />
              </div>
            </div>

            <div>
              <label className="text-xs text-muted-foreground">Rotation</label>
              <div className="grid grid-cols-3 gap-2 mt-1">
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="X"
                  defaultValue={0}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Y"
                  defaultValue={0}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Z"
                  defaultValue={0}
                />
              </div>
            </div>

            <div>
              <label className="text-xs text-muted-foreground">Scale</label>
              <div className="grid grid-cols-3 gap-2 mt-1">
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="X"
                  defaultValue={1}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Y"
                  defaultValue={1}
                />
                <input
                  type="number"
                  className="px-2 py-1 text-xs rounded bg-background border border-border"
                  placeholder="Z"
                  defaultValue={1}
                />
              </div>
            </div>
          </div>
        </div>

        {/* Components Section */}
        <div>
          <h3 className="text-xs font-semibold text-muted-foreground uppercase mb-2">
            Components
          </h3>
          <div className="space-y-2">
            <div className="px-3 py-2 rounded bg-muted/50 border border-border">
              <div className="flex items-center justify-between">
                <span className="text-sm font-medium">MeshRenderer</span>
                <button className="text-xs text-muted-foreground hover:text-foreground">
                  ×
                </button>
              </div>
            </div>

            <button className="w-full px-3 py-2 text-xs rounded border border-dashed border-border hover:bg-accent">
              + Add Component
            </button>
          </div>
        </div>
      </div>
    </div>
  );
}
