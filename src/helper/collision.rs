use bevy::prelude::*;
use heron::prelude::*;

pub fn convex_hull(point: Vec<Vec2>) -> Body {
    let points = point.into_iter().map(|v| v.extend(0.0)).collect::<Vec<_>>();

    Body::ConvexHull { points }
}