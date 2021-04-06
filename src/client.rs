use crate::animation::*;
use crate::attack::*;
use crate::camera::*;
use crate::collider::*;
use crate::frame::*;
use crate::input::*;
use crate::networking::*;
use crate::player::*;
use crate::progress_bar::*;
use crate::world_transform::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "9968f81b-59da-4292-8015-d6d4bbccb5c7"]
pub struct ClientGreeting {}

/// Runs the client.
pub fn run(ip: String) {
    let greeting = ClientGreeting {};
    let payload = NetworkPayload::new(&greeting, ClientGreeting::TYPE_UUID);

    App::build()
        // resources
        .insert_resource(bevy::ecs::schedule::ReportExecutionOrderAmbiguities)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(Mouse::default())
        // plugins
        .add_plugins(DefaultPlugins)
        .add_plugin(NetworkPlugin::client(ip, payload))
        .add_plugin(PlayerPlugin)
        .add_plugin(InputPlugin)
        .add_plugin(FramePlugin)
        .add_plugin(CollisionPlugin)
        .add_plugin(ProgressBarPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(AttackPlugin)
        // network events
        .register_network_event::<WorldTransformEvent>()
        // network spawnables
        // systems
        .add_system(mouse_system.system())
        .add_system(world_transform_system.system())
        .add_system(world_transform_network_system.system())
        // startup systems
        .add_startup_system(setup_system.system())
        .run();
}

/// Setup system for the client.
fn setup_system(mut commands: Commands, asset_server: Res<AssetServer>) {
    asset_server.watch_for_changes().unwrap();
    let handles = asset_server.load_folder(".").unwrap();

    std::thread::sleep_ms(100); // FIXME: please

    let mut camera_bundle = OrthographicCameraBundle::new_2d();

    camera_bundle.orthographic_projection.scaling_mode =
        bevy::render::camera::ScalingMode::FixedVertical;
    camera_bundle.orthographic_projection.scale = 324.0;

    commands
        .spawn()
        .insert_bundle(camera_bundle)
        .insert(MainCamera);

    commands.insert_resource(handles);
}
