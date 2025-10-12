import { useState } from "react";

interface Entity {
  id: string;
  name: string;
  children?: Entity[];
}

export function Sidebar() {
  const [entities] = useState<Entity[]>([
    {
      id: "1",
      name: "Scene Root",
      children: [
        { id: "2", name: "Camera" },
        { id: "3", name: "Light" },
        {
          id: "4",
          name: "Objects",
          children: [
            { id: "5", name: "Cube 1" },
            { id: "6", name: "Cube 2" },
            { id: "7", name: "Cube 3" },
          ],
        },
      ],
    },
  ]);

  const [selectedId, setSelectedId] = useState<string | null>(null);

  const renderEntity = (entity: Entity, depth: number = 0) => {
    const isSelected = entity.id === selectedId;

    return (
      <div key={entity.id}>
        <div
          className={`flex items-center gap-2 px-3 py-1.5 hover:bg-accent cursor-pointer ${
            isSelected ? "bg-accent" : ""
          }`}
          style={{ paddingLeft: `${depth * 16 + 12}px` }}
          onClick={() => setSelectedId(entity.id)}
        >
          {entity.children && entity.children.length > 0 && (
            <span className="text-xs">▼</span>
          )}
          <span className="text-sm">{entity.name}</span>
        </div>
        {entity.children?.map((child) => renderEntity(child, depth + 1))}
      </div>
    );
  };

  return (
    <div className="h-full flex flex-col bg-background/95 backdrop-blur border-r border-border">
      {/* Header */}
      <div className="px-3 py-2 border-b border-border">
        <h2 className="text-sm font-semibold">Scene Hierarchy</h2>
      </div>

      {/* Entity list */}
      <div className="flex-1 overflow-y-auto">
        {entities.map((entity) => renderEntity(entity))}
      </div>

      {/* Footer */}
      <div className="px-3 py-2 border-t border-border flex items-center gap-2">
        <button
          className="px-2 py-1 text-xs rounded bg-primary text-primary-foreground hover:bg-primary/90"
          onClick={() => console.log("Add entity")}
        >
          + Add
        </button>
        <button
          className="px-2 py-1 text-xs rounded border border-border hover:bg-accent"
          onClick={() => console.log("Delete entity")}
          disabled={!selectedId}
        >
          Delete
        </button>
      </div>
    </div>
  );
}
