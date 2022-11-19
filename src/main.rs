use bevy::diagnostic::FrameTimeDiagnosticsPlugin;
use bevy::prelude::*;
use bevy::window::PresentMode;

mod colored_mesh;
mod wave_2d_simulation;

use wave_2d_simulation::Wave2dSimulationPlugin;

pub const RESOLUTION: f32 = 16.0 / 9.0;

const UI_BACKGROUND_COLOR: Color = Color::rgba(0.10, 0.10, 0.10, 0.9);
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);

const SIMULATIONS: [&str; 2] = ["wave_2d_simulation", "particle_3d_simulation"];

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AppState {
    Wave2dSimulation,
    Particle3dSimulation,
}

fn main() {
    let height = 900.0;

    App::new()
        // main plugins
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            window: WindowDescriptor {
                height,
                width: height * RESOLUTION,
                title: "wave_sim".to_string(),
                present_mode: PresentMode::AutoVsync,
                ..default()
            },
            ..default()
        }))
        .insert_resource(Msaa { samples: 1 })
        .add_state(AppState::Wave2dSimulation)
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        // simulation plugins
        .add_plugin(Wave2dSimulationPlugin)
        // main systems
        .add_startup_system(setup)
        .add_system(handle_simulation_selection_buttons)
        .run();
}

fn setup(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle::default());

    let default_medium_font = asset_server.load("fonts/FiraMono-Medium.ttf");
    let default_bold_font = asset_server.load("fonts/FiraSans-Bold.ttf");

    commands
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // side panel right
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(Val::Px(300.0), Val::Percent(100.0)),
                        border: UiRect::all(Val::Px(1.0)),
                        flex_direction: FlexDirection::Column,
                        justify_content: JustifyContent::FlexStart,
                        ..default()
                    },
                    background_color: UI_BACKGROUND_COLOR.into(),
                    ..default()
                })
                .with_children(|parent| {
                    // sidebar title
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0),
                                    Val::Px(20.0),
                                ),
                                margin: UiRect {
                                    bottom: Val::Px(50.0),
                                    ..default()
                                },
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(
                                TextBundle::from_section(
                                    "wave_sim",
                                    TextStyle {
                                        font: default_bold_font,
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_style(
                                    Style {
                                        margin: UiRect {
                                            left: Val::Px(10.0),
                                            top: Val::Px(10.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                ),
                            );
                        });

                    // sidebar items
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0),
                                    Val::Percent(100.0),
                                ),
                                flex_direction: FlexDirection::Column,
                                justify_content: JustifyContent::FlexStart,
                                ..default()
                            },
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(
                                TextBundle::from_section(
                                    "Simulations:",
                                    TextStyle {
                                        font: default_medium_font.clone(),
                                        font_size: 24.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_style(
                                    Style {
                                        margin: UiRect {
                                            left: Val::Px(14.0),
                                            bottom: Val::Px(4.0),
                                            ..default()
                                        },
                                        ..default()
                                    },
                                ),
                            );

                            parent.spawn(
                                NodeBundle {
                                    style: Style {
                                        size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                                        flex_direction: FlexDirection::Column,
                                        justify_content: JustifyContent::FlexStart,
                                        align_items: AlignItems::FlexStart,
                                        ..default()
                                    },
                                    ..default()
                                }
                            ).with_children(|parent| {
                                for simulation in SIMULATIONS {
                                    parent
                                        .spawn(ButtonBundle {
                                            style: Style {
                                                margin: UiRect::new(
                                                    Val::Auto,
                                                    Val::Auto,
                                                    Val::Undefined,
                                                    Val::Px(4.0),
                                                ),
                                                justify_content:
                                                    JustifyContent::Center,
                                                align_items: AlignItems::Center,
                                                ..default()
                                            },
                                            background_color: NORMAL_BUTTON
                                                .into(),
                                            ..default()
                                        })
                                        .with_children(|parent| {
                                            parent.spawn(
                                                TextBundle::from_section(
                                                    simulation,
                                                    TextStyle {
                                                        font: default_medium_font
                                                            .clone(),
                                                        font_size: 18.0,
                                                        color: Color::WHITE,
                                                    },
                                                )
                                                .with_style(Style {
                                                    margin: UiRect::all(Val::Px(10.0)),
                                                    ..default()
                                                }),
                                            );
                                        });
                                }
                            });
                        });
                });

            // rest of the window right
            parent
                .spawn(NodeBundle {
                    style: Style {
                        size: Size::new(
                            Val::Percent(100.0),
                            Val::Percent(100.0),
                        ),
                        ..default()
                    },
                    ..default()
                })
                .with_children(|parent| {
                    // header bar
                    parent.spawn(NodeBundle {
                        style: Style {
                            size: Size::new(Val::Percent(100.0), Val::Px(40.0)),
                            border: UiRect::all(Val::Px(1.0)),
                            ..default()
                        },
                        background_color: UI_BACKGROUND_COLOR.into(),
                        ..default()
                    });
                });
        });
}

#[allow(clippy::type_complexity)]
fn handle_simulation_selection_buttons(
    mut app_state: ResMut<State<AppState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<Button>),
    >,
    text_query: Query<&Text>,
) {
    for (interaction, mut color, children) in &mut interaction_query {
        match *interaction {
            Interaction::Hovered => {
                *color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *color = NORMAL_BUTTON.into();
            }
            Interaction::Clicked => {
                *color = NORMAL_BUTTON.into();

                let text = text_query.get(children[0]).unwrap();
                let clicked_simulation = text.sections[0].value.clone();

                if clicked_simulation == SIMULATIONS[0]
                    && *app_state.current() != AppState::Wave2dSimulation
                {
                    app_state.set(AppState::Wave2dSimulation).unwrap();
                }

                if clicked_simulation == SIMULATIONS[1]
                    && *app_state.current() != AppState::Particle3dSimulation
                {
                    app_state.set(AppState::Particle3dSimulation).unwrap();
                }
            }
        }
    }
}
