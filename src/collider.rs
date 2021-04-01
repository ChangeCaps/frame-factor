use crate::networking::*;
use crate::world_transform::*;
use bevy::prelude::*;
use geo::{algorithm::intersects::Intersects, MultiPoint, Point};
use itertools::Itertools;

#[derive(Clone)]
pub struct Collider {
    polygon: MultiPoint<f32>,
}

impl From<Vec<Vec2>> for Collider {
    fn from(vec: Vec<Vec2>) -> Self {
        Self {
            polygon: MultiPoint::from(
                vec.into_iter()
                    .map(|v| Point::new(v.x, v.y))
                    .collect::<Vec<_>>(),
            ),
        }
    }
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "caf696fe-d97e-4a71-a250-5a790bb2f525"]
pub enum Collision {
    Intersection { a: NetworkEntity, b: NetworkEntity },
}

pub fn collision_system(
    mut event_writer: EventWriter<Collision>,
    event_sender: Res<NetworkEventSender>,
    query: Query<(&NetworkEntity, &Collider, &WorldTransform)>,
) {
    for mut vec in query.iter().combinations(2) {
        let (a_entity, a_collider, a_transform) = vec.pop().unwrap();
        let (b_entity, b_collider, b_transform) = vec.pop().unwrap();

        if a_entity == b_entity {
            continue;
        }

        let mut a_collider = a_collider.clone();
        let mut b_collider = b_collider.clone();

        a_collider.polygon.iter_mut().for_each(|p| {
            let mut v = Vec2::new(p.x(), p.y()).extend(0.0);
            v = a_transform.transform_point(v);

            p.set_x(v.x);
            p.set_y(v.y);
        });

        b_collider.polygon.iter_mut().for_each(|p| {
            let mut v = Vec2::new(p.x(), p.y()).extend(0.0);
            v = b_transform.transform_point(v);

            p.set_x(v.x);
            p.set_y(v.y);
        });

        let intersects = a_collider.polygon.intersects(&b_collider.polygon);

        if intersects {
            let event = Collision::Intersection {
                a: *a_entity,
                b: *b_entity,
            };

            event_sender.send(&event).unwrap();

            event_writer.send(event);
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
            app_builder.add_system(collision_system.system());
            app_builder.add_event::<Collision>();
        }
    }
}
