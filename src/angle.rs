use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum Angle {
    Up,
    UpRight,
    Right,
    DownRight,
    Down,
    DownLeft,
    Left,
    UpLeft,
}
