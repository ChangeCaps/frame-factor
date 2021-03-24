use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum GameState {
    Connection,
    PreDungeon,
    Dungeon,
    PreDuel,
    Duel,
    Ended,
}
