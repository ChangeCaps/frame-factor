use crate::networking::*;
use bevy::prelude::*;

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "219b96a9-7102-4102-9c5d-ca9e7e6b3dbb"]
pub struct Attack {
    pub damage: f32,
    pub hitbox: Vec<Vec2>,
}
