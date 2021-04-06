use bevy::prelude::*;

pub struct MainCamera;

#[derive(Default)]
pub struct Mouse {
    pub world_position: Vec2,
}

pub fn mouse_system(
    mut events: EventReader<CursorMoved>,
    mut mouse: ResMut<Mouse>,
    windows: Res<Windows>,
    query: Query<&GlobalTransform, With<MainCamera>>,
) {
    let transform = query.iter().next().unwrap();

    for event in events.iter() {
        let window = windows.get(event.id).unwrap();
        let size = Vec2::new(window.width() as f32, window.height() as f32);

        let p = (event.position - size / 2.0) / size.y * 324.0 * 2.0;

        let world_position = transform
            .compute_matrix()
            .transform_point3(p.extend(0.0))
            .truncate();
        mouse.world_position = world_position;
    }
}
