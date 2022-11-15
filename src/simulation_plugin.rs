// curtesy of https://beltoforion.de/en/recreational_mathematics/2d-wave-equation.php

use std::time::Duration;

use bevy::prelude::*;
use bevy::sprite::Mesh2dHandle;
use ndarray::prelude::*;
use ndarray::Zip;

use crate::finite_difference::sigmoid;
use crate::finite_difference::update_with_absorbing_boundary;
use crate::finite_difference::{
    update_with_laplace_operator_1, update_with_laplace_operator_4,
};

const HS: f32 = 1.0; // spatial step width
const TS: f32 = 1.0; // time step width
const DIMX: usize = 160;
const DIMY: usize = 90;
const CELLSIZE: f32 = DIMY as f32 / 10.0;
const GAP: f32 = 0.4;

const C: f32 = 0.4;
const TICK: u64 = 27;

const BOUNDARY: usize = 4;
const ABSORBING_BOUNDARY: bool = true;

#[derive(Resource)]
struct SimulationGrid(Array3<f32>);

impl SimulationGrid {
    pub fn init() -> Self {
        Self(Array::zeros((3, DIMX, DIMY)))
    }
}

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

#[derive(Component)]
struct Tile {
    sim_x: usize,
    sim_y: usize,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(init_wave)
            .add_startup_system(init_tiles)
            .insert_resource(SimulationGrid::init())
            .insert_resource(Tau::init())
            .insert_resource(Kappa::init())
            .insert_resource(SimulationTimer(Timer::new(
                Duration::from_millis(TICK),
                TimerMode::Repeating,
            )))
            .add_system(update_wave)
            .add_system(update_tiles)
            .add_system(debug_system);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

fn init_wave(mut u: ResMut<SimulationGrid>) {
    let init_x = (DIMX / 2).try_into().unwrap();
    let init_y = (DIMY / 2).try_into().unwrap();

    *u.0.get_mut((0, init_x, init_y)).unwrap() = 120.0;
    *u.0.get_mut((0, 27, 27)).unwrap() = 270.0;
}

fn init_tiles(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    commands.spawn(Camera2dBundle::default());

    let mesh = meshes.add(Mesh::from(shape::Quad::default()));

    for x in 0..DIMX {
        for y in 0..DIMY {
            let map_x: i64 =
                (x as i32 - (DIMX as i32 / 2) - 1).try_into().unwrap();
            let map_y: i64 =
                (y as i32 - (DIMY as i32 / 2) - 1).try_into().unwrap();

            let tile_x = (CELLSIZE + GAP) * (map_x as f32 - 0.5);
            let tile_y = (CELLSIZE + GAP) * (map_y as f32 - 0.5);

            let transform =
                Transform::from_xyz(tile_x as f32, tile_y as f32, 0.0)
                    .with_scale(Vec3::splat(CELLSIZE));

            let material = materials.add(ColorMaterial::from(Color::WHITE));

            commands.spawn((
                Tile { sim_x: x, sim_y: y },
                Mesh2dHandle::from(mesh.clone()),
                material,
                transform,
                GlobalTransform::default(),
                Visibility::default(),
                ComputedVisibility::default(),
            ));
        }
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

fn update_tiles(
    mut materials: ResMut<Assets<ColorMaterial>>,
    u: Res<SimulationGrid>,
    tiles: Query<(&Tile, &Handle<ColorMaterial>)>,
) {
    for (tile, material_handle) in tiles.iter() {
        let amplitude = u.0.get((0, tile.sim_x, tile.sim_y)).unwrap();
        let r = sigmoid(amplitude, 8.0);

        materials.get_mut(material_handle).unwrap().color =
            Color::rgb(r, 0.0, 1.0);
    }
}

fn debug_system(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    u: Res<SimulationGrid>,
    tiles: Query<&Tile>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let (max_u, min_u) =
            u.0.slice(s![0, .., ..])
                .fold((f32::MIN, f32::MAX), |(acc_max, acc_min), u| {
                    (u.max(acc_max), u.min(acc_min))
                });

        info!("TILES COUNT: {}", tiles.iter().len());
        info!("GRID ITEM COUNT: {}", u.0.len());
        info!("MAXIMUM U: {} MINIMUM: {}", max_u, min_u);
    }
}
