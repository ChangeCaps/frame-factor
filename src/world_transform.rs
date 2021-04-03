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
pub enum WorldTransformEvent {
    SetTranslation(NetworkEntity, Vec3),
    SetTargetTranslation(NetworkEntity, Vec3),
}

pub struct WorldTransform {
    pub translation: Vec3,
    pub rotation: f32,
    pub target_translation: Option<Vec3>,
}

impl WorldTransform {
    pub fn new(translation: Vec3) -> Self {
        Self {
            translation,
            rotation: 0.0,
            target_translation: None,
        }
    }
}

pub fn world_transform_system(
    network_settings: Res<NetworkSettings>,
    event_sender: Res<NetworkEventSender>,
    mut query: Query<
        (&mut Transform, &mut WorldTransform, &NetworkEntity),
        Changed<WorldTransform>,
    >,
) {
    for (mut transform, mut world_transform, network_entity) in query.iter_mut() {
        if let Some(target_translation) = world_transform.target_translation {
            if target_translation.distance(world_transform.translation) > 0.1 {
                world_transform.translation += target_translation;
                world_transform.translation /= 2.0;
            }
        }

        transform.translation = world_transform.translation * 32.0;
        transform.translation.z = world_transform.translation.y * -YZ_RATIO;

        if network_settings.is_server {
            event_sender
                .send(&WorldTransformEvent::SetTranslation(
                    *network_entity,
                    world_transform.translation,
                ))
                .unwrap();
        }
    }
}

pub fn world_transform_network_system(
    network_settings: Res<NetworkSettings>,
    mut events: ResMut<NetworkEvents<WorldTransformEvent>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut query: Query<&mut WorldTransform>,
) {
    if !network_settings.is_server {
        for (_sender, event) in events.take() {
            match event {
                WorldTransformEvent::SetTranslation(network_entity, translation) => {
                    if let Some(entity) = network_entity_registry.get(&network_entity) {
                        if let Ok(mut transform) = query.get_mut(entity) {
                            transform.translation = translation;
                            transform.target_translation = None;
                        }
                    }
                }
                WorldTransformEvent::SetTargetTranslation(network_entity, translation) => {
                    if let Some(entity) = network_entity_registry.get(&network_entity) {
                        if let Ok(mut transform) = query.get_mut(entity) {
                            transform.target_translation = Some(translation);
                        }
                    }
                }
            }
        }
    }
}
