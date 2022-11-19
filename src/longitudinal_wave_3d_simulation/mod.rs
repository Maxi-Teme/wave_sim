use bevy::prelude::*;

mod animation_plugin;
mod simulation_plugin;

pub use animation_plugin::AnimationPlugin;
pub use simulation_plugin::SimulationPlugin;

#[derive(Resource)]
pub struct SimulationParameters {
    pub dimx: usize,
    pub dimy: usize,
    pub dimz: usize,
    pub radius: f32,
    pub applying_force_freq: f32,
    applying_force_factor: f32,
    pub equilibrium_force_factor: f32,
}

impl Default for SimulationParameters {
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

pub struct LongitudinalWaveSimulation3dPlugin;

impl Plugin for LongitudinalWaveSimulation3dPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimulationPlugin)
            .add_plugin(AnimationPlugin)
            .insert_resource(SimulationParameters::default());
    }
}
