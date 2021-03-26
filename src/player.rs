use crate::input::*;
use crate::networking::*;
use crate::world_transform::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "1f4df47b-58da-477b-9921-0ac53cefd889"]
pub enum PlayerInputEvent {
    // TODO: consider removing the NetworkEntity and just look up the senders player.
    SetMovement(NetworkEntity, Vec2),
}

pub struct Player {
    movement_vector: Vec2,
    actor_id: ActorId,
}

pub struct PlayerResource {
    player_entity: Option<Entity>,
}

pub fn player_system(
    time: Res<Time>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    mut events: ResMut<NetworkEvents<PlayerInputEvent>>,
    mut query: Query<(&mut Player, &mut WorldTransform)>,
) {
    for (sender, event) in events.take() {
        match event {
            PlayerInputEvent::SetMovement(network_entity, movement_vector) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (mut player, _) = query.get_mut(entity).unwrap();

                if player.actor_id == sender {
                    player.movement_vector = movement_vector;
                } else {
                    warn!(
                        "Actor: '{:?}' tried to move the player of: '{:?}'",
                        sender, player.actor_id
                    );
                }
            }
        }
    }

    for (player, mut world_transform) in query.iter_mut() {
        // TODO: get movement speed from the players frame
        let movement_speed = 32.0;

        world_transform.translation +=
            player.movement_vector.extend(0.0) * movement_speed * time.delta_seconds();
    }
}

pub fn player_input_system(
    input_handle: Res<Handle<InputSettings>>,
    input_settings: Res<Assets<InputSettings>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    event_sender: Res<NetworkEventSender>,
    player_resource: Res<PlayerResource>,
    mut query: Query<(&NetworkEntity, &mut Player)>,
) {
    if let Some(entity) = player_resource.player_entity {
        let (network_entity, mut player) = query.get_mut(entity).unwrap();

        let input_ctx = InputCtx {
            keyboard: &*keyboard_input,
            mouse: &*mouse_input,
        };

        let input = input_settings.get(&*input_handle).unwrap();

        let mut movement_vector = Vec2::ZERO;

        if input.up.pressed(&input_ctx) {
            movement_vector += Vec2::new(0.0, 1.0);
        }

        if input.down.pressed(&input_ctx) {
            movement_vector += Vec2::new(0.0, -1.0);
        }

        if input.left.pressed(&input_ctx) {
            movement_vector += Vec2::new(-1.0, 0.0);
        }

        if input.right.pressed(&input_ctx) {
            movement_vector += Vec2::new(1.0, 0.0);
        }

        event_sender
            .send(&PlayerInputEvent::SetMovement(
                *network_entity,
                movement_vector,
            ))
            .unwrap();

        player.movement_vector = movement_vector;
    }
}

#[derive(TypeUuid, Clone, Copy, Serialize, Deserialize)]
#[uuid = "053c55fe-dcd8-4746-829f-51760445739e"]
pub struct PlayerSpawner {
    pub player_id: ActorId,
}

impl NetworkSpawnable for PlayerSpawner {
    fn spawn(&self, world: &mut World) -> Entity {
        let world_transform = WorldTransform {
            translation: Vec3::new(0.0, 0.0, 0.0),
        };

        let player = Player {
            movement_vector: Vec2::ZERO,
            actor_id: self.player_id,
        };

        if world.get_resource::<NetworkSettings>().unwrap().is_server {
            world
                .spawn()
                .insert(Transform::identity())
                .insert(world_transform)
                .insert(player)
                .id()
        } else {
            let texture = world
                .get_resource::<Assets<Texture>>()
                .unwrap()
                .get_handle("arrow.png");
            let material = world
                .get_resource_mut::<Assets<ColorMaterial>>()
                .unwrap()
                .add(texture.into());

            let entity = world
                .spawn()
                .insert_bundle(SpriteBundle {
                    material,
                    ..Default::default()
                })
                .insert(world_transform)
                .insert(player)
                .id();

            if world.get_resource::<NetworkResource>().unwrap().local_id == self.player_id {
                world
                    .get_resource_mut::<PlayerResource>()
                    .unwrap()
                    .player_entity = Some(entity);
            }

            entity
        }
    }
}

pub struct PlayerPlugin;

impl Plugin for PlayerPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let is_server = app_builder
            .world()
            .get_resource::<NetworkSettings>()
            .unwrap()
            .is_server;

        app_builder.register_network_event::<PlayerInputEvent>();
        app_builder.register_network_spawnable::<PlayerSpawner>();

        if is_server {
            app_builder.add_system(player_system.system());
        } else {
            app_builder.add_system(player_input_system.system());

            app_builder.insert_resource(PlayerResource {
                player_entity: None,
            });
        }
    }
}
