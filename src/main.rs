use std::time::Duration;

use bevy::prelude::*;

mod animation_plugin;
mod finite_difference;
mod simulation_plugin;

use bevy::window::PresentMode;

use animation_plugin::AnimationPlugin;
use ndarray::{s, Array3};
use simulation_plugin::SimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

pub const HS: f32 = 1.0; // spatial step width
pub const TS: f32 = 1.0; // time step width
pub const DIMX: usize = 16 * 20;
pub const DIMY: usize = 9 * 20;
pub const CELLSIZE: f32 = 0.2;

pub const C: f32 = 0.5;
pub const BOUNDARY: usize = 4;
pub const ABSORBING_BOUNDARY: bool = false;

pub const FRAMES: u64 = 36;
pub const PERIOD: u64 = 1;
pub const AMPLITUDE: f32 = 60.0;

#[derive(Resource)]
pub struct SimulationGrid(Array3<f32>);

impl SimulationGrid {
    pub fn init() -> Self {
        Self(Array3::zeros((3, DIMX, DIMY)))
    }
}

#[derive(Resource)]
struct DebugTimer(Timer);

fn main() {
    let height = 900.0;

    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                height,
                width: height * RESOLUTION,
                title: "Wave Sim".to_string(),
                present_mode: PresentMode::AutoVsync,
                resizable: false,
                ..default()
            },
            ..default()
        }))
        .add_plugin(SimulationPlugin)
        .add_plugin(AnimationPlugin)
        .insert_resource(DebugTimer(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .add_system(debug_system)
        .run();
}

fn debug_system(
    time: Res<Time>,
    mut timer: ResMut<DebugTimer>,
    u: Res<SimulationGrid>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let size = u.0.slice(s![0, .., ..]).len();
        let (max_u, min_u) =
            u.0.slice(s![0, .., ..])
                .fold((f32::MIN, f32::MAX), |(acc_max, acc_min), u| {
                    (u.max(acc_max), u.min(acc_min))
                });

        info!("GRID ITEM COUNT: {}", size);
        info!("MAXIMUM U: {} MINIMUM: {}", max_u, min_u);
    }
}
