use glam::Vec3;

#[derive(Clone, Copy, Debug)]
pub struct Aabb {
    pub center: Vec3,
    pub half_extents: Vec3,
}

impl Aabb {
    pub fn new(center: Vec3, half_extents: Vec3) -> Self {
        Self { center, half_extents }
    }

    pub fn min(&self) -> Vec3 {
        self.center - self.half_extents
    }

    pub fn max(&self) -> Vec3 {
        self.center + self.half_extents
    }

    pub fn intersects(&self, other: &Aabb) -> bool {
        let a_min = self.min();
        let a_max = self.max();
        let b_min = other.min();
        let b_max = other.max();

        a_min.x < b_max.x
            && a_max.x > b_min.x
            && a_min.y < b_max.y
            && a_max.y > b_min.y
            && a_min.z < b_max.z
            && a_max.z > b_min.z
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CollisionHit {
    pub normal: Vec3,
    pub depth: f32,
}

pub fn collision_between(a: &Aabb, b: &Aabb) -> Option<CollisionHit> {
    if !a.intersects(b) {
        return None;
    }

    let delta = b.center - a.center;
    let overlap_x = a.half_extents.x + b.half_extents.x - delta.x.abs();
    let overlap_y = a.half_extents.y + b.half_extents.y - delta.y.abs();
    let overlap_z = a.half_extents.z + b.half_extents.z - delta.z.abs();

    if overlap_x <= overlap_y && overlap_x <= overlap_z {
        Some(CollisionHit {
            normal: Vec3::new(if delta.x >= 0.0 { -1.0 } else { 1.0 }, 0.0, 0.0),
            depth: overlap_x,
        })
    } else if overlap_y <= overlap_z {
        Some(CollisionHit {
            normal: Vec3::new(0.0, if delta.y >= 0.0 { -1.0 } else { 1.0 }, 0.0),
            depth: overlap_y,
        })
    } else {
        Some(CollisionHit {
            normal: Vec3::new(0.0, 0.0, if delta.z >= 0.0 { -1.0 } else { 1.0 }),
            depth: overlap_z,
        })
    }
}

pub fn move_with_collisions<F>(
    position: Vec3,
    half_extents: Vec3,
    movement: Vec3,
    mut collider_at: F,
) -> Vec3
where
    F: FnMut(Vec3) -> Option<Aabb>,
{
    // Resolve each axis independently. This gives natural wall sliding without
    // requiring a full rigid-body solver.
    let mut result = position;

    for (axis, amount) in [(0usize, movement.x), (2usize, movement.z), (1usize, movement.y)] {
        if amount.abs() <= f32::EPSILON {
            continue;
        }

        let mut candidate = result;
        candidate[axis] += amount;
        let player_box = Aabb::new(candidate, half_extents);

        if let Some(world_box) = collider_at(candidate) {
            if let Some(hit) = collision_between(&player_box, &world_box) {
                candidate += hit.normal * hit.depth;
            }
        }

        result = candidate;
    }

    result
}
