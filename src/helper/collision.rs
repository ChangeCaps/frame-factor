use bevy_rapier2d::rapier::{geometry::ColliderBuilder, math::Point};
use bevy::prelude::*;

pub fn polygon_collider(polygon: Vec<Vec2>) -> ColliderBuilder {
    let verts = polygon.into_iter().map(|v| Point::new(v.x, v.y)).collect::<Vec<_>>();
    
    ColliderBuilder::polyline(verts, None)
}