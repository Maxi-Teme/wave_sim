use bevy::prelude::*;
use bevy::ui::FocusPolicy;

use crate::AppState;

#[derive(Component)]
struct SimulationSelectButton;

#[derive(Component)]
struct SimulationSelectionList;

#[derive(Component)]
struct SimulationSelectionListItem;

#[derive(Resource)]
struct UiParameters {
    background_color: BackgroundColor,
    default_medium_font: Handle<Font>,
    default_bold_font: Handle<Font>,
    arrow_up_icon: Handle<Image>,
    arrow_down_icon: Handle<Image>,
    button_color: BackgroundColor,
    hovered_button_color: BackgroundColor,
    simulations: Vec<String>,
}

impl UiParameters {
    fn new(
        default_medium_font: Handle<Font>,
        default_bold_font: Handle<Font>,
        arrow_up_icon: Handle<Image>,
        arrow_down_icon: Handle<Image>,
    ) -> Self {
        Self {
            background_color: Color::rgba(0.10, 0.10, 0.10, 0.9).into(),
            default_medium_font,
            default_bold_font,
            arrow_up_icon,
            arrow_down_icon,
            button_color: Color::rgb(0.15, 0.15, 0.15).into(),
            hovered_button_color: Color::rgb(0.25, 0.25, 0.25).into(),
            simulations: AppState::all_as_string(),
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        let asset_server = app.world.resource::<AssetServer>();

        let default_medium_font: Handle<Font> =
            asset_server.load("fonts/FiraMono-Medium.ttf");
        let default_bold_font: Handle<Font> =
            asset_server.load("fonts/FiraSans-Bold.ttf");
        let arrow_up_icon: Handle<Image> =
            asset_server.load("icons/arrow_up.png");
        let arrow_down_icon: Handle<Image> =
            asset_server.load("icons/arrow_down.png");

        app.insert_resource(UiParameters::new(
            default_medium_font,
            default_bold_font,
            arrow_up_icon,
            arrow_down_icon,
        ))
        .add_startup_system(setup)
        .add_system(handle_simulation_selection_button)
        .add_system(handle_simulation_selection_select);
    }
}

// ui

fn setup(
    mut commands: Commands,
    ui_parameters: Res<UiParameters>,
    app_state: Res<State<AppState>>,
) {
    commands
        .spawn(NodeBundle {
            transform: Transform::from_xyz(0.0, 0.0, 900.0),
            background_color: Color::NONE.into(),
            style: Style {
                size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                justify_content: JustifyContent::SpaceBetween,
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            // side panel right
            setup_side_panel(parent, &ui_parameters, &app_state);
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
                    parent
                        .spawn(NodeBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Percent(100.0),
                                    Val::Px(40.0),
                                ),
                                border: UiRect::all(Val::Px(1.0)),
                                flex_direction: FlexDirection::Row,
                                justify_content: JustifyContent::SpaceBetween,
                                ..default()
                            },
                            background_color: ui_parameters.background_color,
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn(NodeBundle {
                                style: Style {
                                    size: Size::new(
                                        Val::Undefined,
                                        Val::Px(40.0),
                                    ),
                                    ..default()
                                },
                                ..default()
                            });
                            parent.spawn(
                                TextBundle::from_section(
                                    "wave_sim",
                                    TextStyle {
                                        font: ui_parameters
                                            .default_bold_font
                                            .clone(),
                                        font_size: 30.0,
                                        color: Color::WHITE,
                                    },
                                )
                                .with_style(
                                    Style {
                                        margin: UiRect::vertical(Val::Auto),
                                        ..default()
                                    },
                                ),
                            );
                        });
                });
        });
}

fn setup_side_panel(
    parent: &mut ChildBuilder,
    ui_parameters: &UiParameters,
    app_state: &Res<State<AppState>>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                size: Size::new(Val::Px(300.0), Val::Percent(100.0)),
                border: UiRect::all(Val::Px(1.0)),
                flex_direction: FlexDirection::Column,
                justify_content: JustifyContent::FlexStart,
                ..default()
            },
            background_color: ui_parameters.background_color,
            ..default()
        })
        .with_children(|parent| {
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
                                font: ui_parameters.default_medium_font.clone(),
                                font_size: 24.0,
                                color: Color::WHITE,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect {
                                left: Val::Px(14.0),
                                top: Val::Px(14.0),
                                bottom: Val::Px(4.0),
                                ..default()
                            },
                            ..default()
                        }),
                    );

                    setup_side_panel_simulation_selection(
                        parent,
                        ui_parameters,
                        app_state,
                    );
                });
        });
}

fn setup_side_panel_simulation_selection(
    parent: &mut ChildBuilder,
    ui_parameters: &UiParameters,
    app_state: &Res<State<AppState>>,
) {
    parent
        .spawn((
            SimulationSelectButton,
            ButtonBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Px(30.0)),
                    margin: UiRect::bottom(Val::Px(10.0)),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                background_color: ui_parameters.button_color,
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle {
                text: Text::from_section(
                    (*app_state.current()).clone(),
                    TextStyle {
                        font: ui_parameters.default_medium_font.clone(),
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                ),
                style: Style {
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    padding: UiRect::all(Val::Px(5.0)),
                    ..default()
                },
                ..default()
            });

            parent.spawn(ImageBundle {
                image: ui_parameters.arrow_down_icon.clone().into(),
                style: Style {
                    size: Size::new(Val::Px(20.0), Val::Undefined),
                    margin: UiRect::horizontal(Val::Px(10.0)),
                    ..default()
                },
                focus_policy: FocusPolicy::Pass,
                ..default()
            });
        });

    parent
        .spawn((
            SimulationSelectionList,
            NodeBundle {
                style: Style {
                    size: Size::new(Val::Percent(100.0), Val::Percent(100.0)),
                    flex_direction: FlexDirection::Column,
                    justify_content: JustifyContent::FlexStart,
                    align_items: AlignItems::FlexStart,
                    display: Display::None,
                    margin: UiRect::horizontal(Val::Px(20.0)),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            for simulation in ui_parameters.simulations.clone() {
                parent
                    .spawn((
                        SimulationSelectionListItem,
                        ButtonBundle {
                            style: Style {
                                size: Size::new(
                                    Val::Percent(80.0),
                                    Val::Undefined,
                                ),
                                margin: UiRect::bottom(Val::Px(4.0)),
                                justify_content: JustifyContent::Center,
                                align_items: AlignItems::Center,
                                padding: UiRect::all(Val::Px(4.0)),
                                ..default()
                            },
                            background_color: ui_parameters.button_color,
                            ..default()
                        },
                    ))
                    .with_children(|parent| {
                        parent.spawn(
                            TextBundle::from_section(
                                simulation,
                                TextStyle {
                                    font: ui_parameters
                                        .default_medium_font
                                        .clone(),
                                    font_size: 18.0,
                                    color: Color::WHITE,
                                },
                            )
                            .with_style(Style {
                                margin: UiRect::horizontal(Val::Px(10.0)),
                                ..default()
                            }),
                        );
                    });
            }
        });
}

// events

#[allow(clippy::type_complexity)]
fn handle_simulation_selection_button(
    ui_parameters: Res<UiParameters>,
    mut button: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (Changed<Interaction>, With<SimulationSelectButton>),
    >,
    mut list: Query<&mut Style, With<SimulationSelectionList>>,
    mut images: Query<&mut UiImage>,
) {
    if let Ok((interaction, mut button_color, children)) =
        button.get_single_mut()
    {
        if let Ok(mut list_style) = list.get_single_mut() {
            match *interaction {
                Interaction::Hovered => {
                    *button_color = ui_parameters.hovered_button_color;
                }
                Interaction::None => {
                    *button_color = ui_parameters.button_color;
                }
                Interaction::Clicked => {
                    *button_color = ui_parameters.button_color;
                    let mut image = images.get_mut(children[1]).unwrap();

                    if list_style.display == Display::None {
                        list_style.display = Display::Flex;
                        image.0 = ui_parameters.arrow_up_icon.clone();
                    } else {
                        list_style.display = Display::None;
                        image.0 = ui_parameters.arrow_down_icon.clone();
                    }
                }
            }
        }
    }
}

#[allow(clippy::type_complexity)]
fn handle_simulation_selection_select(
    mut app_state: ResMut<State<AppState>>,
    ui_parameters: Res<UiParameters>,
    mut button: Query<
        &mut Children,
        (
            With<SimulationSelectButton>,
            Without<SimulationSelectionList>,
            Without<SimulationSelectionListItem>,
        ),
    >,
    mut list: Query<
        &mut Style,
        (
            With<SimulationSelectionList>,
            Without<SimulationSelectButton>,
            Without<SimulationSelectionListItem>,
        ),
    >,
    mut selections: Query<
        (&Interaction, &mut BackgroundColor, &Children),
        (
            Changed<Interaction>,
            (
                With<SimulationSelectionListItem>,
                Without<SimulationSelectButton>,
                Without<SimulationSelectionList>,
            ),
        ),
    >,
    mut text_query: Query<&mut Text>,
    mut images: Query<&mut UiImage>,
) {
    for (interaction, mut button_color, children) in selections.iter_mut() {
        match *interaction {
            Interaction::Hovered => {
                *button_color = ui_parameters.hovered_button_color;
            }
            Interaction::Clicked => {
                if let Ok(text) = text_query.get(children[0]) {
                    let clicked_simulation = text.sections[0].value.clone();
                    let next_app_state =
                        AppState::from(clicked_simulation.clone());

                    if *app_state.current() != next_app_state {
                        if let Ok(children) = button.get_single_mut() {
                            if let Ok(mut button_text) =
                                text_query.get_mut(children[0])
                            {
                                button_text.sections[0].value =
                                    clicked_simulation;
                            }
                            let mut image =
                                images.get_mut(children[1]).unwrap();
                            image.0 = ui_parameters.arrow_down_icon.clone();
                        }

                        if let Ok(mut list_style) = list.get_single_mut() {
                            list_style.display = Display::None;
                        }

                        app_state.set(next_app_state).unwrap();
                    }
                }
            }
            Interaction::None => {
                *button_color = ui_parameters.button_color;
            }
        }
    }
}
