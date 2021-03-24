//! For our networking model, we need to be able to send events
//!

use bevy::{
    prelude::*,
    reflect::{TypeUuid, Uuid},
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

pub trait NetworkEvent: TypeUuid + Serialize + DeserializeOwned {}

pub fn network_event_system<T: NetworkEvent>(mut network_events: ResMut<NetworkEvents<T>>) {}

pub struct NetworkEvents<T> {
    events: Vec<T>,
}
