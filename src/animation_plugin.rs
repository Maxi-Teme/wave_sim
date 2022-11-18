use std::f32::consts::E;

use bevy::prelude::*;
use bevy::render::mesh::Indices;
use bevy::render::mesh::MeshVertexAttribute;
use bevy::render::mesh::VertexAttributeValues;
use bevy::render::render_resource::PrimitiveTopology;
use bevy::render::render_resource::VertexFormat;
use bevy::sprite::Mesh2dHandle;
use ndarray::Array3;

use crate::colored_mesh::ColoredMesh2d;
use crate::colored_mesh::ColoredMesh2dPlugin;
use crate::SimulationGrid;
use crate::SimulationParameters;

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
            .add_startup_system(init_mesh)
            .add_system(update_mesh)
            .add_system(mouse_event_handler);
    }
}

fn init_mesh(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    parameters: Res<SimulationParameters>,
) {
    let dimx: u32 = (parameters.dimx - 1).try_into().unwrap();
    let dimy: u32 = (parameters.dimy - 1).try_into().unwrap();

    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    let mut v_pos: Vec<[f32; 3]> =
        Vec::with_capacity(parameters.dimx * parameters.dimy);
    let mut v_color: Vec<u32> =
        Vec::with_capacity(parameters.dimx * parameters.dimy);

    let mut count: u32 = 0;

    for x in 0..=dimx {
        for y in 0..=dimy {
            // positions of vertices
            let scaled_x = x as f32 * parameters.cellsize;
            let scaled_y = y as f32 * parameters.cellsize;
            v_pos.push([scaled_x, scaled_y, 0.0]);

            // color of vertices
            if count % 2 == 0 {
                v_color.push(Color::WHITE.as_linear_rgba_u32());
            } else {
                v_color.push(Color::DARK_GRAY.as_linear_rgba_u32());
            }

            count += 1;
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

            info!("{:?}", r_ru_triangle);
            info!("{:?}", ru_u_triangle);

            indices.extend_from_slice(&r_ru_triangle);
            indices.extend_from_slice(&ru_u_triangle);
        }
    }

    mesh.set_indices(Some(Indices::U32(indices)));

    let dimx_shift: f32 = -(dimx as f32) * parameters.cellsize / 2.0;
    let dimy_shift: f32 = -(dimy as f32) * parameters.cellsize / 2.0;

    info!("{:?}", dimx_shift);
    info!("{:?}", dimy_shift);

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

    info!("0.0: {}", Color::rgb(0.0, 0.0, 0.0).as_linear_rgba_u32());
    info!("0.2: {}", Color::rgb(0.2, 0.2, 0.2).as_linear_rgba_u32());
    info!("0.4: {}", Color::rgb(0.4, 0.4, 0.4).as_linear_rgba_u32());
    info!("0.6: {}", Color::rgb(0.6, 0.6, 0.6).as_linear_rgba_u32());
    info!("0.8: {}", Color::rgb(0.8, 0.8, 0.8).as_linear_rgba_u32());
    info!("1.0: {}", Color::rgb(1.0, 1.0, 1.0).as_linear_rgba_u32());
}

fn update_mesh(
    u: Res<SimulationGrid>,
    parameters: Res<SimulationParameters>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    for (_, mesh) in meshes.iter_mut() {
        let vertex_attribute =
            mesh.attribute_mut(VERTEX_ATTRIBUTE_COLOR_ID).unwrap();

        if let VertexAttributeValues::Uint32(color_vector) = vertex_attribute {
            *color_vector = get_color_vector(&parameters, &u.0);
        }
    }
}

fn get_color_vector(
    parameters: &SimulationParameters,
    simulation_grid: &Array3<f32>,
) -> Vec<u32> {
    let dimx = parameters.dimx - 1;
    let dimy = parameters.dimy - 1;

    let mut color_vector =
        Vec::with_capacity(parameters.dimx * parameters.dimy);

    let max_amplitude = parameters.applied_force_amplitude + 1.0;

    for x in 0..=dimx {
        for y in 0..=dimy {
            let amplitude = simulation_grid.get((0, x, y)).unwrap();
            let amplitude = amplitude / max_amplitude;
            let amplitude = (amplitude * 48.0 + 1.0).log(E) / 4.0;

            color_vector.push(get_smooth_color_by_amplitude(amplitude));
        }
    }

    color_vector
}

fn get_smooth_color_by_amplitude(amplitude: f32) -> u32 {
    Color::rgb(amplitude, amplitude, amplitude).as_linear_rgba_u32()
}

fn mouse_event_handler(
    windows: Res<Windows>,
    cameras: Query<(&Camera, &GlobalTransform)>,
    buttons: Res<Input<MouseButton>>,
    plots: Query<&Transform, With<Plot>>,
    parameters: Res<SimulationParameters>,
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

                info!(
                    "World coordinates: {}/{}\nPlot coordinates: {}/{}",
                    world_position.x, world_position.y, plot_x, plot_y,
                );

                event.send(PlotClickedEvent {
                    x: plot_x,
                    y: plot_y,
                });
            }
        }
    }
}
