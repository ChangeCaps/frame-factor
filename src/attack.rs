use crate::networking::*;
use bevy::prelude::*;
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub enum AttackEvent {
    ActivateHitbox {
        damage: f32,
        hitbox: Vec<Vec2>,
    },
    DeactivateHitbox,
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "219b96a9-7102-4102-9c5d-ca9e7e6b3dbb"]
pub struct Attack {
    pub animation: String,
    pub events: HashMap<u32, Vec<AttackEvent>>,
}

pub struct AttackLoader;

crate::ron_loader!(AttackLoader, "atk" => Attack);
