use bevy::prelude::*;
use bevy_rapier2d::rapier::{geometry::ColliderBuilder, math::Point};

pub fn polygon_collider(polygon: Vec<Vec2>) -> ColliderBuilder {
    let verts = polygon
        .into_iter()
        .map(|v| Point::new(v.x, v.y))
        .collect::<Vec<_>>();
    let indices = (0..verts.len())
        .into_iter()
        .map(|i| [i as u32, ((i + 1) % verts.len()) as u32])
        .collect::<Vec<_>>();

    //ColliderBuilder::polyline(verts, Some(indices))
    ColliderBuilder::cuboid(50.0, 50.0)
}
