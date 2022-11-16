// curtesy of https://beltoforion.de/en/recreational_mathematics/2d-wave-equation.php

use std::time::Duration;

use bevy::prelude::*;
use bevy_simple_tilemap::prelude::SimpleTileMapPlugin;
use ndarray::prelude::*;
use ndarray::Zip;

use crate::SimulationGrid;
use crate::finite_difference::update_with_absorbing_boundary;
use crate::finite_difference::{
    update_with_laplace_operator_1, update_with_laplace_operator_4,
};
use crate::ABSORBING_BOUNDARY;
use crate::AMPLITUDE;
use crate::BOUNDARY;

use crate::C;
use crate::DIMX;
use crate::DIMY;
use crate::FRAMES;
use crate::HS;
use crate::PERIOD;
use crate::TS;


/// A field containing the factor for the Laplace Operator that
/// combines Velocity and Grid Constants for the `Wave Equation`
#[derive(Resource)]
struct Tau(Array2<f32>);

impl Tau {
    pub fn init() -> Self {
        Self(Array::from_elem((DIMX, DIMY), ((C * TS) / HS).powi(2)))
    }
}

/// A field containing the factor for the Laplace Operator that
/// combines Velocity and Grid Constants for the `Boundary Condition`
#[derive(Resource)]
struct Kappa(Array2<f32>);

impl Kappa {
    pub fn init() -> Self {
        Self(Array2::from_elem((DIMX, DIMY), TS * C / HS))
    }
}

#[derive(Resource)]
struct SimulationTimer(Timer);

#[derive(Resource)]
struct FrequencyTimer(Timer);

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimpleTileMapPlugin)
            .insert_resource(SimulationGrid::init())
            .insert_resource(Tau::init())
            .insert_resource(Kappa::init())
            .insert_resource(SimulationTimer(Timer::new(
                Duration::from_millis(FRAMES),
                TimerMode::Repeating,
            )))
            .insert_resource(FrequencyTimer(Timer::new(
                Duration::from_secs(PERIOD),
                TimerMode::Repeating,
            )))
            .add_system(apply_force)
            .add_system(update_wave);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }

    fn is_unique(&self) -> bool {
        true
    }
}

fn apply_force(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut u: ResMut<SimulationGrid>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let init_x = (4 * DIMX / 6).try_into().unwrap();
        let init_y = (4 * DIMY / 6).try_into().unwrap();

        *u.0.get_mut((0, init_x, init_y)).unwrap() = AMPLITUDE;
    }
}

fn update_wave(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    mut u: ResMut<SimulationGrid>,
    tau: Res<Tau>,
    kappa: Res<Kappa>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let (u_2, mut u_1, u_0) =
            u.0.multi_slice_mut((s![2, .., ..], s![1, .., ..], s![0, .., ..]));

        Zip::from(u_2)
            .and(&mut u_1)
            .for_each(|u_2, u_1| std::mem::swap(u_2, u_1));

        Zip::from(u_1)
            .and(u_0)
            .for_each(|u_1, u_0| std::mem::swap(u_1, u_0));

        let new_u = if BOUNDARY == 1 {
            update_with_laplace_operator_1(DIMX, DIMY, &tau.0, &u.0)
        } else if BOUNDARY == 4 {
            update_with_laplace_operator_4(DIMX, DIMY, &tau.0, &u.0)
        } else {
            todo!("BOUNDARY of size {}", BOUNDARY);
        };

        u.0.slice_mut(s![
            0,
            BOUNDARY..(DIMX - BOUNDARY),
            BOUNDARY..(DIMY - BOUNDARY)
        ])
        .assign(&new_u);

        if ABSORBING_BOUNDARY {
            update_with_absorbing_boundary(
                DIMX, DIMY, BOUNDARY, &kappa.0, &mut u.0,
            );
        } else {
            u.0.mapv_inplace(|u| u * 0.995);
        }
    }
}
