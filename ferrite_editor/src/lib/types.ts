// Core editor types

export type EntityId = number;

export interface Vec3 {
  x: number;
  y: number;
  z: number;
}

export interface Quat {
  x: number;
  y: number;
  z: number;
  w: number;
}

export interface Transform {
  position: Vec3;
  rotation: Quat;
  scale: Vec3;
}

export interface Entity {
  id: EntityId;
  name: string;
  parent?: EntityId;
  children: EntityId[];
  components: Record<string, ComponentData>;
}

export type ComponentData =
  | TransformComponent
  | MeshComponent
  | MaterialComponent
  | CameraComponent
  | RigidBodyComponent
  | ColliderComponent
  | AudioSourceComponent;

export interface TransformComponent {
  type: 'Transform';
  data: Transform;
}

export interface MeshComponent {
  type: 'Mesh';
  data: {
    mesh_type: 'Cube' | 'Sphere' | 'Plane' | 'Triangle' | 'Quad';
    size?: number;
    radius?: number;
    segments?: number;
    rings?: number;
    subdivisions?: number;
  };
}

export interface MaterialComponent {
  type: 'Material';
  data: {
    color: [number, number, number, number];
    texture?: string;
  };
}

export interface CameraComponent {
  type: 'Camera';
  data: {
    projection: 'Perspective' | 'Orthographic';
    fov?: number;
    near: number;
    far: number;
  };
}

export interface RigidBodyComponent {
  type: 'RigidBody';
  data: 'Dynamic' | 'Kinematic' | 'Static';
}

export interface ColliderComponent {
  type: 'Collider';
  data: {
    shape: 'Box' | 'Sphere' | 'Capsule';
    size: Vec3;
  };
}

export interface AudioSourceComponent {
  type: 'AudioSource';
  data: {
    audio_path: string;
    volume: number;
    looping: boolean;
    spatial: boolean;
  };
}

export interface SceneInfo {
  name: string;
  entity_count: number;
  active: boolean;
}

export interface EditorState {
  selected_entity?: EntityId;
  camera_position: Vec3;
  camera_rotation: Quat;
  play_mode: boolean;
}
