use bevy::prelude::*;
use bevy_egui::egui;

use crate::AppState;

use super::LongitudinalWave3dSimulationParameters;

pub enum UiEvents {
    StartStop,
    Reset,
}

pub fn show_ui(
    ui: &mut egui::Ui,
    _app_state: &mut State<AppState>,
    parameters: &mut LongitudinalWave3dSimulationParameters,
    mut ui_events: EventWriter<UiEvents>,
) {
    ui.allocate_space(egui::Vec2::new(1.0, 10.0));

    ui.add(
        egui::Slider::new(&mut parameters.applying_force_freq, 0.0..=10.0)
            .text("applying force frequency in Hz"),
    );

    ui.add(
        egui::Slider::new(&mut parameters.applying_force_factor, 0.0..=2.0)
            .step_by(0.01)
            .text("applying force factor"),
    );

    ui.add(
        egui::Slider::new(
            &mut parameters.equilibrium_force_factor,
            0.0..=1000.0,
        )
        .text("equilibrium force factor"),
    );

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Start/Stop").clicked() {
            ui_events.send(UiEvents::StartStop);
        }
        if ui.button("Reset values").clicked() {
            *parameters = LongitudinalWave3dSimulationParameters::default();
        }
        if ui.button("Reset particles").clicked() {
            ui_events.send(UiEvents::Reset);
        }
    });
}
