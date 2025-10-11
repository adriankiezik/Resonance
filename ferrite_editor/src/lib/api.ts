import { invoke } from '@tauri-apps/api/core';
import type { Entity, EntityId, ComponentData, SceneInfo } from './types';

// Scene API
export const sceneApi = {
  async createScene(name: string): Promise<SceneInfo> {
    return await invoke('create_scene', { name });
  },

  async loadScene(path: string): Promise<SceneInfo> {
    return await invoke('load_scene', { path });
  },

  async saveScene(path: string): Promise<void> {
    return await invoke('save_scene', { path });
  },

  async getSceneInfo(): Promise<SceneInfo | null> {
    return await invoke('get_scene_info');
  },
};

// Entity API
export const entityApi = {
  async createEntity(
    name?: string,
    parent?: EntityId
  ): Promise<Entity> {
    return await invoke('create_entity', { name, parent });
  },

  async deleteEntity(entityId: EntityId): Promise<void> {
    return await invoke('delete_entity', { entityId });
  },

  async renameEntity(entityId: EntityId, newName: string): Promise<void> {
    return await invoke('rename_entity', { entityId, newName });
  },

  async getEntityHierarchy(): Promise<Entity[]> {
    return await invoke('get_entity_hierarchy');
  },

  async setEntityParent(
    entityId: EntityId,
    parentId?: EntityId
  ): Promise<void> {
    return await invoke('set_entity_parent', { entityId, parentId });
  },

  async getEntity(entityId: EntityId): Promise<Entity> {
    return await invoke('get_entity', { entityId });
  },
};

// Component API
export const componentApi = {
  async addComponent(
    entityId: EntityId,
    componentType: string,
    data: any
  ): Promise<void> {
    return await invoke('add_component', { entityId, componentType, data });
  },

  async updateComponent(
    entityId: EntityId,
    componentType: string,
    data: any
  ): Promise<void> {
    return await invoke('update_component', { entityId, componentType, data });
  },

  async removeComponent(
    entityId: EntityId,
    componentType: string
  ): Promise<void> {
    return await invoke('remove_component', { entityId, componentType });
  },

  async getEntityComponents(entityId: EntityId): Promise<ComponentData[]> {
    return await invoke('get_entity_components', { entityId });
  },
};
