use serde::{Deserialize, Serialize};

/// The state of the game.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Hash)]
pub enum GameState {
    /// When the game has started but is waiting for everyone to connect.
    Connection,
    /// Shows the statistics screen for the players.
    PreDungeon,
    /// When the players are in the dungeon.
    Dungeon,
    /// When the players are done with the dungeon and the players items are shown.
    PreDuel,
    /// When the players are dueling
    Duel,
    /// After the duel has ended and the score screen is shown.
    Ended,
}
