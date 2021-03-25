//! Contains all the resources for connecting.

use super::*;
use bevy::prelude::*;
use std::{
    collections::HashMap,
    net::{TcpListener, TcpStream},
};

/// An id given to each *actor* on the network, these include:
/// * Server
/// * Client
///
/// [`ActorId`]s should always be syncronized, so any actor can be
/// unequely identified and referred to.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ActorId(pub u32);

/// When a client connects to the server, the server sends this greeting
/// to the client with all the perdinent information.
#[derive(Serialize, Deserialize)]
pub struct ServerGreeting {
    client_id: ActorId,
    server_id: ActorId,
}

pub enum ConnectionEvent {
    Connected { id: ActorId },
    Disconnected { id: ActorId, error: anyhow::Error },
}

/// The connections stored in the [`NetworkResource`].
pub struct Connection {
    stream: TcpStream,
}

impl Connection {
    /// Creates a new [`Connection`] from a [`TcpStream`].
    ///
    /// **Note** this both sets nonblocking and nodelay on the stream.
    pub fn new(stream: TcpStream) -> anyhow::Result<Self> {
        stream.set_nonblocking(true)?;
        stream.set_nodelay(true)?;

        Ok(Self { stream })
    }

    /// Receives a [`Vec`] of [`NetworkPayload`]s.
    pub fn recv(&self) -> anyhow::Result<Vec<NetworkPayload>> {
        let mut payloads = Vec::new();

        loop {
            let payload = bincode::deserialize_from(&self.stream);

            match payload {
                Ok(payload) => {
                    payloads.push(payload);
                }
                Err(e) => match *e {
                    bincode::ErrorKind::Io(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        break Ok(payloads);
                    }
                    e => break Err(e.into()),
                },
            }
        }
    }

    pub fn send(&self, payload: &NetworkPayload) -> anyhow::Result<()> {
        bincode::serialize_into(&self.stream, payload)?;

        Ok(())
    }
}

/// A resource used for storing server specific networking data.
pub struct ServerResource {
    listener: TcpListener,
}

impl ServerResource {
    /// Creates a new [`ServerResource`] and binds to the given ip.
    pub fn new(ip: &String) -> anyhow::Result<Self> {
        let listener = TcpListener::bind(ip)?;

        listener.set_nonblocking(true)?;

        Ok(Self { listener })
    }

    /// Listens for connections and adds any connected clients to the [`NetworkResource`].
    ///
    /// **Note** this will never block.
    pub fn listen(&self, net: &mut NetworkResource) -> anyhow::Result<Vec<ConnectionEvent>> {
        let mut events = Vec::new();

        for stream in self.listener.incoming() {
            match stream {
                Ok(stream) => {
                    let id = net.add_client(stream)?;

                    events.push(ConnectionEvent::Connected { id });
                }
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(events);
                }
                e => {
                    e?;
                }
            }
        }

        Ok(events)
    }
}

/// Listens for connecting clients and connects them.
pub fn network_server_system(
    server_resource: Res<ServerResource>,
    mut net: ResMut<NetworkResource>,
    mut events: EventWriter<ConnectionEvent>,
) {
    let connection_events = server_resource.listen(&mut *net).unwrap();

    events.send_batch(connection_events.into_iter());
}

/// Contains all the data for networking.
pub struct NetworkResource {
    pub connections: HashMap<ActorId, Connection>,
    next_id: ActorId,
    pub server_id: ActorId,
    pub local_id: ActorId,
}

impl NetworkResource {
    /// Creates an empty [`NetworkResource`], this assumes both the local_id
    /// and server_id to be 0.
    pub fn empty() -> Self {
        Self {
            connections: HashMap::new(),
            next_id: ActorId(1),
            server_id: ActorId(0),
            local_id: ActorId(0),
        }
    }

    /// Creates a [`NetworkResource`] for a client from the given [`TcpStream`].
    ///
    /// **Note** this will block, since it listens for a [`ServerGreeting`].
    pub fn client(mut stream: TcpStream) -> anyhow::Result<Self> {
        let mut network_resource = Self::empty();

        let greeting: ServerGreeting = bincode::deserialize_from(&mut stream)?;

        network_resource.insert_connection(greeting.server_id, Connection::new(stream)?);

        network_resource.server_id = greeting.server_id;
        network_resource.local_id = greeting.client_id;

        Ok(network_resource)
    }

    /// Takes a [`TcpStream`], sends a [`ServerGreeting`] and adds it as a connection.
    pub fn add_client(&mut self, mut stream: TcpStream) -> anyhow::Result<ActorId> {
        let id = self.next_id;

        let greeting = ServerGreeting {
            client_id: id,
            server_id: self.server_id,
        };

        bincode::serialize_into(&mut stream, &greeting)?;
        self.connections.insert(id, Connection::new(stream)?);

        self.next_id.0 += 1;

        Ok(id)
    }

    /// Inserts a connection.
    pub fn insert_connection(&mut self, actor_id: ActorId, connection: Connection) {
        self.next_id.0 = actor_id.0 + 1;
        self.connections.insert(actor_id, connection);
    }

    pub fn recv(&self) -> (Vec<NetworkMessage>, Vec<ConnectionEvent>) {
        let mut messages = Vec::new();
        let mut connection_events = Vec::new();

        for (id, connection) in &self.connections {
            match connection.recv() {
                Ok(m) => {
                    for payload in m {
                        messages.push(NetworkMessage {
                            sender: *id,
                            payload,
                        });
                    }
                }
                Err(e) => {
                    connection_events.push(ConnectionEvent::Disconnected { id: *id, error: e });
                }
            }
        }

        (messages, connection_events)
    }

    pub fn send(&self, payload: &NetworkPayload) -> anyhow::Result<()> {
        for (_id, connection) in &self.connections {
            connection.send(&payload)?;
        }

        Ok(())
    }
}
