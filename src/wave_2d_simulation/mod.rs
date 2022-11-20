use std::collections::VecDeque;

use bevy::prelude::*;
use ndarray::Array3;

mod animation_plugin;
mod finite_difference;
mod simulation_plugin;
mod ui;

use animation_plugin::AnimationPlugin;
use simulation_plugin::SimulationPlugin;
pub use ui::{show_ui, UiEvents};

#[derive(Default, Resource)]
pub struct Wave2dSimulationGrid(Array3<f32>);

#[derive(Resource)]
pub struct Wave2dSimulationParameters {
    // set on initialization
    dimx: usize,
    dimy: usize,
    cellsize: f32,
    boundary_size: usize,
    pub apply_force: bool,
    pub max_amplitude: f32,
    pub max_amplitude_avg: VecDeque<f32>,

    // set on update
    pub syntetic_energy_loss_fraction: f32,
    pub applied_force_frequency_hz: f32,
    pub wave_velocity: f32,
}

impl Default for Wave2dSimulationParameters {
    fn default() -> Self {
        Self {
            dimx: 160 * 2,
            dimy: 90 * 2,
            cellsize: 2.7,
            boundary_size: 4,
            apply_force: false,
            max_amplitude: 1.0,
            max_amplitude_avg: VecDeque::from(vec![0.0; 27]),

            syntetic_energy_loss_fraction: 0.99,
            applied_force_frequency_hz: 4.0,
            wave_velocity: 0.27,
        }
    }
}

pub struct Wave2dSimulationPlugin;

impl Plugin for Wave2dSimulationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_event::<UiEvents>()
            .add_plugin(SimulationPlugin)
            .add_plugin(AnimationPlugin)
            .insert_resource(Wave2dSimulationParameters::default());
    }
}
