use crate::animation::*;
use crate::networking::*;
use crate::player::*;
use crate::transform::*;
use bevy::prelude::*;
use std::collections::HashMap;
use bevy_rapier2d::rapier::dynamics::RigidBodyBuilder;

#[derive(Serialize, Deserialize)]
pub enum AttackType {
    LightAttack,
}

#[derive(Serialize, Deserialize)]
pub enum AttackEvent {
    /// Activates the hitbox and sets it.
    ActivateHitbox {
        stun: u32,
        damage: f32,
        animation: String,
        hitbox: Vec<Vec2>,
    },
    /// Stuns the player.
    Stun(u32),
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "219b96a9-7102-4102-9c5d-ca9e7e6b3dbb"]
pub struct Attack {
    pub animation: String,
    pub events: HashMap<u32, Vec<AttackEvent>>,
}

#[derive(Clone, Serialize, Deserialize)]
pub struct Damage {
    pub source: NetworkEntity,
    pub stun: u32,
    pub damage: f32,
}

pub struct AttackController {
    pub attack: Option<Handle<Attack>>,
}

impl AttackController {
    pub fn new() -> Self {
        Self { attack: None }
    }

    pub fn attack(&mut self, attack: Handle<Attack>) {
        self.attack = Some(attack);
    }

    pub fn stop(&mut self) {
        self.attack = None;
    }
}

pub fn attack_server_system(
    attacks: Res<Assets<Attack>>,
    network_spawner: Res<NetworkSpawner>,
    mut query: Query<(
        Entity,
        &NetworkEntity,
        &mut AttackController,
        &mut Player,
        &Animator,
        &Transform,
    )>,
) {
    for (entity, network_entity, mut attack_controller, mut player, animator, world_transform) in
        query.iter_mut()
    {
        if attack_controller.attack.is_some() && animator.just_ended() {
            attack_controller.stop();
        }

        // handle current attack if precent
        if animator.just_advanced() {
            if let Some(attack_handle) = attack_controller.attack.clone() {
                let attack = attacks.get(attack_handle).unwrap();

                let frame = animator.frame();

                // get and handle events
                if let Some(events) = attack.events.get(&frame) {
                    for event in events {
                        match event {
                            AttackEvent::ActivateHitbox {
                                stun,
                                damage,
                                animation,
                                hitbox,
                            } => {
                                let spawner = AttackHitSpawner {
                                    parent: *network_entity,
                                    animation: animation.clone(),
                                    damage: Damage {
                                        source: *network_entity,
                                        stun: *stun,
                                        damage: *damage,
                                    },
                                    direction: player.aim_direction,
                                    hitbox: hitbox.clone(),
                                };

                                network_spawner.spawn(spawner);
                            }

                            AttackEvent::Stun(duration) => {
                                player.stun(*duration);
                                info!("stunning player");
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn attack_hit_server_system(
    query: Query<(Entity, &Damage)>,
    mut player_query: Query<(&NetworkEntity, &mut Player)>,
) {
    for (entity, damage) in query.iter() {
        /*
        for entity in collision_resource.just_intersected(&entity) {
            if let Ok((network_entity, mut player)) = player_query.get_mut(entity) {
                if *network_entity != damage.source {
                    player.hit(damage);
                }
            }
        }
        */
    }
}

pub fn attack_hit_despawn_system(
    mut commands: Commands,
    query: Query<(Entity, &Animator), With<Damage>>,
) {
    for (entity, animator) in query.iter() {
        if animator.just_ended() {
            commands.entity(entity).despawn();
        }
    }
}

#[derive(Clone, Serialize, Deserialize, TypeUuid)]
#[uuid = "7d2a65c2-6e1d-4d0a-822a-2ce0c84c5a4c"]
pub struct AttackHitSpawner {
    pub parent: NetworkEntity,
    pub animation: String,
    pub damage: Damage,
    pub direction: Vec2,
    pub hitbox: Vec<Vec2>,
}

impl NetworkSpawnable for AttackHitSpawner {
    fn spawn(&self, world: &mut World) -> Entity {
        let is_server = world.get_resource::<NetworkSettings>().unwrap().is_server;

        let parent = world
            .get_resource::<NetworkEntityRegistry>()
            .unwrap()
            .get(&self.parent)
            .unwrap();

        let rotation = self.direction.y.atan2(self.direction.x) + std::f32::consts::PI / 2.0;

        let collider = crate::helper::collision::polygon_collider(self.hitbox.clone()).sensor(true);
        let rigidbody = RigidBodyBuilder::new_kinematic().rotation(rotation);

        let mut animator = Animator::new();
        animator.play(self.animation.clone());

        if is_server {
            world
                .spawn()
                .insert(self.damage.clone())
                .insert(animator)
                .insert(collider)
                .insert(rigidbody)
                .insert(Transform::default())
                .insert(GlobalTransform::default())
                .insert(crate::helper::polygon_collider(self.hitbox.clone()))
                .insert(Parent(parent))
                .id()
        } else {
            world
                .spawn()
                .insert_bundle(AnimatorBundle {
                    animator,
                    transform: Transform::from_rotation(Quat::from_rotation_z(rotation)),
                    ..Default::default()
                })
                .insert(self.damage.clone())
                .insert(Parent(parent))
                .id()
        }
    }
}

pub struct AttackLoader;

crate::ron_loader!(AttackLoader, "atk" => Attack);

pub struct AttackPlugin;

impl Plugin for AttackPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        let is_server = app_builder
            .world()
            .get_resource::<NetworkSettings>()
            .unwrap()
            .is_server;

        app_builder.add_asset::<Attack>();
        app_builder.add_asset_loader(AttackLoader);
        app_builder.register_network_spawnable::<AttackHitSpawner>();

        app_builder.add_system(attack_hit_despawn_system.system());

        if is_server {
            app_builder.add_system(attack_server_system.system());
            app_builder.add_system(attack_hit_server_system.system());
        }
    }
}
