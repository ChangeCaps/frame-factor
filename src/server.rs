use crate::collider::*;
use crate::frame::*;
use crate::game_state::*;
use crate::networking::*;
use crate::player::*;
use crate::world_transform::*;
use bevy::prelude::*;

pub struct Players {
    pub players: Vec<ActorId>,
}

impl Players {
    pub fn new() -> Self {
        Self {
            players: Vec::new(),
        }
    }
}

pub fn run(ip: String) {
    App::build()
        // resources
        .insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_secs_f32(1.0 / 40.0),
        ))
        .insert_resource(Players::new())
        // plugins
        .add_plugins(MinimalPlugins)
        .add_plugin(bevy::asset::AssetPlugin)
        .add_plugin(bevy::log::LogPlugin)
        .add_plugin(NetworkPlugin::server(ip))
        .add_plugin(PlayerPlugin)
        .add_plugin(FramePlugin)
        .add_plugin(CollisionPlugin)
        // network events
        .register_network_event::<WorldTransformEvent>()
        // network spawnables
        // state
        .add_state(GameState::Connection)
        // systems
        .add_system(world_transform_system.system())
        .add_system(connection_system.system())
        // startup systems
        .add_startup_system(startup_system.system())
        // run
        .run();
}

fn startup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
    let handles = asset_server.load_folder(".").unwrap();

    commands.insert_resource(handles);
}

fn connection_system(
    mut event_reader: EventReader<ConnectionEvent>,
    mut players: ResMut<Players>,
    mut net: ResMut<NetworkResource>,
    network_spawner: Res<NetworkSpawner>,
) {
    for event in event_reader.iter() {
        match event {
            ConnectionEvent::Connected { id, .. } => {
                // TODO: use a more dynamic way of checking player count
                if players.players.len() >= 2 {
                    warn!("Player cap exceeded, '{:?}' tired to connect", id);
                    continue;
                }

                info!("Player connected '{:?}'", id);
                players.players.push(*id);

                if players.players.len() == 2 {
                    info!("All players connected, starting game");

                    for id in &players.players {
                        network_spawner.spawn(PlayerSpawner {
                            frame: String::from("frames/katana_one/frame.fme"),
                            player_id: *id,
                        });
                    }
                }
            }
            ConnectionEvent::Disconnected { id, error } => {
                warn!("'{:?}' disconnected with error: '{:?}'", id, error);

                net.remove_connection(&id);
            }
        }
    }
}
