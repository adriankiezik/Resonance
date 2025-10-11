//! Mesh asset loading.
//!
//! Loads mesh files (OBJ, GLTF) into a common mesh format.

use crate::loader::{AssetLoader, LoadError};
use glam::{Vec2, Vec3};
use std::path::Path;

/// Loaded mesh data
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

    /// Create a fallback cube mesh
    pub fn fallback_cube() -> Self {
        // Simple unit cube
        let positions = vec![
            // Front face
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
            // Back face
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            // Top face
            Vec3::new(-0.5, 0.5, -0.5),
            Vec3::new(-0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, 0.5, -0.5),
            // Bottom face
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, -0.5, 0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            // Right face
            Vec3::new(0.5, -0.5, -0.5),
            Vec3::new(0.5, 0.5, -0.5),
            Vec3::new(0.5, 0.5, 0.5),
            Vec3::new(0.5, -0.5, 0.5),
            // Left face
            Vec3::new(-0.5, -0.5, -0.5),
            Vec3::new(-0.5, -0.5, 0.5),
            Vec3::new(-0.5, 0.5, 0.5),
            Vec3::new(-0.5, 0.5, -0.5),
        ];

        let normals = vec![
            // Front
            Vec3::Z, Vec3::Z, Vec3::Z, Vec3::Z,
            // Back
            Vec3::NEG_Z, Vec3::NEG_Z, Vec3::NEG_Z, Vec3::NEG_Z,
            // Top
            Vec3::Y, Vec3::Y, Vec3::Y, Vec3::Y,
            // Bottom
            Vec3::NEG_Y, Vec3::NEG_Y, Vec3::NEG_Y, Vec3::NEG_Y,
            // Right
            Vec3::X, Vec3::X, Vec3::X, Vec3::X,
            // Left
            Vec3::NEG_X, Vec3::NEG_X, Vec3::NEG_X, Vec3::NEG_X,
        ];

        let uvs = vec![
            // Front
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            // Back
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            // Top
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            // Bottom
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            // Right
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
            // Left
            Vec2::new(0.0, 0.0), Vec2::new(1.0, 0.0), Vec2::new(1.0, 1.0), Vec2::new(0.0, 1.0),
        ];

        #[rustfmt::skip]
        let indices = vec![
            0, 1, 2, 2, 3, 0,       // Front
            4, 5, 6, 6, 7, 4,       // Back
            8, 9, 10, 10, 11, 8,    // Top
            12, 13, 14, 14, 15, 12, // Bottom
            16, 17, 18, 18, 19, 16, // Right
            20, 21, 22, 22, 23, 20, // Left
        ];

        Self {
            positions,
            normals,
            uvs,
            indices,
        }
    }

    /// Get vertex count
    pub fn vertex_count(&self) -> usize {
        self.positions.len()
    }

    /// Get triangle count
    pub fn triangle_count(&self) -> usize {
        self.indices.len() / 3
    }
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

/// OBJ mesh loader
pub struct ObjLoader;

impl AssetLoader for ObjLoader {
    type Asset = Vec<MeshData>; // OBJ can contain multiple meshes

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

            // Convert positions
            let positions: Vec<Vec3> = mesh
                .positions
                .chunks(3)
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();

            // Convert normals
            let normals: Vec<Vec3> = if mesh.normals.is_empty() {
                // Generate flat normals if not provided
                vec![Vec3::Y; positions.len()]
            } else {
                mesh.normals
                    .chunks(3)
                    .map(|n| Vec3::new(n[0], n[1], n[2]))
                    .collect()
            };

            // Convert UVs
            let uvs: Vec<Vec2> = if mesh.texcoords.is_empty() {
                // Default UVs if not provided
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

/// GLTF mesh loader
pub struct GltfLoader;

impl AssetLoader for GltfLoader {
    type Asset = Vec<MeshData>; // GLTF can contain multiple meshes

    fn load(&self, path: &Path) -> Result<Self::Asset, LoadError> {
        let (document, buffers, _images) = gltf::import(path)
            .map_err(|e| LoadError::LoadFailed(format!("Failed to load GLTF: {}", e)))?;

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                // Read positions
                let positions: Vec<Vec3> = reader
                    .read_positions()
                    .ok_or_else(|| LoadError::LoadFailed("Missing positions in GLTF mesh".into()))?
                    .map(|p| Vec3::from_array(p))
                    .collect();

                // Read normals (or generate default)
                let normals: Vec<Vec3> = reader
                    .read_normals()
                    .map(|iter| iter.map(|n| Vec3::from_array(n)).collect())
                    .unwrap_or_else(|| vec![Vec3::Y; positions.len()]);

                // Read UVs (or generate default)
                let uvs: Vec<Vec2> = reader
                    .read_tex_coords(0)
                    .map(|iter| {
                        iter.into_f32()
                            .map(|uv| Vec2::from_array(uv))
                            .collect()
                    })
                    .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

                // Read indices
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

/// Load mesh from raw bytes (OBJ only for now)
pub fn load_mesh_from_bytes(_bytes: &[u8], format: MeshFormat) -> Result<Vec<MeshData>, LoadError> {
    match format {
        MeshFormat::Obj => {
            // tobj requires a file path, so we'd need to write to temp file
            // For now, return an error
            Err(LoadError::UnsupportedType(
                "Loading OBJ from bytes not supported yet".into(),
            ))
        }
        MeshFormat::Gltf => {
            // GLTF from bytes is complex, return error for now
            Err(LoadError::UnsupportedType(
                "Loading GLTF from bytes not supported yet".into(),
            ))
        }
    }
}

/// Mesh file format
#[derive(Debug, Clone, Copy)]
pub enum MeshFormat {
    Obj,
    Gltf,
}
