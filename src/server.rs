use crate::networking::*;
use bevy::prelude::*;

pub fn run(ip: String) {
    App::build()
        // resources
        .insert_resource(bevy::app::ScheduleRunnerSettings::run_loop(
            std::time::Duration::from_secs_f32(1.0 / 40.0),
        ))
        // plugins
        .add_plugin(NetworkPlugin::server(ip))
        .add_plugins(MinimalPlugins)
        // systems
        // startup_systems
        .run();
}
