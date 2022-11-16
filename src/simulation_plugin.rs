// curtesy of https://beltoforion.de/en/recreational_mathematics/2d-wave-equation.php

use std::time::Duration;

use bevy::prelude::*;
use bevy_simple_tilemap::prelude::SimpleTileMapPlugin;
use ndarray::prelude::*;
use ndarray::Zip;

use crate::finite_difference::update_with_absorbing_boundary;
use crate::finite_difference::{
    update_with_laplace_operator_1, update_with_laplace_operator_4,
};
use crate::CommandClear;
use crate::CommandStart;
use crate::CommandStop;
use crate::SimulationGrid;
use crate::SimulationParameters;

/// A field containing the factor for the Laplace Operator that
/// combines Velocity and Grid Constants for the `Wave Equation`
#[derive(Default, Resource)]
struct Tau(Array2<f32>);

/// A field containing the factor for the Laplace Operator that
/// combines Velocity and Grid Constants for the `Boundary Condition`
#[derive(Default, Resource)]
struct Kappa(Array2<f32>);

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct FrequencyTimer(Timer);

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimpleTileMapPlugin)
            .insert_resource(SimulationGrid::default())
            .insert_resource(Tau::default())
            .insert_resource(Kappa::default())
            .insert_resource(SimulationTimer(Timer::default()))
            .insert_resource(FrequencyTimer(Timer::default()))
            .add_startup_system(init_resources)
            .add_system(apply_force)
            .add_system(update_wave)
            .add_system(command_event_handler);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn is_unique(&self) -> bool {
        true
    }
}

fn init_resources(
    mut tau: ResMut<Tau>,
    mut kappa: ResMut<Kappa>,
    mut u: ResMut<SimulationGrid>,
    parameters: Res<SimulationParameters>,
) {
    tau.0 = Array::from_elem(
        (parameters.dimx, parameters.dimy),
        ((parameters.wave_velocity * parameters.time_step_width)
            / parameters.spatial_step_width)
            .powi(2),
    );

    kappa.0 = Array2::from_elem(
        (parameters.dimx, parameters.dimy),
        parameters.time_step_width * parameters.wave_velocity
            / parameters.spatial_step_width,
    );

    u.0 = Array3::zeros((3, parameters.dimx, parameters.dimy));
}

fn apply_force(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut u: ResMut<SimulationGrid>,
    parameters: Res<SimulationParameters>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let init_x = (4 * parameters.dimx / 6).try_into().unwrap();
        let init_y = (4 * parameters.dimy / 6).try_into().unwrap();

        *u.0.get_mut((0, init_x, init_y)).unwrap() =
            parameters.applying_force_amplitude;
    }
}

fn update_wave(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut u: ResMut<SimulationGrid>,
    tau: Res<Tau>,
    kappa: Res<Kappa>,
    parameters: Res<SimulationParameters>,
) {
    let boundary_size = parameters.boundary_size;

    if timer.0.tick(time.delta()).just_finished() {
        let (u_2, mut u_1, u_0) =
            u.0.multi_slice_mut((s![2, .., ..], s![1, .., ..], s![0, .., ..]));

        Zip::from(u_2)
            .and(&mut u_1)
            .for_each(|u_2, u_1| std::mem::swap(u_2, u_1));

        Zip::from(u_1)
            .and(u_0)
            .for_each(|u_1, u_0| std::mem::swap(u_1, u_0));

        let new_u = if boundary_size == 1 {
            update_with_laplace_operator_1(
                parameters.dimx,
                parameters.dimy,
                &tau.0,
                &u.0,
            )
        } else if boundary_size == 4 {
            update_with_laplace_operator_4(
                parameters.dimx,
                parameters.dimy,
                &tau.0,
                &u.0,
            )
        } else {
            todo!("boundary_size of size {}", boundary_size);
        };

        u.0.slice_mut(s![
            0,
            boundary_size..(parameters.dimx - boundary_size),
            boundary_size..(parameters.dimy - boundary_size)
        ])
        .assign(&new_u);

        if parameters.use_absorbing_boundary {
            update_with_absorbing_boundary(
                parameters.dimx,
                parameters.dimy,
                boundary_size,
                &kappa.0,
                &mut u.0,
            );
        } else {
            u.0.mapv_inplace(|u| u * 0.995);
        }
    }
}

fn command_event_handler(
    mut start_events: EventReader<CommandStart>,
    mut stop_events: EventReader<CommandStop>,
    mut clear_events: EventReader<CommandClear>,
    parameters: Res<SimulationParameters>,
    mut u: ResMut<SimulationGrid>,
    mut simulation_timer: ResMut<SimulationTimer>,
    mut frequency_timer: ResMut<FrequencyTimer>,
) {
    for _ in start_events.iter() {
        simulation_timer
            .0
            .set_duration(Duration::from_millis(parameters.frames_per_second));
        simulation_timer.0.set_mode(TimerMode::Repeating);
        simulation_timer.0.unpause();

        frequency_timer
            .0
            .set_duration(Duration::from_secs(parameters.wave_period));
        frequency_timer.0.set_mode(TimerMode::Repeating);
        frequency_timer.0.unpause();
    }

    for _ in stop_events.iter() {
        simulation_timer.0.pause();
        simulation_timer.0.pause();
    }

    for _ in clear_events.iter() {
        u.0 = Array3::zeros((3, parameters.dimx, parameters.dimy));
    }
}
