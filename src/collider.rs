use crate::networking::*;
use bevy::prelude::*;
use geo::{algorithm::intersects::Intersects, LineString, Polygon};
use itertools::Itertools;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct Collider {
    polygon: Vec<Vec2>,
}

impl From<Vec<Vec2>> for Collider {
    fn from(vec: Vec<Vec2>) -> Self {
        Self {
            polygon: vec,
        }
    }
}

pub struct CollisionResource {
    pub just_intersected: HashMap<Entity, HashSet<Entity>>,
    pub intersecting: HashMap<Entity, HashSet<Entity>>,
}

impl CollisionResource {
    pub fn new() -> Self {
        Self {
            just_intersected: HashMap::new(),
            intersecting: HashMap::new(),
        }
    }

    pub fn add_intersection(&mut self, a: Entity, b: Entity) {
        self.intersecting
            .entry(a)
            .or_insert(HashSet::new())
            .insert(b);
        self.intersecting
            .entry(b)
            .or_insert(HashSet::new())
            .insert(a);
    }

    pub fn just_intersected(&self, entity: &Entity) -> HashSet<Entity> {
        match self.just_intersected.get(entity) {
            Some(v) => v.clone(),
            None => HashSet::new(),
        }
    }
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "caf696fe-d97e-4a71-a250-5a790bb2f525"]
pub enum Collision {
    Intersection { a: NetworkEntity, b: NetworkEntity },
}

pub fn collision_system(
    mut collision_resource: ResMut<CollisionResource>,
    query: Query<(Entity, &Collider, &GlobalTransform)>,
) {
    let prev = std::mem::replace(&mut collision_resource.intersecting, HashMap::new());

    for mut vec in query.iter().combinations(2) {
        let (a_entity, a_collider, a_transform) = vec.pop().unwrap();
        let (b_entity, b_collider, b_transform) = vec.pop().unwrap();

        if a_entity == b_entity {
            continue;
        }

        let a_points = a_collider.polygon.iter().map(|v| {
            let mut v = v.extend(0.0);
            v = a_transform.compute_matrix().transform_point3(v * 32.0);

            (v.x, v.y)
        }).collect::<Vec<_>>();

        let b_points = b_collider.polygon.iter().map(|v| {
            let mut v = v.extend(0.0);
            v = b_transform.compute_matrix().transform_point3(v * 32.0);

            (v.x, v.y)
        }).collect::<Vec<_>>();

        let mut a_lines = LineString::from(a_points);
        let mut b_lines = LineString::from(b_points);

        a_lines.close();
        b_lines.close();

        let a_polygon = Polygon::new(a_lines, Vec::new());
        let b_polygon = Polygon::new(b_lines, Vec::new());

        let intersects = a_polygon.intersects(&b_polygon);

        if intersects {
            info!("intersection: {:?}, {:?}", a_entity, b_entity);
            collision_resource.add_intersection(a_entity, b_entity);
        }
    }

    let CollisionResource {
        just_intersected,
        intersecting,
    } = &mut *collision_resource;
    just_intersected.clear();

    for (entity, intersections) in intersecting {
        if let Some(prev_intersections) = prev.get(entity) {
            for intersection in intersections.iter() {
                if !prev_intersections.contains(intersection) {
                    just_intersected
                        .entry(*entity)
                        .or_insert(HashSet::new())
                        .insert(*intersection);
                }
            }
        } else {
            just_intersected.insert(*entity, intersections.clone());
        }
    }
}

pub struct CollisionPlugin;

impl Plugin for CollisionPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let is_server = app_builder
            .world()
            .get_resource::<NetworkSettings>()
            .unwrap()
            .is_server;

        app_builder.register_network_event::<Collision>();

        if is_server {
            app_builder.insert_resource(CollisionResource::new());
            app_builder.add_system(collision_system.system());
            app_builder.add_event::<Collision>();
        }
    }
}
