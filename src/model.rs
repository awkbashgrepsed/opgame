use std::sync::OnceLock;

use glam::Vec3;

use crate::assets;
use crate::gl;

struct ModelMesh {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    color: [f32; 4],
}

static PLAYER_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();

fn load_player_model() -> &'static Vec<ModelMesh> {
    PLAYER_MODEL.get_or_init(|| {
        let path = assets::path("models/player.glb");
        if !path.is_file() {
            panic!("Player model not found: {}", path.display());
        }

        // The model is deliberately read from disk. Nothing from the GLB is
        // embedded into the executable; packaged builds keep it beside the game.
        let (document, buffers, _images) = gltf::import(&path)
            .unwrap_or_else(|e| panic!("Failed to load {}: {e}", path.display()));

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

                let positions: Vec<Vec3> = reader
                    .read_positions()
                    .unwrap_or_else(|| panic!("{} contains a primitive without positions", path.display()))
                    .map(|p| Vec3::new(p[0], p[1], p[2]))
                    .collect();

                let indices: Vec<u32> = match reader.read_indices() {
                    Some(indices) => indices.into_u32().collect(),
                    None => (0..positions.len() as u32).collect(),
                };

                let color = primitive
                    .material()
                    .pbr_metallic_roughness()
                    .base_color_factor();

                meshes.push(ModelMesh {
                    positions,
                    indices,
                    color,
                });
            }
        }

        if meshes.is_empty() {
            panic!("{} contains no mesh primitives", path.display());
        }

        log::info!("Loaded player model from {}: {} mesh primitive(s)", path.display(), meshes.len());
        meshes
    })
}

pub unsafe fn draw_player(position: Vec3, rotation: f32) {
    let meshes = load_player_model();

    // Keep the prototype player's approximate footprint while using the real model.
    const SCALE: f32 = 0.8;

    gl::PushMatrix();
    gl::Translatef(position.x, position.y, position.z);
    gl::Rotatef(rotation.to_degrees(), 0.0, 1.0, 0.0);
    gl::Scalef(SCALE, SCALE, SCALE);

    for mesh in meshes {
        gl::Color4f(mesh.color[0], mesh.color[1], mesh.color[2], mesh.color[3]);
        gl::Begin(gl::TRIANGLES);

        for &index in &mesh.indices {
            if let Some(p) = mesh.positions.get(index as usize) {
                gl::Vertex3f(p.x, p.y, p.z);
            }
        }

        gl::End();
    }

    gl::PopMatrix();
}
