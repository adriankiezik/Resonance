use crate::assets::loader::{AssetLoader, LoadError};
use glam::{Vec2, Vec3};
use std::path::Path;

#[derive(Clone, Debug)]
pub struct MeshData {
    pub positions: Vec<Vec3>,
    pub normals: Vec<Vec3>,
    pub uvs: Vec<Vec2>,
    pub colors: Vec<Vec3>,
    pub indices: Vec<u32>,
    pub texture: Option<std::sync::Arc<crate::assets::TextureData>>,
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            positions: Vec::new(),
            normals: Vec::new(),
            uvs: Vec::new(),
            colors: Vec::new(),
            indices: Vec::new(),
            texture: None,
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
        let (models, materials) = tobj::load_obj(
            path,
            &tobj::LoadOptions {
                triangulate: true,
                single_index: true,
                ..Default::default()
            },
        )
        .map_err(|e| LoadError::LoadFailed(format!("Failed to load OBJ: {}", e)))?;

        let materials = materials.map_err(|e| LoadError::LoadFailed(format!("Failed to load MTL: {}", e)))?;

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

            let (color, texture) = if let Some(material_id) = mesh.material_id {
                if let Some(material) = materials.get(material_id) {
                    let color = if let Some(diffuse) = material.diffuse {
                        let c = Vec3::new(diffuse[0], diffuse[1], diffuse[2]);
                        log::info!("Mesh '{}' using material {} with color: {:?}", model.name, material_id, c);
                        c
                    } else {
                        log::warn!("Mesh '{}' material {} has no diffuse color, using white", model.name, material_id);
                        Vec3::ONE
                    };

                    let texture = if let Some(ref texture_path) = material.diffuse_texture {
                        let texture_full_path = path.parent().unwrap_or(Path::new(".")).join(texture_path);
                        match image::open(&texture_full_path) {
                            Ok(img) => {
                                log::info!("Loaded texture for mesh '{}': {:?}", model.name, texture_path);
                                Some(std::sync::Arc::new(crate::assets::TextureData::from_image(img)))
                            }
                            Err(e) => {
                                log::warn!("Failed to load texture {:?} for mesh '{}': {}", texture_path, model.name, e);
                                None
                            }
                        }
                    } else {
                        None
                    };

                    (color, texture)
                } else {
                    log::warn!("Mesh '{}' references invalid material_id {}, using white", model.name, material_id);
                    (Vec3::ONE, None)
                }
            } else {
                log::info!("Mesh '{}' has no material, using white", model.name);
                (Vec3::ONE, None)
            };

            let colors = vec![color; positions.len()];

            meshes.push(MeshData {
                positions,
                normals,
                uvs,
                colors,
                indices: mesh.indices.clone(),
                texture,
            });
        }

        if meshes.is_empty() {
            return Err(LoadError::LoadFailed("No meshes found in OBJ file".into()));
        }

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
        let (document, buffers, images) = gltf::import(path)
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
                    .map(|iter| iter.into_f32().map(|uv| Vec2::from_array(uv)).collect())
                    .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

                let indices: Vec<u32> = reader
                    .read_indices()
                    .ok_or_else(|| LoadError::LoadFailed("Missing indices in GLTF mesh".into()))?
                    .into_u32()
                    .collect();

                let colors = vec![Vec3::ONE; positions.len()];

                let texture = primitive.material()
                    .pbr_metallic_roughness()
                    .base_color_texture()
                    .and_then(|info| {
                        let image_index = info.texture().source().index();
                        images.get(image_index).map(|img_data| {
                            let width = img_data.width;
                            let height = img_data.height;
                            let data = img_data.pixels.clone();
                            let format = match img_data.format {
                                gltf::image::Format::R8G8B8A8 => crate::assets::TextureFormat::Rgba8,
                                gltf::image::Format::R8G8B8 => crate::assets::TextureFormat::Rgb8,
                                gltf::image::Format::R8 => crate::assets::TextureFormat::R8,
                                _ => crate::assets::TextureFormat::Rgba8,
                            };
                            log::info!("Loaded texture from GLTF: {}x{} format: {:?}", width, height, format);
                            std::sync::Arc::new(crate::assets::TextureData {
                                width,
                                height,
                                data,
                                format,
                            })
                        })
                    });

                meshes.push(MeshData {
                    positions,
                    normals,
                    uvs,
                    colors,
                    indices,
                    texture,
                });
            }
        }

        if meshes.is_empty() {
            return Err(LoadError::LoadFailed("No meshes found in GLTF file".into()));
        }

        Ok(meshes)
    }

    fn extensions(&self) -> &[&str] {
        &["gltf", "glb"]
    }
}

pub fn load_mesh_from_bytes(bytes: &[u8], format: MeshFormat) -> Result<Vec<MeshData>, LoadError> {
    match format {
        MeshFormat::Obj => load_obj_from_bytes(bytes),
        MeshFormat::Gltf => load_gltf_from_bytes(bytes),
    }
}

fn load_obj_from_bytes(bytes: &[u8]) -> Result<Vec<MeshData>, LoadError> {
    use std::io::Cursor;

    let cursor = Cursor::new(bytes);
    let mut reader = std::io::BufReader::new(cursor);

    let (models, materials) = tobj::load_obj_buf(
        &mut reader,
        &tobj::LoadOptions {
            triangulate: true,
            single_index: true,
            ..Default::default()
        },
        |_| Ok(Default::default()),
    )
    .map_err(|e| LoadError::LoadFailed(format!("Failed to load OBJ from bytes: {}", e)))?;

    let materials = materials.unwrap_or_default();

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

        let color = if let Some(material_id) = mesh.material_id {
            if let Some(material) = materials.get(material_id) {
                if let Some(diffuse) = material.diffuse {
                    Vec3::new(diffuse[0], diffuse[1], diffuse[2])
                } else {
                    Vec3::ONE
                }
            } else {
                Vec3::ONE
            }
        } else {
            Vec3::ONE
        };

        let colors = vec![color; positions.len()];

        meshes.push(MeshData {
            positions,
            normals,
            uvs,
            colors,
            indices: mesh.indices.clone(),
            texture: None,
        });
    }

    if meshes.is_empty() {
        return Err(LoadError::LoadFailed("No meshes found in OBJ data".into()));
    }

    Ok(meshes)
}

fn load_gltf_from_bytes(bytes: &[u8]) -> Result<Vec<MeshData>, LoadError> {
    let (document, buffers, images) = gltf::import_slice(bytes)
        .map_err(|e| LoadError::LoadFailed(format!("Failed to load GLTF from bytes: {}", e)))?;

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
                .map(|iter| iter.into_f32().map(|uv| Vec2::from_array(uv)).collect())
                .unwrap_or_else(|| vec![Vec2::ZERO; positions.len()]);

            let indices: Vec<u32> = reader
                .read_indices()
                .ok_or_else(|| LoadError::LoadFailed("Missing indices in GLTF mesh".into()))?
                .into_u32()
                .collect();

            let colors = vec![Vec3::ONE; positions.len()];

            let texture = primitive.material()
                .pbr_metallic_roughness()
                .base_color_texture()
                .and_then(|info| {
                    let image_index = info.texture().source().index();
                    images.get(image_index).map(|img_data| {
                        let width = img_data.width;
                        let height = img_data.height;
                        let data = img_data.pixels.clone();
                        let format = match img_data.format {
                            gltf::image::Format::R8G8B8A8 => crate::assets::TextureFormat::Rgba8,
                            gltf::image::Format::R8G8B8 => crate::assets::TextureFormat::Rgb8,
                            gltf::image::Format::R8 => crate::assets::TextureFormat::R8,
                            _ => crate::assets::TextureFormat::Rgba8,
                        };
                        std::sync::Arc::new(crate::assets::TextureData {
                            width,
                            height,
                            data,
                            format,
                        })
                    })
                });

            meshes.push(MeshData {
                positions,
                normals,
                uvs,
                colors,
                indices,
                texture,
            });
        }
    }

    if meshes.is_empty() {
        return Err(LoadError::LoadFailed("No meshes found in GLTF data".into()));
    }

    Ok(meshes)
}

#[derive(Debug, Clone, Copy)]
pub enum MeshFormat {
    Obj,
    Gltf,
}
