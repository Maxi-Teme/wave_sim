use std::collections::VecDeque;

use bevy::diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin};
use bevy::prelude::*;
use bevy_egui::{egui, EguiContext, EguiPlugin};
use bevy_rapier3d::render::DebugRenderContext;

use crate::longitudinal_wave_3d_simulation::LongitudinalWave3dSimulationParameters;
use crate::particle_mess::ParticleMessParameters;
use crate::wave_2d_simulation::Wave2dSimulationParameters;
use crate::wave_in_panel::WaveInPanelParameters;
use crate::{
    longitudinal_wave_3d_simulation, wave_2d_simulation, wave_in_panel, particle_mess,
    AppState,
};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource(UiState::default())
            .add_startup_system(configure_ui)
            .add_system(show_ui);
    }
}

#[derive(Resource)]
pub struct UiState {
    fps_avg: VecDeque<f64>,
    pub panel_x: f32,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            fps_avg: VecDeque::from(vec![0.0; 27]),
            panel_x: 350.0,
        }
    }
}

fn configure_ui(mut egui_ctx: ResMut<EguiContext>) {
    egui_ctx.ctx_mut().set_visuals(egui::Visuals {
        window_rounding: 0.0.into(),
        ..Default::default()
    });
}

#[allow(clippy::too_many_arguments)]
fn show_ui(
    mut egui_ctx: ResMut<EguiContext>,
    mut ui_state: ResMut<UiState>,
    mut app_state: ResMut<State<AppState>>,
    diagnostics: Res<Diagnostics>,
    mut rapier_debug_config: ResMut<DebugRenderContext>,
    mut wave_2d_parameters: ResMut<Wave2dSimulationParameters>,
    wave_2d_events: EventWriter<wave_2d_simulation::UiEvents>,
    mut longitudinal_wave_3d_parameters: ResMut<
        LongitudinalWave3dSimulationParameters,
    >,
    longitudinal_wave_3d_events: EventWriter<
        longitudinal_wave_3d_simulation::UiEvents,
    >,
    mut particle_mess_parameters: ResMut<ParticleMessParameters>,
    particle_mess_events: EventWriter<particle_mess::UiEvents>,
    mut wave_in_panel_parameters: ResMut<WaveInPanelParameters>,
    wave_in_panel_events: EventWriter<wave_in_panel::UiEvents>,
) {
    egui::TopBottomPanel::top("top_panel")
        .resizable(false)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.with_layout(
                egui::Layout::right_to_left(egui::Align::Center),
                |ui| {
                    ui.heading("wave_sim");
                    ui.allocate_space(egui::Vec2::new(0.0, 27.0));
                },
            );
        });

    egui::SidePanel::left("side_panel")
        .default_width(200.0)
        .resizable(true)
        .show(egui_ctx.ctx_mut(), |ui| {
            ui.allocate_space(egui::Vec2::new(1.0, 20.0));

            // simulation selection
            select_simulation(ui, &mut app_state);

            ui.separator();

            // simulation parameter
            match app_state.current() {
                AppState::Wave2dSimulation => {
                    wave_2d_simulation::show_ui(
                        ui,
                        &mut ui_state,
                        &mut app_state,
                        &mut wave_2d_parameters,
                        wave_2d_events,
                    );
                }
                AppState::LongitudinalWaveSimulation3d => {
                    longitudinal_wave_3d_simulation::show_ui(
                        ui,
                        &mut app_state,
                        &mut longitudinal_wave_3d_parameters,
                        longitudinal_wave_3d_events,
                        &mut rapier_debug_config,
                    );
                }
                AppState::ParticleMess => {
                    particle_mess::show_ui(
                        ui,
                        &mut rapier_debug_config,
                        particle_mess_events,
                        &mut particle_mess_parameters,
                    );
                }
                AppState::WaveInPanel => {
                    wave_in_panel::show_ui(
                        ui,
                        &mut rapier_debug_config,
                        wave_in_panel_events,
                        &mut wave_in_panel_parameters,
                    );
                }
            }

            // debug info
            show_debug(ui, &diagnostics, &mut ui_state);
        });
}

fn select_simulation(ui: &mut egui::Ui, app_state: &mut State<AppState>) {
    ui.heading("Simulations: ");
    let mut current_state = app_state.current().clone();
    egui::ComboBox::from_id_source("simulation_selection")
        .selected_text(format!("{:?}", current_state))
        .show_ui(ui, |ui| {
            ui.selectable_value(
                &mut current_state,
                AppState::Wave2dSimulation,
                String::from(AppState::Wave2dSimulation),
            );
            ui.selectable_value(
                &mut current_state,
                AppState::LongitudinalWaveSimulation3d,
                String::from(AppState::LongitudinalWaveSimulation3d),
            );
            ui.selectable_value(
                &mut current_state,
                AppState::ParticleMess,
                String::from(AppState::ParticleMess),
            );
            ui.selectable_value(
                &mut current_state,
                AppState::WaveInPanel,
                String::from(AppState::WaveInPanel),
            );
        });
    if current_state != *app_state.current() {
        app_state.set(current_state).unwrap();
    }
}

fn show_debug(
    ui: &mut egui::Ui,
    diagnostics: &Diagnostics,
    ui_state: &mut UiState,
) {
    ui.with_layout(egui::Layout::bottom_up(egui::Align::Center), |ui| {
        if let Some(fps) =
            diagnostics.get_measurement(FrameTimeDiagnosticsPlugin::FPS)
        {
            ui_state.fps_avg.pop_back();
            ui_state.fps_avg.push_front(fps.value);

            let avg: f64 = ui_state.fps_avg.iter().sum::<f64>()
                / ui_state.fps_avg.len() as f64;

            ui.label(format!(
                "fps: {}   avg: {}",
                fps.value.round(),
                avg.round()
            ));
        } else {
            ui.label("fps: No value available");
        }
    });
}
