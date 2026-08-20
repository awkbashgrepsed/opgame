use std::sync::OnceLock;

use glam::Vec3;

use crate::assets;
use crate::gl;

struct ModelMesh {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    indices: Vec<u32>,
    color: [f32; 4],
}

fn load_model(relative_path: &str, label: &str) -> Vec<ModelMesh> {
    let path = assets::path(relative_path);
    if !path.is_file() {
        panic!("{label} model not found: {}", path.display());
    }

    // Models are deliberately read from disk. Nothing from the GLB is embedded
    // into the executable; packaged builds keep it beside the game.
    let (document, buffers, _images) = gltf::import(&path)
        .unwrap_or_else(|e| panic!("Failed to load {}: {e}", path.display()));

    let mut meshes = Vec::new();

    for mesh in document.meshes() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

            let positions: Vec<Vec3> = reader
                .read_positions()
                .unwrap_or_else(|| {
                    panic!(
                        "{} contains a primitive without positions",
                        path.display()
                    )
                })
                .map(|p| Vec3::new(p[0], p[1], p[2]))
                .collect();

            // GLTF normals are already vertex normals. Keeping one normal per
            // vertex gives OpenGL the data it needs for smooth/Gouraud shading.
            // If a primitive has no normals, generate a simple averaged normal
            // from its triangles so untextured prototype geometry still shades.
            let normals = if let Some(read_normals) = reader.read_normals() {
                read_normals
                    .map(|n| Vec3::new(n[0], n[1], n[2]).normalize_or_zero())
                    .collect()
            } else {
                vec![Vec3::ZERO; positions.len()]
            };

            let indices: Vec<u32> = match reader.read_indices() {
                Some(indices) => indices.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };

            let normals = if normals.iter().any(|n| *n != Vec3::ZERO) {
                normals
            } else {
                generate_vertex_normals(&positions, &indices)
            };

            let color = primitive
                .material()
                .pbr_metallic_roughness()
                .base_color_factor();

            meshes.push(ModelMesh {
                positions,
                normals,
                indices,
                color,
            });
        }
    }

    if meshes.is_empty() {
        panic!("{} contains no mesh primitives", path.display());
    }

    log::info!(
        "Loaded {label} from {}: {} mesh primitive(s)",
        path.display(),
        meshes.len()
    );
    meshes
}

fn generate_vertex_normals(positions: &[Vec3], indices: &[u32]) -> Vec<Vec3> {
    let mut normals = vec![Vec3::ZERO; positions.len()];

    for triangle in indices.chunks_exact(3) {
        let a = triangle[0] as usize;
        let b = triangle[1] as usize;
        let c = triangle[2] as usize;

        if a >= positions.len() || b >= positions.len() || c >= positions.len() {
            continue;
        }

        let edge_a = positions[b] - positions[a];
        let edge_b = positions[c] - positions[a];
        let face_normal = edge_a.cross(edge_b);

        // Accumulating unnormalized face normals gives larger triangles more
        // influence over the shared vertex normal, which is useful for smooth
        // Gouraud shading.
        normals[a] += face_normal;
        normals[b] += face_normal;
        normals[c] += face_normal;
    }

    for normal in &mut normals {
        *normal = normal.normalize_or_zero();
        if *normal == Vec3::ZERO {
            *normal = Vec3::Y;
        }
    }

    normals
}

static PLAYER_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();
static MAP_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();

fn load_player_model() -> &'static Vec<ModelMesh> {
    PLAYER_MODEL.get_or_init(|| load_model("models/player.glb", "player"))
}

fn load_map_model() -> &'static Vec<ModelMesh> {
    MAP_MODEL.get_or_init(|| load_model("models/environment/map.glb", "map"))
}

unsafe fn draw_meshes(meshes: &[ModelMesh]) {
    for mesh in meshes {
        gl::Color4f(
            mesh.color[0],
            mesh.color[1],
            mesh.color[2],
            mesh.color[3],
        );

        gl::Begin(gl::TRIANGLES);

        for &index in &mesh.indices {
            if let Some(p) = mesh.positions.get(index as usize) {
                let normal = mesh
                    .normals
                    .get(index as usize)
                    .copied()
                    .unwrap_or(Vec3::Y);

                // With GL_SMOOTH enabled, OpenGL interpolates these per-vertex
                // normals across each triangle: classic Gouraud shading.
                gl::Normal3f(normal.x, normal.y, normal.z);
                gl::Vertex3f(p.x, p.y, p.z);
            }
        }

        gl::End();
    }
}

pub unsafe fn draw_player(position: Vec3, rotation: f32) {
    let meshes = load_player_model();

    // Keep the prototype player's approximate footprint while using the real model.
    const SCALE: f32 = 0.8;

    gl::PushMatrix();
    gl::Translatef(position.x, position.y, position.z);
    gl::Rotatef(rotation.to_degrees(), 0.0, 1.0, 0.0);
    gl::Scalef(SCALE, SCALE, SCALE);
    draw_meshes(meshes);
    gl::PopMatrix();
}

pub unsafe fn draw_map() {
    let meshes = load_map_model();

    // The Blender scene is authored in world space. No artificial map scale is
    // applied here.
    gl::PushMatrix();
    draw_meshes(meshes);
    gl::PopMatrix();
}
