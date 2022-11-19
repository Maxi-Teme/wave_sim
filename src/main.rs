use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;

mod colored_mesh;
mod longitudinal_wave_3d_simulation;
mod pan_orbit_camera;
mod ui;
mod wave_2d_simulation;

use longitudinal_wave_3d_simulation::LongitudinalWaveSimulation3dPlugin;
use ui::UiPlugin;
use wave_2d_simulation::Wave2dSimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Wave2dSimulation,
    LongitudinalWaveSimulation3d,
}

impl AppState {
    fn start() -> Self {
        Self::LongitudinalWaveSimulation3d
    }

    fn all_as_string() -> Vec<String> {
        vec![
            Self::Wave2dSimulation.into(),
            Self::LongitudinalWaveSimulation3d.into(),
        ]
    }
}

impl From<String> for AppState {
    fn from(value: String) -> Self {
        if value == "wave_2d" {
            Self::Wave2dSimulation
        } else {
            Self::LongitudinalWaveSimulation3d
        }
    }
}

impl From<AppState> for String {
    fn from(value: AppState) -> Self {
        match value {
            AppState::Wave2dSimulation => "wave_2d".to_string(),
            _ => "longitudinal_wave_3d".to_string(),
        }
    }
}

#[derive(Component)]
pub struct UiCamera;

#[derive(Component)]
pub struct AppCamera;

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
                resizable: true,
                ..default()
            },
            ..default()
        }))
        .insert_resource(Msaa { samples: 1 })
        .add_state(AppState::start())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(UiPlugin)
        // simulation plugins
        .add_plugin(Wave2dSimulationPlugin)
        .add_plugin(LongitudinalWaveSimulation3dPlugin)
        // camera configuration
        .add_startup_system(setup_cameras)
        // event consumption
        .run();
}

fn setup_cameras(mut commands: Commands) {
    commands.spawn((AppCamera, Camera2dBundle::default()));
}
