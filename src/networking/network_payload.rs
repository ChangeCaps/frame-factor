use super::*;
use bevy::{prelude::*, reflect::Uuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// A payload sent through the network, supposed to be deserialized on arrival.
#[derive(Serialize, Deserialize)]
pub struct NetworkPayload {
    pub uuid: Uuid,
    pub data: Vec<u8>,
}

impl NetworkPayload {
    /// Creates a new [`NetworkPayload`] from a [`Serialize`] T and a [`Uuid`].
    /// The T is serialized with [`bincode`].
    pub fn new<T: Serialize>(payload: &T, uuid: Uuid) -> Self {
        Self {
            uuid,
            data: bincode::serialize(payload).unwrap(),
        }
    }
}

/// A [`NetworkPayload`] wrapped with the [`ActorId`] of the sender.
pub struct NetworkMessage {
    pub sender: ActorId,
    pub payload: NetworkPayload,
}

/// Contains the payloads last received.
pub struct NetworkMessages {
    messages: HashMap<Uuid, Vec<NetworkMessage>>,
}

impl NetworkMessages {
    /// Creates new [`NetworkMessages`].
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    /// Clears the contents.
    pub fn clear(&mut self) {
        self.messages.clear();
    }

    /// Adds the message to self.
    pub fn add(&mut self, message: NetworkMessage) {
        self.messages
            .entry(message.payload.uuid)
            .or_insert(Vec::new())
            .push(message);
    }

    /// Takes the payloads marked with the same uuid.
    pub fn take_messages(&mut self, uuid: &Uuid) -> Vec<NetworkMessage> {
        self.messages.remove(uuid).unwrap_or(Vec::new())
    }
}

pub fn network_receive_system(
    net: Res<NetworkResource>,
    mut messages: ResMut<NetworkMessages>,
    mut events: EventWriter<ConnectionEvent>,
) {
    messages.clear();

    let (m, c) = net.recv();
    
    for message in m {
        messages.add(message);
    }

    for event in c {
        events.send(event);
    }
}
