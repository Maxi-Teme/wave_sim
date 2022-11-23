use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_rapier3d::prelude::*;

mod colored_mesh;
mod longitudinal_wave_3d_simulation;
mod objects_3d;
mod pan_orbit_camera;
mod particle_mess;
mod ui;
mod wave_2d_simulation;
mod wave_in_panel;

use longitudinal_wave_3d_simulation::LongitudinalWave3dSimulationPlugin;
use particle_mess::ParticleMessPlugin;
use ui::UiPlugin;
use wave_2d_simulation::Wave2dSimulationPlugin;
use wave_in_panel::WaveInPanelPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Wave2dSimulation,
    LongitudinalWaveSimulation3d,
    ParticleMess,
    WaveInPanel,
}

impl AppState {
    fn start() -> Self {
        Self::WaveInPanel
    }
}

impl From<AppState> for String {
    fn from(value: AppState) -> Self {
        match value {
            AppState::Wave2dSimulation => "wave_2d".to_string(),
            AppState::LongitudinalWaveSimulation3d => {
                "longitudinal_wave_3d".to_string()
            }
            AppState::ParticleMess => "particle_mess".to_string(),
            AppState::WaveInPanel => "wave_in_panel".to_string(),
        }
    }
}

#[derive(Component)]
pub struct AppCamera;

fn main() {
    let height = 900.0;

    App::new()
        // core systems
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
        // app
        .add_state(AppState::start())
        // physics
        .insert_resource(RapierConfiguration::default())
        .add_plugin(RapierPhysicsPlugin::<()>::default())
        // debug systems
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        // ui configuration
        .add_plugin(UiPlugin)
        // simulation systems
        .add_plugin(Wave2dSimulationPlugin)
        .add_plugin(LongitudinalWave3dSimulationPlugin)
        .add_plugin(ParticleMessPlugin)
        .add_plugin(WaveInPanelPlugin)
        .run();
}
