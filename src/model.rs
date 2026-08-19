use std::sync::OnceLock;

use glam::Vec3;

struct ModelMesh {
    positions: Vec<Vec3>,
    indices: Vec<u32>,
    color: [f32; 4],
}

static PLAYER_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();

fn load_player_model() -> &'static Vec<ModelMesh> {
    PLAYER_MODEL.get_or_init(|| {
        let bytes = include_bytes!("../assets/player.glb");
        let (document, buffers, _images) = gltf::import_slice(bytes)
            .unwrap_or_else(|e| panic!("Failed to load assets/player.glb: {e}"));

        let mut meshes = Vec::new();

        for mesh in document.meshes() {
            for primitive in mesh.primitives() {
                let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));

                let positions: Vec<Vec3> = reader
                    .read_positions()
                    .unwrap_or_else(|| panic!("assets/player.glb contains a primitive without positions"))
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
            panic!("assets/player.glb contains no mesh primitives");
        }

        log::info!("Loaded player.glb: {} mesh primitive(s)", meshes.len());
        meshes
    })
}

pub unsafe fn draw_player(position: Vec3, rotation: f32) {
    let meshes = load_player_model();

    // Blender's default cube is 2 units across. Keep the first placeholder
    // player's approximate footprint while using the real model.
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
