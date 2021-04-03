use crate::attack::*;
use crate::networking::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "0dd5b51e-b5b4-4c84-8eec-c2e72d0cb0e8"]
pub struct Frame {
    pub name: String,
    pub max_health: f32,
    pub walking_speed: f32,
    pub collision_box: Vec<Vec2>,
    pub idle_animation: String,
    pub light_attack: String,
}

impl Frame {
    pub fn get_attack(&self, attack_type: &AttackType) -> &String {
        match attack_type {
            AttackType::LightAttack => &self.light_attack,
        }
    }
}

pub struct FrameLoader;

crate::ron_loader!(FrameLoader, "fme" => Frame);

pub struct FramePlugin;

impl Plugin for FramePlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<Frame>();
        app_builder.add_asset_loader(FrameLoader);
    }
}
