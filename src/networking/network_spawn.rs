//! For multiple actors to be able to refer to the same entities
//! on multiple machines, they have to agree on a way to identify
//! them. For various reasons, it's impractical for that to be
//! their [`Entity`]. In stead, we use a [`NetworkEntity`].
//! The [`NetworkEntity`] is both stored as a component in the ecs
//! and in a [`NetworkEntityRegistry`], for quick lookups.

use super::*;
use bevy::reflect::Uuid;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

#[derive(Clone, Copy, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub struct NetworkEntity(u64);

pub struct NetworkEntityRegistry {
    entities: HashMap<NetworkEntity, Entity>,
    next_network_entity: NetworkEntity,
}

impl NetworkEntityRegistry {
    pub fn new() -> Self {
        Self {
            entities: HashMap::new(),
            next_network_entity: NetworkEntity(0),
        }
    }

    pub fn get(&self, network_entity: &NetworkEntity) -> Option<Entity> {
        self.entities.get(network_entity).cloned()
    }

    pub fn generate(&mut self) -> NetworkEntity {
        let entity = self.next_network_entity;
        self.next_network_entity.0 += 1;
        entity
    }

    pub fn insert(&mut self, network_entity: NetworkEntity, entity: Entity) {
        self.entities.insert(network_entity, entity);
    }

    pub fn add(&mut self, entity: Entity) -> NetworkEntity {
        let network_entity = self.generate();
        self.insert(network_entity, entity);
        network_entity
    }
}

#[derive(Serialize, Deserialize)]
pub struct NetworkSpawnEvent<T> {
    network_entity: NetworkEntity,
    spawnable: T,
}

pub trait NetworkSpawnable:
    Serialize + serde::de::DeserializeOwned + TypeUuid + 'static + Send + Sync
{
    fn spawn(&self, world: &mut World) -> Entity;
}

fn spawn<T: NetworkSpawnable>(world: &mut World, payload: &NetworkPayload) {
    let event: NetworkSpawnEvent<T> = bincode::deserialize(&payload.data).unwrap();

    let entity = event.spawnable.spawn(world);

    world.entity_mut(entity).insert(event.network_entity);

    world
        .get_resource_mut::<NetworkEntityRegistry>()
        .unwrap()
        .insert(event.network_entity, entity);
}

pub struct NetworkSpawner {
    spawnables: HashMap<Uuid, Box<dyn Fn(&mut World, &NetworkPayload) + 'static + Send + Sync>>,
    spawns: Arc<Mutex<Vec<Box<dyn FnOnce(&mut World) -> NetworkPayload + 'static + Send + Sync>>>>,
}

impl NetworkSpawner {
    pub fn new() -> Self {
        Self {
            spawnables: HashMap::new(),
            spawns: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn register_spawnable<T: NetworkSpawnable>(&mut self) {
        self.spawnables.insert(T::TYPE_UUID, Box::new(spawn::<T>));
    }

    pub fn spawn_payload(&self, world: &mut World, payload: &NetworkPayload) {
        if let Some(spawner) = self.spawnables.get(&payload.uuid) {
            spawner(world, payload);
        }
    }

    pub fn spawn<T: NetworkSpawnable>(&self, spawnable: T) {
        let mut spawns = self.spawns.lock().unwrap();

        spawns.push(Box::new(move |world| {
            let entity = spawnable.spawn(world);

            let mut network_entity_registry =
                world.get_resource_mut::<NetworkEntityRegistry>().unwrap();
            let network_entity = network_entity_registry.add(entity);

            world.entity_mut(entity).insert(network_entity);

            let event = NetworkSpawnEvent {
                network_entity,
                spawnable,
            };

            NetworkPayload::new(&event, T::TYPE_UUID)
        }));
    }

    pub fn take_spawns(&self, world: &mut World) -> Vec<NetworkPayload> {
        self.spawns
            .lock()
            .unwrap()
            .drain(..)
            .map(|spawn| spawn(world))
            .collect()
    }
}

pub fn network_spawner_system(world: &mut World) {
    world.resource_scope(|mut messages: Mut<NetworkMessages>, world| {
        world.resource_scope(|network_spawner: Mut<NetworkSpawner>, world| {
            for payload in network_spawner.take_spawns(world) {
                let net = world.get_resource::<NetworkResource>().unwrap();

                net.send(&payload).unwrap();

                network_spawner.spawn_payload(world, &payload);
            }

            for uuid in network_spawner.spawnables.keys() {
                for message in messages.take_messages(uuid) {
                    network_spawner.spawn_payload(world, &message.payload);
                }
            }
        });
    });
}
