use crate::animation::*;
use crate::attack::*;
use crate::camera::*;
use crate::frame::*;
use crate::input::*;
use crate::networking::*;
use crate::progress_bar::*;
use crate::transform::*;
use bevy::prelude::*;
use heron::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "1d042690-8b1a-45ec-94db-7fdccaab7090"]
pub enum PlayerEvent {
    SetHealth(NetworkEntity, f32),
    SetAttacking(NetworkEntity, bool),
    SetMovementVector(NetworkEntity, Vec2),
    PlayAnimation(NetworkEntity, String),
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "1f4df47b-58da-477b-9921-0ac53cefd889"]
pub enum PlayerInputEvent {
    // TODO: consider removing the NetworkEntity and just look up the senders player.
    SetMovement(NetworkEntity, Vec2),
    Attack(NetworkEntity, AttackType),
    SetAimDirection(NetworkEntity, Vec2),
}

pub struct Player {
    pub frame: Handle<Frame>,
    pub movement_vector: Vec2,
    pub actor_id: ActorId,
    pub health: f32,
    pub update_health: bool,
    pub stun: Option<u32>,
    pub attacking: bool,
    pub aim_direction: Vec2,
}

impl Player {
    #[inline(always)]
    pub fn damage(&mut self, damage: f32) {
        self.health -= damage;
        self.update_health = true;
    }

    #[inline(always)]
    pub fn stun(&mut self, new_stun: u32) {
        if let Some(stun) = &mut self.stun {
            *stun = (*stun).max(new_stun);
        } else {
            self.stun = Some(new_stun);
        }
    }

    #[inline(always)]
    pub fn hit(&mut self, hit: &Damage) {
        self.damage(hit.damage);
        self.stun(hit.stun);
    }
}

pub struct PlayerResource {
    player_entity: Option<Entity>,
}

pub fn player_server_system(
    time: Res<Time>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    attacks: Res<Assets<Attack>>,
    frames: Res<Assets<Frame>>,
    event_sender: Res<NetworkEventSender>,
    mut events: ResMut<NetworkEvents<PlayerInputEvent>>,
    mut query: Query<(
        &NetworkEntity,
        &mut Player,
        &mut Animator,
        &mut AttackController,
        &mut Transform,
        &mut Velocity,
    )>,
) {
    // update players
    for (network_entity, mut player, animator, _, _, mut velocity) in query.iter_mut() {
        // remove stun if duration is over
        if player.stun == Some(0) {
            player.stun = None;
        }

        // if stun is active tick down
        if let Some(stun) = &mut player.stun {
            *stun -= 1;
        }

        // if the player is currently attacking and an animation just ended,
        // set attacking to false
        if player.attacking && (animator.just_ended() || !animator.is_playing()) {
            player.attacking = false;
            event_sender
                .send(&PlayerEvent::SetAttacking(*network_entity, false))
                .unwrap();
        }

        if player.update_health {
            player.update_health = false;
            let event = PlayerEvent::SetHealth(*network_entity, player.health);

            event_sender.send(&event).unwrap();
        }

        let frame = frames.get(&player.frame).unwrap();

        if player.stun.is_none() {
            let v = if player.movement_vector.length() == 0.0 {
                Vec2::ZERO
            } else {
                player.movement_vector.normalize() * frame.walking_speed
            };
    
            velocity.linear = v.extend(0.0);
        }
    }

    // handle player input events
    for (sender, event) in events.take() {
        match event {
            PlayerInputEvent::SetMovement(network_entity, movement_vector) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (_, mut player, _, _, _, _) = query.get_mut(entity).unwrap();

                if player.actor_id == sender {
                    player.movement_vector = movement_vector;

                    event_sender
                        .send(&PlayerEvent::SetMovementVector(
                            network_entity,
                            movement_vector,
                        ))
                        .unwrap();
                } else {
                    warn!(
                        "Actor: '{:?}' tried to move as: '{:?}'",
                        sender, player.actor_id
                    );
                }
            }

            PlayerInputEvent::Attack(network_entity, attack_type) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();
                let (network_entity, mut player, mut animator, mut attack_controller, _, _) =
                    query.get_mut(entity).unwrap();

                if player.attacking || player.actor_id != sender {
                    continue;
                }

                let frame = frames.get(&player.frame).unwrap();
                let attack = frame.get_attack(&attack_type);
                let attack_handle = attacks.get_handle(attack.as_str());
                let attack = attacks.get(&attack_handle).unwrap();

                attack_controller.attack(attack_handle);
                animator.play(attack.animation.clone());

                player.attacking = true;
                event_sender
                    .send(&PlayerEvent::SetAttacking(*network_entity, true))
                    .unwrap();
                event_sender
                    .send(&PlayerEvent::PlayAnimation(
                        *network_entity,
                        attack.animation.clone(),
                    ))
                    .unwrap();
            }

            PlayerInputEvent::SetAimDirection(network_entity, aim_direction) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();
                let (_, mut player, _, _, _, _) = query.get_mut(entity).unwrap();

                if player.actor_id != sender {
                    continue;
                }

                player.aim_direction = aim_direction;
            }
        }
    }
}

pub fn player_input_system(
    input_handle: Res<Handle<InputSettings>>,
    input_settings: Res<Assets<InputSettings>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    event_sender: Res<NetworkEventSender>,
    player_resource: Res<PlayerResource>,
    mouse: Res<Mouse>,
    mut query: Query<(&NetworkEntity, &mut Player, &Transform)>,
) {
    if let Some(entity) = player_resource.player_entity {
        let (network_entity, mut player, transform) = query.get_mut(entity).unwrap();

        let input_ctx = InputCtx {
            keyboard: &*keyboard_input,
            mouse: &*mouse_input,
        };

        let input = input_settings.get(&*input_handle).unwrap();

        // movement

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

        // rotation
        let diff = transform.translation.truncate() - mouse.world_position;

        event_sender
            .send(&PlayerInputEvent::SetAimDirection(
                *network_entity,
                diff.normalize(),
            ))
            .unwrap();

        // attacks

        if input.light_attack.just_pressed(&input_ctx) {
            event_sender
                .send(&PlayerInputEvent::Attack(
                    *network_entity,
                    AttackType::LightAttack,
                ))
                .unwrap();
        }
    }
}

pub fn player_client_system(
    mut events: ResMut<NetworkEvents<PlayerEvent>>,
    network_entity_registry: Res<NetworkEntityRegistry>,
    frames: Res<Assets<Frame>>,
    mut player_query: Query<(&mut Player, &Children, &mut Animator)>,
    mut health_bar_query: Query<&mut ProgressBar>,
) {
    for (_sender, event) in events.take() {
        match event {
            PlayerEvent::SetHealth(network_entity, value) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (mut player, children, _) = player_query.get_mut(entity).unwrap();
                let mut health_bar = health_bar_query.get_mut(children[0]).unwrap();

                info!("health set!");

                player.health = value;
                health_bar.value = player.health;
            }
            PlayerEvent::SetMovementVector(network_entity, movement_vector) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (mut player, _, _) = player_query.get_mut(entity).unwrap();

                player.movement_vector = movement_vector;
            }
            PlayerEvent::SetAttacking(network_entity, attacking) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (mut player, _, _) = player_query.get_mut(entity).unwrap();

                player.attacking = attacking;
            }
            PlayerEvent::PlayAnimation(network_entity, animation) => {
                let entity = network_entity_registry.get(&network_entity).unwrap();

                let (_, _, mut animator) = player_query.get_mut(entity).unwrap();

                animator.play(animation);
            }
        }
    }

    for (player, _, mut animator) in player_query.iter_mut() {
        let frame = frames.get(&player.frame).unwrap();

        if !player.attacking && player.movement_vector.length() == 0.0 {
            animator.set_playing(frame.idle_animation.clone(), true);
        }
    }
}

#[derive(TypeUuid, Clone, Serialize, Deserialize)]
#[uuid = "053c55fe-dcd8-4746-829f-51760445739e"]
pub struct PlayerSpawner {
    pub frame: String,
    pub player_id: ActorId,
    pub position: Vec2,
}

impl NetworkSpawnable for PlayerSpawner {
    fn spawn(&self, world: &mut World) -> Entity {
        let frames = world.get_resource::<Assets<Frame>>().unwrap();

        let frame_handle = frames.get_handle(self.frame.as_str());

        let frame = frames.get(&frame_handle).unwrap();

        let max_health = frame.max_health;

        let player = Player {
            frame: frame_handle,
            movement_vector: Vec2::ZERO,
            actor_id: self.player_id,
            health: max_health,
            update_health: false,
            stun: None,
            attacking: false,
            aim_direction: Vec2::new(1.0, 0.0),
        };

        let mut animator = Animator::new();
        animator.play(frame.idle_animation.clone());

        let body = Body::Sphere {
            radius: frame.collider_radius,
        };

        let transform = Transform::from_translation(self.position.extend(0.0));

        if world.get_resource::<NetworkSettings>().unwrap().is_server {
            world
                .spawn()
                .insert(transform)
                .insert(GlobalTransform::default())
                .insert(player)
                .insert(body)
                .insert(Velocity::from_linear(Vec3::ZERO))
                .insert(animator)
                .insert(AttackController::new())
                .insert(RotationConstraints::lock())
                .id()
        } else {
            let progress_bar_material = world
                .get_resource::<Assets<ProgressBarMaterial>>()
                .unwrap()
                .get_handle("misc/health_bar.pb");

            let entity = world
                .spawn()
                .insert_bundle(AnimatorBundle {
                    animator,
                    transform,
                    ..Default::default()
                })
                .insert(player)
                .insert(ZSort)
                .with_children(|world| {
                    world.spawn_bundle(ProgressBarBundle {
                        progress_bar: ProgressBar {
                            value: max_health,
                            value_max: max_health,
                        },
                        material: progress_bar_material,
                        transform: Transform::from_translation(Vec3::new(0.0, 90.0, 0.0)),
                        ..Default::default()
                    });
                })
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

        app_builder.register_network_event::<PlayerEvent>();
        app_builder.register_network_event::<PlayerInputEvent>();
        app_builder.register_network_spawnable::<PlayerSpawner>();

        if is_server {
            app_builder.add_system(player_server_system.system());
        } else {
            app_builder.add_system(player_input_system.system());
            app_builder.add_system(player_client_system.system());

            app_builder.insert_resource(PlayerResource {
                player_entity: None,
            });
        }
    }
}
