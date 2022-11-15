use bevy::prelude::*;

mod finite_difference;
mod simulation_plugin;

use bevy::window::PresentMode;
use simulation_plugin::SimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

fn main() {
    let height = 900.0;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                height,
                width: height * RESOLUTION,
                title: "Wave Sim".to_string(),
                present_mode: PresentMode::Immediate,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .add_plugin(SimulationPlugin)
        .run();
}
