// curtesy of https://beltoforion.de/en/recreational_mathematics/2d-wave-equation.php

use std::time::Duration;

use bevy::math::ivec3;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;
use bevy_simple_tilemap::prelude::SimpleTileMapPlugin;
use bevy_simple_tilemap::prelude::TileMapBundle;
use bevy_simple_tilemap::Tile;
use bevy_simple_tilemap::TileMap;
use ndarray::prelude::*;
use ndarray::Zip;

use crate::finite_difference::sigmoid;
use crate::finite_difference::update_with_absorbing_boundary;
use crate::finite_difference::{
    update_with_laplace_operator_1, update_with_laplace_operator_4,
};

const HS: f32 = 1.0; // spatial step width
const TS: f32 = 1.0; // time step width
const DIMX: usize = 16 * 20;
const DIMY: usize = 9 * 20;
const CELLSIZE: f32 = 0.2;

const C: f32 = 0.5;
const BOUNDARY: usize = 4;
const ABSORBING_BOUNDARY: bool = false;

const FRAMES: u64 = 27;
const PERIOD: u64 = 1;
const AMPLITUDE: f32 = 60.0;

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

#[derive(Resource)]
struct FrequencyTimer(Timer);

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(SimpleTileMapPlugin)
            .add_startup_system(init_camera)
            .add_startup_system(init_tiles)
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
            .add_system(update_wave)
            .add_system(update_tiles)
            .add_system(debug_system);
    }

    fn name(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

fn init_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();

    camera_bundle.projection = OrthographicProjection {
        window_origin: WindowOrigin::BottomLeft,
        ..default()
    };

    commands.spawn(camera_bundle);
}

fn init_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/tilesheet.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        vec2(16.0, 16.0),
        4,
        1,
        Some(vec2(1.0, 1.0)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut tiles = Vec::new();

    for x in 0..DIMX {
        for y in 0..DIMY {
            tiles.push((
                ivec3(x.try_into().unwrap(), y.try_into().unwrap(), 0),
                Some(Tile {
                    sprite_index: 3,
                    color: Color::WHITE,
                    ..Default::default()
                }),
            ));
        }
    }

    let mut tilemap = TileMap::default();
    tilemap.set_tiles(tiles);

    let tilemap_bundle = TileMapBundle {
        tilemap,
        texture_atlas: texture_atlas_handle,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::splat(CELLSIZE),
            ..default()
        },
        ..default()
    };

    commands.spawn(tilemap_bundle);
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

fn update_tiles(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    u: Res<SimulationGrid>,
    mut tilemaps: Query<&mut TileMap>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut tilemap = tilemaps.get_single_mut().unwrap();
        tilemap.clear();

        let mut tiles = Vec::new();

        for x in 0..DIMX {
            for y in 0..DIMY {
                let amplitude = u.0.get((0, x, y)).unwrap();
                let r = sigmoid(amplitude, 0.8);

                tiles.push((
                    ivec3(x.try_into().unwrap(), y.try_into().unwrap(), 0),
                    Some(Tile {
                        sprite_index: 3,
                        color: Color::rgb(r, 0.0, 1.0),
                        ..Default::default()
                    }),
                ));
            }
        }

        tilemap.set_tiles(tiles);
    }
}

fn debug_system(
    time: Res<Time>,
    mut timer: ResMut<SimulationTimer>,
    u: Res<SimulationGrid>,
    tilemaps: Query<&TileMap>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let tilemap = tilemaps.get_single().unwrap();

        let (max_u, min_u) =
            u.0.slice(s![0, .., ..])
                .fold((f32::MIN, f32::MAX), |(acc_max, acc_min), u| {
                    (u.max(acc_max), u.min(acc_min))
                });

        info!("TILES COUNT: {}", tilemap.chunks.iter().len());
        info!("GRID ITEM COUNT: {}", u.0.len());
        info!("MAXIMUM U: {} MINIMUM: {}", max_u, min_u);
    }
}
