use bevy::{prelude::*, reflect::TypeUuid};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, TypeUuid)]
#[uuid = "0c4e97f6-10a1-4a92-98c7-07c81bebfe9a"]
pub struct InputSettings {
    pub up: ButtonInput,
    pub down: ButtonInput,
    pub left: ButtonInput,
    pub right: ButtonInput,
    pub light_attack: ButtonInput,
}

#[derive(Serialize, Deserialize)]
pub enum ButtonInput {
    Key(KeyCode),
    Mouse(MouseButton),
}

pub struct InputCtx<'a> {
    pub keyboard: &'a Input<KeyCode>,
    pub mouse: &'a Input<MouseButton>,
}

impl ButtonInput {
    pub fn pressed(&self, input: &InputCtx) -> bool {
        match self {
            Self::Key(key) => input.keyboard.pressed(*key),
            Self::Mouse(btn) => input.mouse.pressed(*btn),
        }
    }

    pub fn just_pressed(&self, input: &InputCtx) -> bool {
        match self {
            Self::Key(key) => input.keyboard.just_pressed(*key),
            Self::Mouse(btn) => input.mouse.just_pressed(*btn),
        }
    }
}

pub struct InputLoader;

crate::ron_loader!(InputLoader, "inp" => InputSettings);

pub struct InputPlugin;

impl Plugin for InputPlugin {
    fn build(&self, app_builder: &mut AppBuilder) {
        app_builder.add_asset::<InputSettings>();

        let input_handle = app_builder
            .world()
            .get_resource::<Assets<InputSettings>>()
            .unwrap()
            .get_handle("input.inp");

        app_builder.insert_resource(input_handle);

        app_builder.add_asset_loader(InputLoader);
    }
}
