use crate::networking::*;

#[derive(Serialize, Deserialize)]
pub enum GameMode {
    OneVersusOne,
}

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "4a559fd6-20c6-4d5e-85e8-3e5611b0987f"]
pub struct GameSettings {
    pub mode: GameMode,
}
