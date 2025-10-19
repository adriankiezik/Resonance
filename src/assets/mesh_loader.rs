
use crate::assets::loader::{AssetLoader, LoadError};
use glam::{Vec2, Vec3};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct MeshData {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub indices: Vec<u32>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            indices: Vec::new(),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Asset = Vec<MeshData>;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let (models, _materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|e| LoadError::LoadFailed(format!("Failed to load OBJ: {}", e)))?;

        let mut meshes = Vec::new();

        for model in models {
            let mesh = &model.mesh;

            let positions: Vec<Vec3> = mesh
                .positions
                .chunks(3)
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();

            let normals: Vec<Vec3> = if mesh.normals.is_empty() {

                vec![Vec3::Y; positions.len()]
            } else {
                mesh.normals
                    .chunks(3)
                    .map(|n| Vec3::new(n[0], n[1], n[2]))
                    .collect()
            };

            let uvs: Vec<Vec2> = if mesh.texcoords.is_empty() {

                vec![Vec2::ZERO; positions.len()]
            } else {
                mesh.texcoords
                    .chunks(2)
                    .map(|uv| Vec2::new(uv[0], uv[1]))
                    .collect()
            };

            meshes.push(MeshData {
                positions,
                normals,
                uvs,
                indices: mesh.indices.clone(),
            });
        }

        if meshes.is_empty() {
            return Err(LoadError::LoadFailed("No meshes found in OBJ file".into()));
        }

        log::info!(
            "Loaded OBJ with {} mesh(es) from {:?}",
            meshes.len(),
            path
        );

        Ok(meshes)
    }

    fn extensions(&self) -> &[&str] {
        &["obj"]
    }
}

pub struct GltfLoader;

impl AssetLoader for GltfLoader {
    type Asset = Vec<MeshData>;

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to load GLTF: {}", e)))?;

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                let positions: Vec<Vec3> = reader
                    .read_positions()
                    .ok_or_else(|| LoadError::LoadFailed("Missing positions in GLTF mesh".into()))?
                    .map(|p| Vec3::from_array(p))
                    .collect();

                let normals: Vec<Vec3> = reader
                    .read_normals()
                    .map(|iter| iter.map(|n| Vec3::from_array(n)).collect())
                    .unwrap_or_else(|| vec![Vec3::Y; positions.len()]);

                let uvs: Vec<Vec2> = reader
                    .read_tex_coords(0)
                    .map(|iter| {
                        iter.into_f32()
                            .map(|uv| Vec2::from_array(uv))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

                let indices: Vec<u32> = reader
                    .read_indices()
                    .ok_or_else(|| LoadError::LoadFailed("Missing indices in GLTF mesh".into()))?
                    .into_u32()
                    .collect();

                meshes.push(MeshData {
                    positions,
                    normals,
                    uvs,
                    indices,
                });
            }
        }

        if meshes.is_empty() {
            return Err(LoadError::LoadFailed("No meshes found in GLTF file".into()));
        }

        log::info!(
            "Loaded GLTF with {} mesh(es) from {:?}",
            meshes.len(),
            path
        );

        Ok(meshes)
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }
}

pub fn load_mesh_from_bytes(_bytes: &[u8], format: MeshFormat) -> Result<Vec<MeshData>, LoadError> {
    match format {
        MeshFormat::Obj => {

            Err(LoadError::UnsupportedType(
                "Loading OBJ from bytes not supported yet".into(),
            ))
        }
        MeshFormat::Gltf => {

            Err(LoadError::UnsupportedType(
                "Loading GLTF from bytes not supported yet".into(),
            ))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum MeshFormat {
    Obj,
    Gltf,
}
