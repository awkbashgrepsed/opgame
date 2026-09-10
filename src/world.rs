use crate::assets::AssetManager;
use crate::collision::Aabb;
use crate::entity::GameObject;
use glam::Vec3;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use uuid::Uuid;

#[derive(Clone, Debug)]
pub struct Road {
    pub start: Vec3,
    pub end: Vec3,
    pub width: f32,
}

pub struct World {
    pub objects: HashMap<Uuid, GameObject>,
    map_collider: Option<Aabb>,
    pub roads: Vec<Road>,
    pub width: f32,
    pub height: f32,
    pub time_of_day: f32,
    pub weather: Weather,
}

#[derive(Clone, Copy, Debug)]
pub enum Weather {
    Clear,
    Rainy,
    Foggy,
    Stormy,
}

#[derive(Debug, Deserialize)]
struct MapFile {
    version: u32,
    map_asset: String,
    locations: Vec<MapLocation>,
}

#[derive(Debug, Deserialize)]
struct MapLocation {
    id: String,
    config: String,
    position: [f32; 3],
}

#[derive(Debug, Deserialize)]
struct LocationFile {
    version: u32,
    #[allow(dead_code)]
    name: String,
    objects: String,
}

#[derive(Debug, Deserialize)]
struct LocationObjects {
    version: u32,
    objects: Vec<ObjectDefinition>,
}

#[derive(Debug, Deserialize)]
struct ObjectDefinition {
    asset: String,
    position: [f32; 3],
    rotation: [f32; 3],
    scale: [f32; 3],
}

impl World {
    pub fn new(asset_manager: &AssetManager) -> Self {
        let mut world = Self {
            objects: HashMap::new(),
            map_collider: None,
            roads: Vec::new(),
            width: 1000.0,
            height: 1000.0,
            time_of_day: 12.0,
            weather: Weather::Clear,
        };

        world.load_map("main", asset_manager);
        world
    }

    fn load_map(&mut self, map_name: &str, asset_manager: &AssetManager) {
        let path = crate::assets::path(format!("maps/{map_name}.toml"));
        let text = fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("Failed to read map {}: {e}", path.display()));
        let file: MapFile = toml::from_str(&text)
            .unwrap_or_else(|e| panic!("Failed to parse map {}: {e}", path.display()));

        if file.version != 1 {
            panic!("Unsupported map version {}", file.version);
        }

        if !asset_manager.contains(&file.map_asset) {
            panic!("Map references unknown asset '{}'", file.map_asset);
        }

        // Keep the render model separate from the object list. The renderer
        // already draws the map asset directly; physics gets its collision
        // from the same data-driven asset definition.
        let map_scale = asset_manager.default_scale(&file.map_asset);
        let local = asset_manager.collision_aabb(&file.map_asset);
        self.map_collider = Some(Aabb::new(
            local.center * map_scale,
            local.half_extents * map_scale.abs(),
        ));

        let location_count = file.locations.len();
        for location in file.locations {
            self.load_location(
                &location.id,
                &location.config,
                Vec3::from(location.position),
                asset_manager,
            );
        }

        log::info!(
            "Loaded map '{}' with {} location(s) and map asset '{}'",
            map_name,
            location_count,
            file.map_asset
        );
    }

    fn load_location(
        &mut self,
        location: &str,
        config_path: &str,
        location_position: Vec3,
        asset_manager: &AssetManager,
    ) {
        let path = crate::assets::path(config_path);
        let text = fs::read_to_string(&path).unwrap_or_else(|e| {
            panic!("Failed to read location config {}: {e}", path.display())
        });
        let file: LocationFile = toml::from_str(&text).unwrap_or_else(|e| {
            panic!("Failed to parse location config {}: {e}", path.display())
        });

        if file.version != 1 {
            panic!("Unsupported location version {}", file.version);
        }

        let objects_path = path
            .parent()
            .unwrap_or_else(|| panic!("Location config has no parent directory: {}", path.display()))
            .join(&file.objects);
        let objects_text = fs::read_to_string(&objects_path).unwrap_or_else(|e| {
            panic!("Failed to read location objects {}: {e}", objects_path.display())
        });
        let objects: LocationObjects = toml::from_str(&objects_text).unwrap_or_else(|e| {
            panic!("Failed to parse location objects {}: {e}", objects_path.display())
        });

        if objects.version != 1 {
            panic!("Unsupported location objects version {}", objects.version);
        }

        let count = objects.objects.len();
        for definition in objects.objects {
            if !asset_manager.contains(&definition.asset) {
                panic!("Location references unknown asset '{}'", definition.asset);
            }

            let position = location_position + Vec3::from(definition.position);
            let rotation = Vec3::from(definition.rotation);
            let scale = Vec3::from(definition.scale) * asset_manager.default_scale(&definition.asset);
            let collider = asset_manager.collision_aabb(&definition.asset);
            let object = GameObject::from_asset(
                &definition.asset,
                position,
                rotation,
                scale,
                collider,
            );
            self.objects.insert(object.id, object);
        }

        log::info!("Loaded location '{}' with {} object(s)", location, count);
    }

    pub fn add_road(&mut self, start: Vec3, end: Vec3, width: f32) {
        self.roads.push(Road { start, end, width });
    }

    pub fn remove_object(&mut self, id: Uuid) {
        self.objects.remove(&id);
    }

    pub fn collision_boxes(&self) -> impl Iterator<Item = Aabb> + '_ {
        self.map_collider
            .into_iter()
            .chain(self.objects.values().filter_map(|object| object.collision_aabb()))
    }

    pub fn update_time(&mut self, delta: f32) {
        self.time_of_day += delta * 0.001;
        if self.time_of_day >= 24.0 {
            self.time_of_day = 0.0;
        }
    }

    pub fn set_weather(&mut self, weather: Weather) {
        self.weather = weather;
    }

    pub fn is_night(&self) -> bool {
        self.time_of_day < 6.0 || self.time_of_day >= 18.0
    }

    pub fn is_day(&self) -> bool {
        !self.is_night()
    }
}
