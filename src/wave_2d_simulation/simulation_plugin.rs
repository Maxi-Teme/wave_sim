use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::time::Stopwatch;
use ndarray::prelude::*;
use ndarray::Zip;

use crate::AppState;

use super::animation_plugin::PlotClickedEvent;
use super::finite_difference::update_with_laplace_operator;
use super::Wave2dSimulationGrid;
use super::Wave2dSimulationParameters;

#[derive(Resource)]
struct ApplyingForceTimer(Stopwatch);

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Wave2dSimulationGrid::default())
            .insert_resource(ApplyingForceTimer(Stopwatch::new()))
            .add_system_set(
                SystemSet::on_enter(AppState::Wave2dSimulation)
                    .with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Wave2dSimulation)
                    .with_system(apply_force)
                    .with_system(update_wave)
                    .with_system(on_mouseclick),
            );
    }
}

fn setup(
    mut u: ResMut<Wave2dSimulationGrid>,
    parameters: Res<Wave2dSimulationParameters>,
) {
    u.0 = Array3::zeros((3, parameters.dimx, parameters.dimy));
}

fn apply_force(
    time: Res<Time>,
    mut applying_force_timer: ResMut<ApplyingForceTimer>,
    mut u: ResMut<Wave2dSimulationGrid>,
    parameters: Res<Wave2dSimulationParameters>,
) {
    if !parameters.apply_force {
        return;
    }

    let elapsed = applying_force_timer.0.elapsed();
    let amplitude =
        (elapsed.as_secs_f32() * parameters.applied_force_frequency_hz * TAU)
            .sin();

    let init_x = 4 * parameters.dimx / 6;
    let init_y = 4 * parameters.dimy / 6;

    *u.0.get_mut((0, init_x, init_y)).unwrap() = amplitude;

    applying_force_timer.0.tick(time.delta());
}

fn on_mouseclick(
    mut u: ResMut<Wave2dSimulationGrid>,
    parameters: Res<Wave2dSimulationParameters>,
    mut plot_clicked_events: EventReader<PlotClickedEvent>,
) {
    for event in plot_clicked_events.iter() {
        let event_x: usize = event.x.round() as usize;
        let event_y: usize = event.y.round() as usize;

        if 0 < event_x
            && event_x < parameters.dimx
            && 0 < event_y
            && event_y < parameters.dimy
        {
            *u.0.get_mut((0, event_x, event_y)).unwrap() = 1.0;
        }
    }
}

fn update_wave(
    time: Res<Time>,
    mut u: ResMut<Wave2dSimulationGrid>,
    parameters: Res<Wave2dSimulationParameters>,
) {
    if time.is_paused() {
        return;
    }

    let (u_2, mut u_1, u_0) =
        u.0.multi_slice_mut((s![2, .., ..], s![1, .., ..], s![0, .., ..]));

    Zip::from(u_2).and(&mut u_1).for_each(std::mem::swap);

    Zip::from(u_1).and(u_0).for_each(std::mem::swap);

    let tau = get_tau(&parameters);

    let new_u = update_with_laplace_operator(
        parameters.dimx,
        parameters.dimy,
        tau,
        &u.0,
    );

    u.0.slice_mut(s![
        0,
        parameters.boundary_size..(parameters.dimx - parameters.boundary_size),
        parameters.boundary_size..(parameters.dimy - parameters.boundary_size)
    ])
    .assign(&new_u);

    u.0.mapv_inplace(|u| u * parameters.syntetic_energy_loss_fraction);
}

fn get_tau(parameters: &Wave2dSimulationParameters) -> Array2<f32> {
    Array::from_elem(
        (parameters.dimx, parameters.dimy),
        parameters.wave_velocity,
    )
}
