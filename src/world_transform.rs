use crate::networking::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "d4acd5da-0fdd-412c-9c6b-96ed1bca3595"]
pub enum WorldTransformEvent {
    SetTranslation(NetworkEntity, Vec3),
}

pub struct WorldTransform {
    pub translation: Vec3,
}

pub fn world_transform_system(
    network_settings: Res<NetworkSettings>,
    event_sender: Res<NetworkEventSender>,
    mut query: Query<(&mut Transform, &WorldTransform, &NetworkEntity), Changed<WorldTransform>>,
) {
    for (mut transform, world_transform, network_entity) in query.iter_mut() {
        transform.translation = world_transform.translation;

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
                        }
                    }
                }
            }
        }
    }
}
