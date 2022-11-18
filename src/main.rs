use std::time::Duration;

use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::prelude::*;
use bevy::window::PresentMode;
use bevy_inspector_egui::widgets::InspectableButton;
use bevy_inspector_egui::{Inspectable, InspectorPlugin};
use ndarray::{s, Array3};

mod animation_plugin;
mod colored_mesh;
mod finite_difference;
mod simulation_plugin;

use animation_plugin::AnimationPlugin;
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
    #[inspectable(read_only)]
    pub spatial_step_width: f32,
    #[inspectable(read_only)]
    pub time_step_width: f32,
    #[inspectable(read_only)]
    pub dimx: usize,
    #[inspectable(read_only)]
    pub dimy: usize,
    #[inspectable(read_only)]
    pub cellsize: f32,
    #[inspectable(read_only)]
    pub wave_period: u64,
    #[inspectable(read_only)]
    pub wave_velocity: f32,
    #[inspectable(read_only)]
    pub boundary_size: usize,

    pub use_absorbing_boundary: bool,
    #[inspectable(
        label = "amplitude of applied force",
        min = 0.1,
        max = 1000.0
    )]
    pub applied_force_amplitude: f32,
    #[inspectable(label = "frequency of applied force in Hz", min = 0.0)]
    pub applied_force_frequency_hz: f32,
    #[inspectable(label = "start / restart applied force", text = "Start")]
    pub applied_force_start_restart: InspectableButton<CommandStart>,
    #[inspectable(label = "stop applied force", text = "Stop")]
    pub applied_force_stop: InspectableButton<CommandStop>,
    #[inspectable(label = "reset all forces", text = "Clear")]
    pub clear_all_forces: InspectableButton<CommandClear>,
}

impl Default for SimulationParameters {
    fn default() -> Self {
        Self {
            spatial_step_width: 1.0,
            time_step_width: 1.0,
            dimx: 160 * 2,
            dimy: 90 * 2,
            cellsize: 4.0,
            wave_period: 1,
            wave_velocity: 0.3,
            boundary_size: 4,

            use_absorbing_boundary: false,
            applied_force_amplitude: 5.0,
            applied_force_frequency_hz: 4.0,
            applied_force_start_restart:
                InspectableButton::<CommandStart>::default(),
            applied_force_stop: InspectableButton::<CommandStop>::default(),
            clear_all_forces: InspectableButton::<CommandClear>::default(),
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
                ..default()
            },
            ..default()
        }))
        .add_plugin(InspectorPlugin::<SimulationParameters>::new())
        .add_plugin(SimulationPlugin)
        .add_plugin(AnimationPlugin)
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .insert_resource(DebugTimer(Timer::new(
            Duration::from_secs(1),
            TimerMode::Repeating,
        )))
        .add_startup_system(init_camera)
        .add_system(debug_system)
        .run();
}

fn init_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
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
