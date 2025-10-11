import { create } from 'zustand';
import type { Entity, EntityId, SceneInfo } from '@/lib/types';
import { entityApi, sceneApi } from '@/lib/api';

interface EditorStore {
  // Scene state
  sceneInfo: SceneInfo | null;
  entities: Entity[];

  // Selection state
  selectedEntityId: EntityId | null;

  // UI state
  isPlaying: boolean;

  // Actions
  setSceneInfo: (info: SceneInfo | null) => void;
  setEntities: (entities: Entity[]) => void;
  setSelectedEntityId: (id: EntityId | null) => void;
  setIsPlaying: (playing: boolean) => void;

  // Async actions
  loadScene: (path: string) => Promise<void>;
  createScene: (name: string) => Promise<void>;
  saveScene: (path: string) => Promise<void>;
  refreshHierarchy: () => Promise<void>;
  createEntity: (name?: string, parent?: EntityId) => Promise<void>;
  deleteEntity: (entityId: EntityId) => Promise<void>;
}

export const useEditorStore = create<EditorStore>((set, get) => ({
  // Initial state
  sceneInfo: null,
  entities: [],
  selectedEntityId: null,
  isPlaying: false,

  // Setters
  setSceneInfo: (info) => set({ sceneInfo: info }),
  setEntities: (entities) => set({ entities }),
  setSelectedEntityId: (id) => set({ selectedEntityId: id }),
  setIsPlaying: (playing) => set({ isPlaying: playing }),

  // Async actions
  loadScene: async (path: string) => {
    try {
      const info = await sceneApi.loadScene(path);
      set({ sceneInfo: info });
      await get().refreshHierarchy();
    } catch (error) {
      console.error('Failed to load scene:', error);
      throw error;
    }
  },

  createScene: async (name: string) => {
    try {
      const info = await sceneApi.createScene(name);
      set({ sceneInfo: info, entities: [] });
    } catch (error) {
      console.error('Failed to create scene:', error);
      throw error;
    }
  },

  saveScene: async (path: string) => {
    try {
      await sceneApi.saveScene(path);
    } catch (error) {
      console.error('Failed to save scene:', error);
      throw error;
    }
  },

  refreshHierarchy: async () => {
    try {
      const entities = await entityApi.getEntityHierarchy();
      set({ entities });
    } catch (error) {
      console.error('Failed to refresh hierarchy:', error);
      throw error;
    }
  },

  createEntity: async (name?: string, parent?: EntityId) => {
    try {
      await entityApi.createEntity(name, parent);
      await get().refreshHierarchy();
    } catch (error) {
      console.error('Failed to create entity:', error);
      throw error;
    }
  },

  deleteEntity: async (entityId: EntityId) => {
    try {
      await entityApi.deleteEntity(entityId);
      if (get().selectedEntityId === entityId) {
        set({ selectedEntityId: null });
      }
      await get().refreshHierarchy();
    } catch (error) {
      console.error('Failed to delete entity:', error);
      throw error;
    }
  },
}));
