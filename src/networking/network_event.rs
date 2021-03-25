//! For our networking model, we need to be able to send events
//! these events are registered in the [`AppBuilder`] with
//! [`NetworkAppBuilderExt`].
//!
//! These events are serialized with [`bincode`] and [`serde`]
//! and sent through the network. When they arrive, they're
//! deserialized and sent to the [`NetworkEvents`] registered.

use super::*;
use bevy::{prelude::*, reflect::TypeUuid};
use serde::de::DeserializeOwned;
use std::sync::{Arc, Mutex};

pub trait NetworkEvent: TypeUuid + Serialize + DeserializeOwned + 'static + Send + Sync {}

impl<T> NetworkEvent for T where T: TypeUuid + Serialize + DeserializeOwned + 'static + Send + Sync {}

pub fn network_event_system<T: NetworkEvent>(
    mut messages: ResMut<NetworkMessages>,
    mut network_events: ResMut<NetworkEvents<T>>,
) {
    network_events.clear();

    for message in messages.take_messages(&T::TYPE_UUID) {
        let event: T = bincode::deserialize(&message.payload.data).unwrap();

        network_events.push(message.sender, event);
    }
}

pub fn network_event_sender_system(sender: Res<NetworkEventSender>, net: Res<NetworkResource>) {
    for payload in sender.take() {
        net.send(&payload).unwrap();
    }
}

pub struct NetworkEvents<T> {
    events: Vec<(ActorId, T)>,
}

impl<T> NetworkEvents<T> {
    pub fn new() -> Self {
        Self { events: Vec::new() }
    }

    pub fn clear(&mut self) {
        self.events.clear();
    }

    pub fn push(&mut self, sender: ActorId, event: T) {
        self.events.push((sender, event));
    }

    pub fn take(&mut self) -> Vec<(ActorId, T)> {
        std::mem::replace(&mut self.events, Vec::new())
    }
}

pub struct NetworkEventSender {
    payloads: Arc<Mutex<Vec<NetworkPayload>>>,
}

impl NetworkEventSender {
    pub fn new() -> Self {
        Self {
            payloads: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn take(&self) -> Vec<NetworkPayload> {
        std::mem::replace(&mut *self.payloads.lock().unwrap(), Vec::new())
    }

    pub fn send<T: NetworkEvent>(&self, event: &T) -> anyhow::Result<()> {
        let data = bincode::serialize(event)?;

        let payload = NetworkPayload {
            uuid: T::TYPE_UUID,
            data,
        };

        let mut payloads = self.payloads.lock().unwrap();

        payloads.push(payload);

        Ok(())
    }
}
