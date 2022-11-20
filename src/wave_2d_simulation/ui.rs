use bevy::prelude::*;
use bevy_egui::egui;

use crate::ui::UiState;
use crate::AppState;

use super::Wave2dSimulationParameters;

pub enum UiEvents {
    StartStopTime,
    Reset,
}

pub fn show_ui(
    ui: &mut egui::Ui,
    _ui_state: &mut UiState,
    _app_state: &mut State<AppState>,
    parameters: &mut Wave2dSimulationParameters,
    mut ui_events: EventWriter<UiEvents>,
) {
    ui.allocate_space(egui::Vec2::new(1.0, 10.0));

    ui.add(
        egui::Slider::new(
            &mut parameters.syntetic_energy_loss_fraction,
            0.8..=1.0,
        )
        .step_by(0.001)
        .text("energy loss fraction"),
    );

    ui.add(
        egui::Slider::new(&mut parameters.wave_velocity, 0.00..=0.4)
            .step_by(0.001)
            .text("wave velocity"),
    );

    ui.add(
        egui::Slider::new(
            &mut parameters.applied_force_frequency_hz,
            0.0..=100.0,
        )
        .step_by(0.01)
        .text("frequency in Hz of applying force"),
    );

    ui.add(egui::Checkbox::new(
        &mut parameters.apply_force,
        "continuously apply frequency",
    ));

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Start/Stop time").clicked() {
            ui_events.send(UiEvents::StartStopTime);
        }
        if ui.button("Reset values").clicked() {
            *parameters = Wave2dSimulationParameters::default();
        }
        if ui.button("Reset waves").clicked() {
            ui_events.send(UiEvents::Reset);
        }
    });

    ui.separator();

    ui.label(format!("max amplitude: {}", parameters.max_amplitude));
}
