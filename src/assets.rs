use crate::collision::Aabb;
use glam::Vec3;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Deserialize)]
pub struct AssetDefinition {
    pub model: String,
    pub collision: String,
    #[serde(default = "default_scale")]
    pub scale: [f32; 3],
}

fn default_scale() -> [f32; 3] { [1.0, 1.0, 1.0] }

#[derive(Debug, Deserialize)]
struct AssetFile {
    version: u32,
    assets: HashMap<String, AssetDefinition>,
}

pub struct AssetManager {
    definitions: HashMap<String, AssetDefinition>,
}

impl AssetManager {
    pub fn load() -> Self {
        let path = path("assets/assets.toml");
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read asset manifest {}: {e}", path.display()));
        let file: AssetFile = toml::from_str(&text)
            .unwrap_or_else(|e| panic!("Failed to parse asset manifest {}: {e}", path.display()));
        if file.version != 1 {
            panic!("Unsupported asset manifest version {}", file.version);
        }
        log::info!("Loaded {} asset definition(s) from {}", file.assets.len(), path.display());
        Self { definitions: file.assets }
    }

    pub fn get(&self, id: &str) -> &AssetDefinition {
        self.definitions.get(id).unwrap_or_else(|| panic!("Unknown asset '{id}'"))
    }

    pub fn model_path(&self, id: &str) -> PathBuf { path(&self.get(id).model) }
    pub fn collision_path(&self, id: &str) -> PathBuf { path(&self.get(id).collision) }
    pub fn default_scale(&self, id: &str) -> Vec3 { self.get(id).scale.into() }

    pub fn collision_aabb(&self, id: &str) -> Aabb {
        let collision_path = self.collision_path(id);
        let (document, buffers, _) = gltf::import(&collision_path)
            .unwrap_or_else(|e| panic!("Failed to load collision asset {}: {e}", collision_path.display()));
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        let mut found = false;
        for scene in document.scenes() {
            for node in scene.nodes() {
                collect_bounds(node, glam::Mat4::IDENTITY, &buffers, &mut min, &mut max, &mut found);
            }
        }
        if !found {
            panic!("Collision asset {} contains no mesh positions", collision_path.display());
        }
        Aabb::from_min_max(min, max)
    }

    pub fn contains(&self, id: &str) -> bool { self.definitions.contains_key(id) }
}

fn collect_bounds(
    node: gltf::Node,
    parent: glam::Mat4,
    buffers: &[gltf::buffer::Data],
    min: &mut Vec3,
    max: &mut Vec3,
    found: &mut bool,
) {
    let transform = parent * glam::Mat4::from_cols_array_2d(&node.transform().matrix());
    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            if let Some(positions) = reader.read_positions() {
                for p in positions {
                    let v = transform * glam::Vec4::new(p[0], p[1], p[2], 1.0);
                    *min = min.min(Vec3::new(v.x, v.y, v.z));
                    *max = max.max(Vec3::new(v.x, v.y, v.z));
                    *found = true;
                }
            }
        }
    }
    for child in node.children() {
        collect_bounds(child, transform, buffers, min, max, found);
    }
}

/// Resolve a runtime asset path. Development assets live under `data/`.
pub fn path(relative: impl AsRef<Path>) -> PathBuf {
    let relative = relative.as_ref();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(exe_dir) = exe.parent() {
            let packaged = exe_dir.join("assets").join(relative);
            if packaged.exists() { return packaged; }
        }
    }
    let development = PathBuf::from("data").join(relative);
    if development.exists() { return development; }
    PathBuf::from("assets").join(relative)
}
