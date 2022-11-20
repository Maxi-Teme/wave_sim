use std::f32::consts::E;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_resource::VertexFormat;
use bevy::sprite::Mesh2dHandle;
use ndarray::Array3;

use super::UiEvents;
use super::Wave2dSimulationGrid;
use super::Wave2dSimulationParameters;
use crate::colored_mesh::ColoredMesh2d;
use crate::colored_mesh::ColoredMesh2dPlugin;
use crate::AppCamera;
use crate::AppState;

const VERTEX_ATTRIBUTE_COLOR_ID: MeshVertexAttribute =
    MeshVertexAttribute::new("Vertex_Color", 1, VertexFormat::Uint32);

#[derive(Component)]
struct Plot;

pub struct PlotClickedEvent {
    pub x: f32,
    pub y: f32,
}

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_plugin(ColoredMesh2dPlugin)
            .add_event::<PlotClickedEvent>()
            .add_system_set(
                SystemSet::on_enter(AppState::Wave2dSimulation)
                    .with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::Wave2dSimulation)
                    .with_system(update_mesh)
                    .with_system(mouse_event_handler)
                    .with_system(on_ui_events),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::Wave2dSimulation)
                    .with_system(cleanup),
            );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    parameters: Res<Wave2dSimulationParameters>,
    cameras: Query<Entity, With<AppCamera>>,
    mut mouse_button: ResMut<Input<MouseButton>>,
) {
    mouse_button.reset_all();

    initialize_plot(&mut commands, &parameters, &mut meshes);

    if let Ok(camera_entity) = cameras.get_single() {
        commands.entity(camera_entity).despawn();
    }

    commands.spawn((AppCamera, Camera2dBundle::default()));
}

fn initialize_plot(
    commands: &mut Commands,
    parameters: &Wave2dSimulationParameters,
    meshes: &mut Assets<Mesh>,
) {
    let dimx: u32 = (parameters.dimx - 1).try_into().unwrap();
    let dimy: u32 = (parameters.dimy - 1).try_into().unwrap();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut v_pos: Vec<[f32; 3]> =
        Vec::with_capacity(parameters.dimx * parameters.dimy);
    let mut v_color: Vec<u32> =
        Vec::with_capacity(parameters.dimx * parameters.dimy);

    let white = Color::WHITE.as_linear_rgba_u32();

    for x in 0..=dimx {
        for y in 0..=dimy {
            // positions of vertices
            let scaled_x = x as f32 * parameters.cellsize;
            let scaled_y = y as f32 * parameters.cellsize;
            v_pos.push([scaled_x, scaled_y, 0.0]);

            // color of vertices
            v_color.push(white);
        }
    }

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, v_pos);
    mesh.insert_attribute(VERTEX_ATTRIBUTE_COLOR_ID, v_color);

    // indices of vertices
    let mut indices: Vec<u32> =
        Vec::with_capacity(parameters.dimx * parameters.dimy);

    for c in 0..dimx {
        for r in 0..dimy {
            let i = c * (dimy + 1) + r;

            let r_ru_triangle = [i, i + dimy + 1, i + dimy + 2]; // right and right up triangle
            let ru_u_triangle = [i, i + dimy + 2, i + 1]; // right up and up triagle

            indices.extend_from_slice(&r_ru_triangle);
            indices.extend_from_slice(&ru_u_triangle);
        }
    }

    mesh.set_indices(Some(Indices::U32(indices)));

    let dimx_shift: f32 = -(dimx as f32) * parameters.cellsize / 4.0;
    let dimy_shift: f32 = -(dimy as f32) * parameters.cellsize / 2.0;

    // info!("{:?}", dimx_shift);
    // info!("{:?}", dimy_shift);

    commands.spawn((
        Plot,
        ColoredMesh2d::default(),
        Mesh2dHandle(meshes.add(mesh)),
        SpatialBundle {
            visibility: Visibility::VISIBLE,
            computed: ComputedVisibility::INVISIBLE,
            transform: Transform {
                translation: Vec3::new(dimx_shift, dimy_shift, 0.0),
                rotation: Quat::IDENTITY,
                scale: Vec3::ONE,
            },
            global_transform: GlobalTransform::IDENTITY,
        },
    ));
}

fn update_mesh(
    u: Res<Wave2dSimulationGrid>,
    mut parameters: ResMut<Wave2dSimulationParameters>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (_, mesh) in meshes.iter_mut() {
        let vertex_attribute = mesh.attribute_mut(VERTEX_ATTRIBUTE_COLOR_ID);

        if let Some(VertexAttributeValues::Uint32(color_vector)) =
            vertex_attribute
        {
            *color_vector = get_color_vector(&mut parameters, &u.0);
        }
    }
}

fn get_color_vector(
    parameters: &mut Wave2dSimulationParameters,
    simulation_grid: &Array3<f32>,
) -> Vec<u32> {
    let dimx = parameters.dimx - 1;
    let dimy = parameters.dimy - 1;

    let mut color_vector =
        Vec::with_capacity(parameters.dimx * parameters.dimy);

    let mut max_amplitude = f32::MIN;

    for x in 0..=dimx {
        for y in 0..=dimy {
            let amplitude = simulation_grid.get((0, x, y)).unwrap();

            if *amplitude > max_amplitude {
                max_amplitude = *amplitude;
            }

            let amplitude = amplitude / parameters.max_amplitude;
            let amplitude = (amplitude * 48.0 + 1.0).log(E) / 4.0;

            color_vector.push(get_smooth_color_by_amplitude(amplitude));
        }
    }

    parameters.max_amplitude_avg.pop_back();
    parameters.max_amplitude_avg.push_front(max_amplitude);

    let avg = parameters.max_amplitude_avg.iter().sum::<f32>()
        / parameters.max_amplitude_avg.len() as f32;

    parameters.max_amplitude = avg.clamp(0.1, 0.9);

    color_vector
}

fn get_smooth_color_by_amplitude(amplitude: f32) -> u32 {
    Color::rgb(amplitude, amplitude, amplitude).as_linear_rgba_u32()
}

fn mouse_event_handler(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform), With<AppCamera>>,
    buttons: Res<Input<MouseButton>>,
    plots: Query<&Transform, With<Plot>>,
    parameters: Res<Wave2dSimulationParameters>,
    mut event: EventWriter<PlotClickedEvent>,
) {
    let (camera, camera_transform) = cameras.get_single().unwrap();
    if buttons.just_pressed(MouseButton::Left) {
        let window = windows.get_primary().unwrap();

        if let Some(screen_position) = window.cursor_position() {
            let window_size = Vec2::new(window.width(), window.height());
            let ndc = (screen_position / window_size) * 2.0 - Vec2::ONE;
            let ndc_to_world = camera_transform.compute_matrix()
                * camera.projection_matrix().inverse();
            let world_position = ndc_to_world.project_point3(ndc.extend(-1.0));
            let world_position: Vec2 = world_position.truncate();

            if let Some(plot_transform) = plots.iter().next() {
                let plot_x = (world_position.x - plot_transform.translation.x)
                    / parameters.cellsize;
                let plot_y = (world_position.y - plot_transform.translation.y)
                    / parameters.cellsize;

                event.send(PlotClickedEvent {
                    x: plot_x,
                    y: plot_y,
                });
            }
        }
    }
}

fn on_ui_events(
    mut time: ResMut<Time>,
    mut ui_events: EventReader<UiEvents>,
    mut u: ResMut<Wave2dSimulationGrid>,
    parameters: Res<Wave2dSimulationParameters>,
) {
    for event in ui_events.iter() {
        match event {
            UiEvents::StartStopTime => {
                if time.is_paused() {
                    time.unpause();
                } else {
                    time.pause();
                }
            }
            UiEvents::Reset => {
                u.0 = Array3::zeros((3, parameters.dimx, parameters.dimy));
            }
        }
    }
}

fn cleanup(mut commands: Commands, plots: Query<Entity, With<Plot>>) {
    for plot in plots.iter() {
        if let Some(mut entity) = commands.get_entity(plot) {
            entity.despawn();
        }
    }
}
