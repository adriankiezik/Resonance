import { useEffect, useState, useCallback } from 'react';
import { useEditorStore } from '@/stores/editorStore';
import { componentApi } from '@/lib/api';
import { ScrollArea } from '../ui/scroll-area';
import { Label } from '../ui/label';
import { Input } from '../ui/input';
import { Button } from '../ui/button';
import { Separator } from '../ui/separator';
import { Plus } from 'lucide-react';
import type { ComponentData, TransformComponent } from '@/lib/types';

export function Inspector() {
  const { selectedEntityId } = useEditorStore();
  const [components, setComponents] = useState<ComponentData[]>([]);

  const loadComponents = useCallback(async () => {
    if (!selectedEntityId) return;

    try {
      const comps = await componentApi.getEntityComponents(selectedEntityId);
      setComponents(comps);
    } catch (error) {
      console.error('Failed to load components:', error);
    }
  }, [selectedEntityId]);

  useEffect(() => {
    if (selectedEntityId) {
      loadComponents();
    } else {
      setComponents([]);
    }
  }, [selectedEntityId, loadComponents]);

  if (!selectedEntityId) {
    return (
      <div className="flex h-full flex-col border-l border-border bg-card">
        <div className="border-b border-border px-3 py-2">
          <h2 className="text-sm font-semibold">Inspector</h2>
        </div>
        <div className="flex flex-1 items-center justify-center text-sm text-muted-foreground">
          No entity selected
        </div>
      </div>
    );
  }

  return (
    <div className="flex h-full flex-col border-l border-border bg-card">
      {/* Header */}
      <div className="border-b border-border px-3 py-2">
        <h2 className="text-sm font-semibold">Inspector</h2>
      </div>

      {/* Components */}
      <ScrollArea className="flex-1">
        <div className="p-3 space-y-4">
          {components.map((component, index) => (
            <ComponentEditor
              key={index}
              component={component}
              entityId={selectedEntityId}
              onUpdate={loadComponents}
            />
          ))}

          {components.length === 0 && (
            <div className="text-sm text-muted-foreground">
              No components on this entity
            </div>
          )}

          {/* Add Component Button */}
          <Button
            variant="outline"
            size="sm"
            className="w-full"
            onClick={() => console.log('Add component')}
          >
            <Plus className="h-4 w-4" />
            Add Component
          </Button>
        </div>
      </ScrollArea>
    </div>
  );
}

interface ComponentEditorProps {
  component: ComponentData;
  entityId: number;
  onUpdate: () => void;
}

function ComponentEditor({
  component,
  entityId,
  onUpdate,
}: ComponentEditorProps) {
  if (component.type === 'Transform') {
    return <TransformEditor component={component} entityId={entityId} onUpdate={onUpdate} />;
  }

  return (
    <div className="rounded-lg border border-border p-3">
      <h3 className="mb-2 text-sm font-medium">{component.type}</h3>
      <div className="text-xs text-muted-foreground">
        Editor not implemented for this component type
      </div>
    </div>
  );
}

interface TransformEditorProps {
  component: TransformComponent;
  entityId: number;
  onUpdate: () => void;
}

function TransformEditor({ component, entityId, onUpdate }: TransformEditorProps) {
  const [position, setPosition] = useState(component.data.position);
  const [rotation, setRotation] = useState(component.data.rotation);
  const [scale, setScale] = useState(component.data.scale);

  const handleUpdate = async () => {
    try {
      await componentApi.updateComponent(entityId, 'Transform', {
        position,
        rotation,
        scale,
      });
      onUpdate();
    } catch (error) {
      console.error('Failed to update transform:', error);
    }
  };

  return (
    <div className="rounded-lg border border-border p-3">
      <h3 className="mb-3 text-sm font-medium">Transform</h3>

      {/* Position */}
      <div className="mb-3">
        <Label className="text-xs text-muted-foreground">Position</Label>
        <div className="grid grid-cols-3 gap-2 mt-1">
          <div>
            <Label className="text-xs">X</Label>
            <Input
              type="number"
              step="0.1"
              value={position.x}
              onChange={(e) =>
                setPosition({ ...position, x: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
          <div>
            <Label className="text-xs">Y</Label>
            <Input
              type="number"
              step="0.1"
              value={position.y}
              onChange={(e) =>
                setPosition({ ...position, y: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
          <div>
            <Label className="text-xs">Z</Label>
            <Input
              type="number"
              step="0.1"
              value={position.z}
              onChange={(e) =>
                setPosition({ ...position, z: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
        </div>
      </div>

      <Separator className="my-3" />

      {/* Rotation */}
      <div className="mb-3">
        <Label className="text-xs text-muted-foreground">Rotation (Quaternion)</Label>
        <div className="grid grid-cols-4 gap-1 mt-1">
          <div>
            <Label className="text-xs">X</Label>
            <Input
              type="number"
              step="0.01"
              value={rotation.x}
              onChange={(e) =>
                setRotation({ ...rotation, x: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7 text-xs"
            />
          </div>
          <div>
            <Label className="text-xs">Y</Label>
            <Input
              type="number"
              step="0.01"
              value={rotation.y}
              onChange={(e) =>
                setRotation({ ...rotation, y: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7 text-xs"
            />
          </div>
          <div>
            <Label className="text-xs">Z</Label>
            <Input
              type="number"
              step="0.01"
              value={rotation.z}
              onChange={(e) =>
                setRotation({ ...rotation, z: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7 text-xs"
            />
          </div>
          <div>
            <Label className="text-xs">W</Label>
            <Input
              type="number"
              step="0.01"
              value={rotation.w}
              onChange={(e) =>
                setRotation({ ...rotation, w: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7 text-xs"
            />
          </div>
        </div>
      </div>

      <Separator className="my-3" />

      {/* Scale */}
      <div>
        <Label className="text-xs text-muted-foreground">Scale</Label>
        <div className="grid grid-cols-3 gap-2 mt-1">
          <div>
            <Label className="text-xs">X</Label>
            <Input
              type="number"
              step="0.1"
              value={scale.x}
              onChange={(e) =>
                setScale({ ...scale, x: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
          <div>
            <Label className="text-xs">Y</Label>
            <Input
              type="number"
              step="0.1"
              value={scale.y}
              onChange={(e) =>
                setScale({ ...scale, y: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
          <div>
            <Label className="text-xs">Z</Label>
            <Input
              type="number"
              step="0.1"
              value={scale.z}
              onChange={(e) =>
                setScale({ ...scale, z: parseFloat(e.target.value) })
              }
              onBlur={handleUpdate}
              className="h-7"
            />
          </div>
        </div>
      </div>
    </div>
  );
}
