use bevy::prelude::*;

mod animation_plugin;
mod simulation_plugin;

pub use animation_plugin::AnimationPlugin;
pub use simulation_plugin::SimulationPlugin;

pub struct Particle3dSimulationPlugin;

impl Plugin for Particle3dSimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimulationPlugin).add_plugin(AnimationPlugin);
    }
}
