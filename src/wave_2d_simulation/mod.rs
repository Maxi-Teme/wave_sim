use bevy::prelude::*;
use ndarray::Array3;

mod animation_plugin;
mod finite_difference;
mod simulation_plugin;

use animation_plugin::AnimationPlugin;
use simulation_plugin::SimulationPlugin;

#[derive(Default, Resource)]
pub struct SimulationGrid(Array3<f32>);

#[derive(Resource)]
pub struct SimulationParameters {
    pub spatial_step_width: f32,
    pub time_step_width: f32,
    pub dimx: usize,
    pub dimy: usize,
    pub cellsize: f32,
    pub wave_period: u64,
    pub wave_velocity: f32,
    pub boundary_size: usize,
    pub use_absorbing_boundary: bool,
    pub applied_force_amplitude: f32,
    pub applied_force_frequency_hz: f32,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            spatial_step_width: 1.0,
            time_step_width: 1.0,
            dimx: 160 * 2,
            dimy: 90 * 2,
            cellsize: 2.7,
            wave_period: 1,
            wave_velocity: 0.4,
            boundary_size: 4,

            use_absorbing_boundary: false,
            applied_force_amplitude: 5.0,
            applied_force_frequency_hz: 4.0,
        }
    }
}

pub struct Wave2dSimulationPlugin;

impl Plugin for Wave2dSimulationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(SimulationPlugin)
            .add_plugin(AnimationPlugin)
            .insert_resource(SimulationParameters::default());
    }
}
