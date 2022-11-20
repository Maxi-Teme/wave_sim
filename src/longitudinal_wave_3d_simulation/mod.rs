use bevy::prelude::*;

mod animation_plugin;
mod simulation_plugin;
mod ui;

pub use animation_plugin::AnimationPlugin;
pub use simulation_plugin::SimulationPlugin;
pub use ui::{show_ui, UiEvents};

#[derive(Resource)]
pub struct LongitudinalWave3dSimulationParameters {
    // set on initialization
    pub dimx: usize,
    pub dimy: usize,
    pub dimz: usize,
    pub radius: f32,
    // set on update
    pub applying_force_freq: f32,
    pub applying_force_factor: f32,
    pub equilibrium_force_factor: f32,
}

impl Default for LongitudinalWave3dSimulationParameters {
    fn default() -> Self {
        Self {
            dimx: 10,
            dimy: 4,
            dimz: 10,
            radius: 0.4,
            applying_force_freq: 3.7,
            applying_force_factor: 0.6,
            equilibrium_force_factor: 6.0,
        }
    }
}

pub struct LongitudinalWave3dSimulationPlugin;

impl Plugin for LongitudinalWave3dSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvents>()
            .add_plugin(SimulationPlugin)
            .add_plugin(AnimationPlugin)
            .insert_resource(LongitudinalWave3dSimulationParameters::default());
    }
}
