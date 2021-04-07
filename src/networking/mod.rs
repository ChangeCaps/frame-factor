//! Contains the networking framework.

pub mod network_event;
pub mod network_payload;
pub mod network_resource;
pub mod network_spawn;

pub use network_event::*;
pub use network_payload::*;
pub use network_resource::*;
pub use network_spawn::*;

pub use bevy::reflect::TypeUuid;
pub use serde::{Deserialize, Serialize};

use bevy::prelude::*;
use std::net::TcpStream;

#[derive(Debug, Hash, PartialEq, Eq, Clone, StageLabel)]
pub enum NetworkStage {
    Spawn,
}

pub struct NetworkSettings {
    pub is_server: bool,
}

pub struct NetworkPlugin {
    client_payload: Option<NetworkPayload>,
    server_ip: String,
}

impl NetworkPlugin {
    pub fn client(ip: String, greeting: NetworkPayload) -> Self {
        Self {
            client_payload: Some(greeting),
            server_ip: ip,
        }
    }

    pub fn server(ip: String) -> Self {
        Self {
            client_payload: None,
            server_ip: ip,
        }
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_event::<ConnectionEvent>();
        app_builder.add_stage_before(
            bevy::app::CoreStage::PostUpdate,
            NetworkStage::Spawn,
            SystemStage::parallel(),
        );

        app_builder.insert_resource(NetworkMessages::new());
        app_builder.insert_resource(NetworkEventSender::new());
        app_builder.insert_resource(NetworkSpawner::new());
        app_builder.insert_resource(NetworkEntityRegistry::new());
        app_builder.insert_resource(NetworkSettings {
            is_server: self.client_payload.is_none(),
        });

        app_builder.add_system_to_stage(
            bevy::app::CoreStage::PreUpdate,
            network_receive_system.system(),
        );

        app_builder.add_system_to_stage(
            bevy::app::CoreStage::PostUpdate,
            network_event_sender_system.system(),
        );

        app_builder.add_system_to_stage(
            NetworkStage::Spawn,
            network_spawner_system.exclusive_system(),
        );

        if let Some(greeting) = &self.client_payload {
            let stream = TcpStream::connect(&self.server_ip).unwrap();

            app_builder.insert_resource(NetworkResource::client(stream, greeting).unwrap());
        } else {
            app_builder.insert_resource(ServerResource::new(&self.server_ip).unwrap());
            app_builder.insert_resource(NetworkResource::empty());

            app_builder.add_system_to_stage(
                bevy::app::CoreStage::PreUpdate,
                network_server_system.system(),
            );
        }
    }
}

pub trait NetworkAppBuilderExt {
    fn app_builder(&mut self) -> &mut AppBuilder;

    fn register_network_event<T: NetworkEvent>(&mut self) -> &mut AppBuilder {
        let app_builder = self.app_builder();

        app_builder.insert_resource(NetworkEvents::<T>::new());

        app_builder.add_system_to_stage(
            bevy::app::CoreStage::PreUpdate,
            network_event_system::<T>.system(),
        );

        app_builder
    }

    fn register_network_spawnable<T: NetworkSpawnable>(&mut self) -> &mut AppBuilder {
        let app_builder = self.app_builder();

        app_builder
            .world_mut()
            .get_resource_mut::<NetworkSpawner>()
            .unwrap()
            .register_spawnable::<T>();

        app_builder
    }
}

impl NetworkAppBuilderExt for AppBuilder {
    fn app_builder(&mut self) -> &mut AppBuilder {
        self
    }
}
