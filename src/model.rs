use std::collections::HashMap;
use std::sync::OnceLock;

use glam::{Mat4, Vec2, Vec3, Vec4};

use crate::assets;
use crate::gl;

struct ModelMesh {
    positions: Vec<Vec3>,
    normals: Vec<Vec3>,
    uvs: Vec<Vec2>,
    indices: Vec<u32>,
    color: [f32; 4],
    texture: Option<u32>,
}

unsafe fn load_texture(image: &gltf::image::Data) -> u32 {
    let mut rgba = Vec::with_capacity((image.width * image.height * 4) as usize);
    match image.format {
        gltf::image::Format::R8 => {
            for &r in &image.pixels { rgba.extend_from_slice(&[r, r, r, 255]); }
        }
        gltf::image::Format::R8G8 => {
            for p in image.pixels.chunks_exact(2) { rgba.extend_from_slice(&[p[0], p[0], p[0], p[1]]); }
        }
        gltf::image::Format::R8G8B8 => {
            for p in image.pixels.chunks_exact(3) { rgba.extend_from_slice(&[p[0], p[1], p[2], 255]); }
        }
        gltf::image::Format::R8G8B8A8 => rgba.extend_from_slice(&image.pixels),
        _ => panic!("Unsupported GLB image format {:?}; export textures as 8-bit PNG/JPEG data", image.format),
    }

    let mut texture = 0;
    gl::GenTextures(1, &mut texture);
    gl::BindTexture(gl::TEXTURE_2D, texture);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MIN_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_MAG_FILTER, gl::LINEAR as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_S, gl::REPEAT as i32);
    gl::TexParameteri(gl::TEXTURE_2D, gl::TEXTURE_WRAP_T, gl::REPEAT as i32);
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
    gl::TexImage2D(gl::TEXTURE_2D, 0, gl::RGBA as i32, image.width as i32, image.height as i32, 0, gl::RGBA, gl::UNSIGNED_BYTE, rgba.as_ptr() as *const std::ffi::c_void);
    gl::BindTexture(gl::TEXTURE_2D, 0);
    texture
}

unsafe fn load_model(relative_path: &str, label: &str) -> Vec<ModelMesh> {
    let path = assets::path(relative_path);
    if !path.is_file() { panic!("{label} model not found: {}", path.display()); }
    let (document, buffers, images) = gltf::import(&path).unwrap_or_else(|e| panic!("Failed to load {}: {e}", path.display()));
    let mut meshes = Vec::new();
    let mut texture_cache: HashMap<usize, u32> = HashMap::new();

    for scene in document.scenes() {
        for node in scene.nodes() {
            load_node(node, Mat4::IDENTITY, &buffers, &images, &mut texture_cache, &mut meshes, &path);
        }
    }
    if meshes.is_empty() { panic!("{} contains no mesh primitives", path.display()); }
    log::info!("Loaded {label} from {}: {} mesh primitive(s), {} texture(s)", path.display(), meshes.len(), texture_cache.len());
    meshes
}

unsafe fn load_node(node: gltf::Node, parent_transform: Mat4, buffers: &[gltf::buffer::Data], images: &[gltf::image::Data], texture_cache: &mut HashMap<usize, u32>, meshes: &mut Vec<ModelMesh>, path: &std::path::Path) {
    let transform = parent_transform * Mat4::from_cols_array_2d(&node.transform().matrix());
    let normal_matrix = transform.inverse().transpose();

    if let Some(mesh) = node.mesh() {
        for primitive in mesh.primitives() {
            let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()].0));
            let positions: Vec<Vec3> = reader.read_positions().unwrap_or_else(|| panic!("{} contains a primitive without positions", path.display())).map(|p| {
                let v = transform * Vec4::new(p[0], p[1], p[2], 1.0);
                Vec3::new(v.x, v.y, v.z)
            }).collect();
            let normals: Vec<Vec3> = match reader.read_normals() {
                Some(values) => values.map(|n| {
                    let v = normal_matrix * Vec4::new(n[0], n[1], n[2], 0.0);
                    Vec3::new(v.x, v.y, v.z).normalize_or_zero()
                }).collect(),
                None => vec![Vec3::Y; positions.len()],
            };
            let uvs: Vec<Vec2> = match reader.read_tex_coords(0) {
                Some(values) => values.into_f32().map(|uv| Vec2::new(uv[0], uv[1])).collect(),
                None => vec![Vec2::ZERO; positions.len()],
            };
            let indices: Vec<u32> = match reader.read_indices() {
                Some(indices) => indices.into_u32().collect(),
                None => (0..positions.len() as u32).collect(),
            };
            let pbr = primitive.material().pbr_metallic_roughness();
            let color = pbr.base_color_factor();
            let texture = pbr.base_color_texture().map(|info| {
                let image_index = info.texture().source().index();
                if let Some(&texture) = texture_cache.get(&image_index) { texture } else {
                    let texture = load_texture(&images[image_index]);
                    texture_cache.insert(image_index, texture);
                    texture
                }
            });
            meshes.push(ModelMesh { positions, normals, uvs, indices, color, texture });
        }
    }
    for child in node.children() { load_node(child, transform, buffers, images, texture_cache, meshes, path); }
}

static PLAYER_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();
static MAP_MODEL: OnceLock<Vec<ModelMesh>> = OnceLock::new();

fn load_player_model() -> &'static Vec<ModelMesh> { PLAYER_MODEL.get_or_init(|| unsafe { load_model("models/player.glb", "player") }) }
fn load_map_model() -> &'static Vec<ModelMesh> { MAP_MODEL.get_or_init(|| unsafe { load_model("models/environment/map.glb", "map") }) }

unsafe fn draw_meshes(meshes: &[ModelMesh]) {
    gl::Enable(gl::TEXTURE_2D);
    for mesh in meshes {
        gl::Color4f(mesh.color[0], mesh.color[1], mesh.color[2], mesh.color[3]);
        gl::BindTexture(gl::TEXTURE_2D, mesh.texture.unwrap_or(0));
        gl::Begin(gl::TRIANGLES);
        for &index in &mesh.indices {
            if let Some(p) = mesh.positions.get(index as usize) {
                if let Some(n) = mesh.normals.get(index as usize) { gl::Normal3f(n.x, n.y, n.z); }
                if let Some(uv) = mesh.uvs.get(index as usize) { gl::TexCoord2f(uv.x, uv.y); }
                gl::Vertex3f(p.x, p.y, p.z);
            }
        }
        gl::End();
    }
    gl::BindTexture(gl::TEXTURE_2D, 0);
    gl::Disable(gl::TEXTURE_2D);
}

pub unsafe fn draw_player(position: Vec3, rotation: f32) {
    let meshes = load_player_model();
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
    gl::PushMatrix();
    draw_meshes(meshes);
    gl::PopMatrix();
}
