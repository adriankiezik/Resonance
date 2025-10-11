import { useEffect } from 'react';
import { useEditorStore } from '@/stores/editorStore';
import { ScrollArea } from '../ui/scroll-area';
import { Button } from '../ui/button';
import { Plus, Trash2, ChevronRight, ChevronDown } from 'lucide-react';
import { cn } from '@/lib/utils';
import type { Entity } from '@/lib/types';

export function SceneHierarchy() {
  const {
    entities,
    selectedEntityId,
    setSelectedEntityId,
    refreshHierarchy,
    createEntity,
    deleteEntity,
  } = useEditorStore();

  useEffect(() => {
    refreshHierarchy().catch(console.error);
  }, [refreshHierarchy]);

  const handleCreateEntity = async () => {
    try {
      await createEntity('Entity');
    } catch (error) {
      console.error('Failed to create entity:', error);
    }
  };

  const handleDeleteEntity = async () => {
    if (selectedEntityId) {
      try {
        await deleteEntity(selectedEntityId);
      } catch (error) {
        console.error('Failed to delete entity:', error);
      }
    }
  };

  return (
    <div className="flex h-full flex-col border-r border-border bg-card">
      {/* Header */}
      <div className="flex items-center justify-between border-b border-border px-3 py-2">
        <h2 className="text-sm font-semibold">Scene Hierarchy</h2>
        <div className="flex gap-1">
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleCreateEntity}
            title="Create Entity"
          >
            <Plus className="h-4 w-4" />
          </Button>
          <Button
            variant="ghost"
            size="icon"
            className="h-7 w-7"
            onClick={handleDeleteEntity}
            disabled={!selectedEntityId}
            title="Delete Entity"
          >
            <Trash2 className="h-4 w-4" />
          </Button>
        </div>
      </div>

      {/* Entity List */}
      <ScrollArea className="flex-1">
        <div className="p-2 space-y-1">
          {entities.length === 0 ? (
            <div className="flex h-32 items-center justify-center text-sm text-muted-foreground">
              No entities in scene
            </div>
          ) : (
            entities.map((entity) => (
              <EntityTreeNode
                key={entity.id}
                entity={entity}
                selectedId={selectedEntityId}
                onSelect={setSelectedEntityId}
              />
            ))
          )}
        </div>
      </ScrollArea>
    </div>
  );
}

interface EntityTreeNodeProps {
  entity: Entity;
  selectedId: number | null;
  onSelect: (id: number) => void;
  depth?: number;
}

function EntityTreeNode({
  entity,
  selectedId,
  onSelect,
  depth = 0,
}: EntityTreeNodeProps) {
  const hasChildren = entity.children && entity.children.length > 0;
  const isSelected = entity.id === selectedId;

  return (
    <div>
      <button
        className={cn(
          'flex w-full items-center gap-1 rounded px-2 py-1 text-left text-sm hover:bg-accent',
          isSelected && 'bg-accent'
        )}
        style={{ paddingLeft: `${depth * 12 + 8}px` }}
        onClick={() => onSelect(entity.id)}
      >
        {hasChildren ? (
          <ChevronDown className="h-4 w-4 shrink-0" />
        ) : (
          <ChevronRight className="h-4 w-4 shrink-0 opacity-0" />
        )}
        <span className="truncate">{entity.name}</span>
      </button>

      {/* Render children (if any) */}
      {hasChildren &&
        entity.children.map((_childId) => {
          // Note: In a real implementation, you'd fetch child entities
          // For now, this is a placeholder
          return null;
        })}
    </div>
  );
}
