use crate::collision::{collision_between, Aabb};
use crate::player::Player;
use crate::world::World;
use glam::Vec3;

const GRAVITY: f32 = 20.0;
const PLAYER_HALF_EXTENTS: Vec3 = Vec3::new(0.35, 0.9, 0.35);
const GROUND_Y: f32 = 0.5;

pub struct PhysicsEngine {
    gravity: f32,
}

impl PhysicsEngine {
    pub fn new() -> Self {
        Self { gravity: GRAVITY }
    }

    pub fn update(&mut self, player: &mut Player, world: &World, movement: Vec3, dt: f32) {
        // Gravity is owned by the physics system. Player.position is the
        // character's foot position, so the collision box is centered above it.
        if player.position.y > GROUND_Y || player.velocity.y > 0.0 {
            player.velocity.y -= self.gravity * dt;
        }

        let mut displacement = movement;
        displacement.y += player.velocity.y * dt;

        let mut position = player.position;
        position = self.resolve_axis(position, PLAYER_HALF_EXTENTS, displacement.x, 0, world);
        position = self.resolve_axis(position, PLAYER_HALF_EXTENTS, displacement.z, 2, world);

        // Vertical collision needs to tell us whether we landed on a surface.
        // A positive Y collision normal means the surface is underneath the
        // player, regardless of whether that surface is the ground, a crate,
        // a platform, or another world object.
        let (vertical_position, landed) = self.resolve_vertical(
            position,
            PLAYER_HALF_EXTENTS,
            displacement.y,
            world,
        );
        position = vertical_position;

        if position.y <= GROUND_Y {
            position.y = GROUND_Y;
            if player.velocity.y < 0.0 {
                player.velocity.y = 0.0;
            }
            player.is_falling = false;
            player.is_jumping = false;
        } else if landed {
            player.velocity.y = 0.0;
            player.is_falling = false;
            player.is_jumping = false;
        } else {
            player.is_falling = true;
        }

        position.x = position.x.clamp(-500.0, 500.0);
        position.z = position.z.clamp(-500.0, 500.0);
        player.position = position;
    }

    fn resolve_axis(
        &self,
        position: Vec3,
        half_extents: Vec3,
        amount: f32,
        axis: usize,
        world: &World,
    ) -> Vec3 {
        if amount.abs() <= f32::EPSILON {
            return position;
        }

        let mut candidate = position;
        candidate[axis] += amount;
        let mut player_box = self.player_aabb(candidate, half_extents);

        for world_box in world.collision_boxes() {
            if let Some(hit) = collision_between(&player_box, &world_box) {
                candidate += hit.normal * hit.depth;
                player_box = self.player_aabb(candidate, half_extents);
            }
        }

        candidate
    }

    fn resolve_vertical(
        &self,
        position: Vec3,
        half_extents: Vec3,
        amount: f32,
        world: &World,
    ) -> (Vec3, bool) {
        if amount.abs() <= f32::EPSILON {
            return (position, false);
        }

        let mut candidate = position;
        candidate.y += amount;
        let mut player_box = self.player_aabb(candidate, half_extents);
        let mut landed = false;

        for world_box in world.collision_boxes() {
            if let Some(hit) = collision_between(&player_box, &world_box) {
                // Only a downward movement into an upward-facing surface is
                // considered a landing. Hitting a wall or the underside of an
                // object must not make the player grounded.
                if amount < 0.0 && hit.normal.y > 0.5 {
                    landed = true;
                }

                candidate += hit.normal * hit.depth;
                player_box = self.player_aabb(candidate, half_extents);
            }
        }

        (candidate, landed)
    }

    fn player_aabb(&self, feet_position: Vec3, half_extents: Vec3) -> Aabb {
        Aabb::new(
            feet_position + Vec3::new(0.0, half_extents.y, 0.0),
            half_extents,
        )
    }

    pub fn raycast(&self, origin: Vec3, direction: Vec3, max_distance: f32, world: &World) -> Option<Vec3> {
        let direction = direction.try_normalize()?;
        let mut closest_distance: f32 = max_distance;
        let mut closest_hit = None;

        for object_box in world.collision_boxes() {
            if let Some(distance) = ray_aabb(origin, direction, &object_box, max_distance) {
                if distance < closest_distance {
                    closest_distance = distance;
                    closest_hit = Some(origin + direction * distance);
                }
            }
        }

        closest_hit
    }
}

fn ray_aabb(origin: Vec3, direction: Vec3, aabb: &Aabb, max_distance: f32) -> Option<f32> {
    let min = aabb.min();
    let max = aabb.max();
    let mut t_min: f32 = 0.0;
    let mut t_max: f32 = max_distance;

    for axis in 0..3 {
        if direction[axis].abs() < 1e-6 {
            if origin[axis] < min[axis] || origin[axis] > max[axis] {
                return None;
            }
            continue;
        }

        let inv_dir = 1.0 / direction[axis];
        let mut t1 = (min[axis] - origin[axis]) * inv_dir;
        let mut t2 = (max[axis] - origin[axis]) * inv_dir;
        if t1 > t2 {
            std::mem::swap(&mut t1, &mut t2);
        }

        t_min = t_min.max(t1);
        t_max = t_max.min(t2);
        if t_min > t_max {
            return None;
        }
    }

    Some(t_min)
}
