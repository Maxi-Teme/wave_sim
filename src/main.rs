use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;

mod colored_mesh;
mod particle_3d_simulation;
mod ui;
mod wave_2d_simulation;

use particle_3d_simulation::Particle3dSimulationPlugin;
use ui::UiPlugin;
use wave_2d_simulation::Wave2dSimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Wave2dSimulation,
    Particle3dSimulation,
}

impl From<String> for AppState {
    fn from(value: String) -> Self {
        if value == "wave_2d_simulation" {
            Self::Wave2dSimulation
        } else {
            Self::Particle3dSimulation
        }
    }
}

impl From<AppState> for String {
    fn from(value: AppState) -> Self {
        match value {
            AppState::Wave2dSimulation => "wave_2d_simulation".to_string(),
            _ => "particle_3d_simulation".to_string(),
        }
    }
}

fn main() {
    let height = 900.0;

    App::new()
        // main plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                height,
                width: height * RESOLUTION,
                title: "wave_sim".to_string(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .insert_resource(Msaa { samples: 1 })
        .add_state(AppState::Wave2dSimulation)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(UiPlugin)
        // simulation plugins
        .add_plugin(Particle3dSimulationPlugin)
        .add_plugin(Wave2dSimulationPlugin)
        .run();
}
