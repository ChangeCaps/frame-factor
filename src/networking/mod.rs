//! Contains the networking framework.

pub mod network_event;
pub mod network_payload;
pub mod network_resource;

pub use network_event::*;
pub use network_payload::*;
pub use network_resource::*;

use bevy::prelude::*;
use std::net::TcpStream;

pub struct NetworkPlugin {
    is_server: bool,
    server_ip: String,
}

impl NetworkPlugin {
    pub fn client(ip: String) -> Self {
        Self {
            is_server: false,
            server_ip: ip,
        }
    }

    pub fn server(ip: String) -> Self {
        Self {
            is_server: true,
            server_ip: ip,
        }
    }
}

impl Plugin for NetworkPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.insert_resource(NetworkMessages::new());

        app_builder.add_system_to_stage(
            bevy::app::CoreStage::PreUpdate,
            network_receive_system.system(),
        );

        if self.is_server {
            app_builder.insert_resource(ServerResource::new(&self.server_ip).unwrap());
            app_builder.insert_resource(NetworkResource::empty());
            
            app_builder.add_system_to_stage(
                bevy::app::CoreStage::PreUpdate,
                network_server_system.system(),
            );
        } else {
            let stream = TcpStream::connect(&self.server_ip).unwrap();

            app_builder.insert_resource(NetworkResource::client(stream).unwrap());
        }
    }
}
