use super::*;
use bevy::{prelude::*, reflect::Uuid};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct NetworkPayload {
    pub uuid: Uuid,
    pub data: Vec<u8>,
}

pub struct NetworkMessage {
    pub sender: ActorId,
    pub payload: NetworkPayload,
}

/// Contains the payloads last received.
pub struct NetworkMessages {
    messages: HashMap<Uuid, Vec<NetworkMessage>>,
}

impl NetworkMessages {
    pub fn new() -> Self {
        Self {
            messages: HashMap::new(),
        }
    }

    pub fn clear(&mut self) {
        self.messages.clear();
    }

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
    mut net: ResMut<NetworkResource>,
    mut messages: ResMut<NetworkMessages>,
) {
    messages.clear();

    for message in net.recv().unwrap() {
        messages.add(message);
    }
}
