use crate::networking::*;
use crate::world_transform::*;
use bevy::prelude::*;

pub struct Player {
    actor_id: ActorId,
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

            world
                .spawn()
                .insert_bundle(SpriteBundle {
                    material,
                    ..Default::default()
                })
                .insert(world_transform)
                .insert(player)
                .id()
        }
    }
}
