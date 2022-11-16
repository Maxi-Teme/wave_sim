use std::time::Duration;

use bevy::prelude::*;

mod animation_plugin;
mod finite_difference;
mod simulation_plugin;

use bevy::window::PresentMode;

use animation_plugin::AnimationPlugin;
use bevy_inspector_egui::widgets::InspectableButton;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use ndarray::{s, Array3};
use simulation_plugin::SimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

#[derive(Default, Component)]
pub struct CommandStart;

#[derive(Default, Component)]
pub struct CommandStop;

#[derive(Default, Component)]
pub struct CommandClear;

#[derive(Resource, Inspectable)]
pub struct SimulationParameters {
    pub frames_per_second: u64,
    pub spatial_step_width: f32,
    pub time_step_width: f32,
    #[inspectable(min = 16, max = 1600)]
    pub dimx: usize,
    #[inspectable(min = 9, max = 900)]
    pub dimy: usize,
    #[inspectable(min = 0.01, max = 1.0)]
    pub cellsize: f32,
    pub wave_period: u64,
    pub wave_velocity: f32,
    pub boundary_size: usize,
    pub use_absorbing_boundary: bool,
    pub applying_force_amplitude: f32,
    #[inspectable(label = "", text = "Start")]
    pub command_start: InspectableButton<CommandStart>,
    #[inspectable(label = "", text = "Stop")]
    pub command_stop: InspectableButton<CommandStop>,
    #[inspectable(label = "", text = "Clear")]
    pub command_clear: InspectableButton<CommandClear>,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            frames_per_second: 36,
            spatial_step_width: 1.0,
            time_step_width: 1.0,
            dimx: 320,
            dimy: 180,
            cellsize: 0.2,
            wave_period: 1,
            wave_velocity: 0.5,
            boundary_size: 1,
            use_absorbing_boundary: false,
            applying_force_amplitude: 10.0,
            command_start: InspectableButton::<CommandStart>::default(),
            command_stop: InspectableButton::<CommandStop>::default(),
            command_clear: InspectableButton::<CommandClear>::default(),
        }
    }
}

#[derive(Default, Resource)]
pub struct SimulationGrid(Array3<f32>);

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
        .add_plugin(InspectorPlugin::<SimulationParameters>::new())
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
