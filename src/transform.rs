//! [`WorldTransform`] serves two purposes, the first being that
//! we need a transform separate from the [`Transform`]. This is
//! because we want to have 'heigh'. The higher an object is, the
//! further along the y axis it goes. These kinds of transformations
//! are possible in bevy, but not very easily, and would create problems
//! concerning draw order. It also serves the purpose of smoothening the
//! position difference on the client side, for when the tickrate of the
//! server is lower than the frame rate of the client.

use crate::networking::*;
use bevy::prelude::*;

const YZ_RATIO: f32 = 1.0 / 2048.0;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "d4acd5da-0fdd-412c-9c6b-96ed1bca3595"]
pub enum TransformEvent {
    SetTranslation(NetworkEntity, Vec2),
}

pub struct Height(pub f32);

pub fn transform_z_sort_system(
    mut query: Query<&mut Transform, Changed<Transform>>,
) {
    for mut transform in query.iter_mut() {
        transform.translation.z = transform.translation.y * -YZ_RATIO;
    }
}

pub fn transform_server_system(
    event_sender: Res<NetworkEventSender>,
    query: Query<(&NetworkEntity, &Transform), Changed<Transform>>,
) {
    for (network_entity, transform) in query.iter() {
        event_sender
            .send(&TransformEvent::SetTranslation(
                *network_entity,
                transform.translation.truncate(),
            ))
            .unwrap();
    }
}

pub fn transform_client_system(
    network_settings: Res<NetworkSettings>,
    mut events: ResMut<NetworkEvents<TransformEvent>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut Transform>,
) {
    if !network_settings.is_server {
        for (_sender, event) in events.take() {
            match event {
                TransformEvent::SetTranslation(network_entity, translation) => {
                    if let Some(entity) = network_entity_registry.get(&network_entity) {
                        if let Ok(mut transform) = query.get_mut(entity) {
                            transform.translation = translation.extend(transform.translation.z);
                        }
                    }
                }
            }
        }
    }
}
