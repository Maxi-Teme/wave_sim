use std::f32::consts::TAU;

use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy::utils::HashMap;
use bevy_egui::egui;
use bevy_rapier3d::prelude::*;
use bevy_rapier3d::render::DebugRenderContext;
use itertools::Itertools;

use crate::objects_3d::spawn_koordinate_system_helper;
use crate::pan_orbit_camera::{update_pan_orbit_camera, PanOrbitCamera};
use crate::{AppCamera, AppState};

#[derive(Default, Resource)]
struct WaveStopwatch(Stopwatch);

#[derive(Component)]
enum Particle {
    Passive,
    Active,
}

#[derive(Resource)]
pub struct WaveInPanelParameters {
    particle_mesh_handle: Handle<Mesh>,
    passive_particle_material_handle: Handle<StandardMaterial>,
    active_particle_material_handle: Handle<StandardMaterial>,
    particles_map: HashMap<Entity, Vec<Entity>>,

    dimx: f32,
    dimy: f32,
    dimz: f32,
    particle_radius: f32,

    equalizing_force_factor: f32,
    applying_force_frequency: f32,
    applying_force_factor: f32,
    sysnthetic_energy_loss_factor: f32,
}

impl Default for WaveInPanelParameters {
    fn default() -> Self {
        Self {
            particle_mesh_handle: Handle::<Mesh>::default(),
            passive_particle_material_handle:
                Handle::<StandardMaterial>::default(),
            active_particle_material_handle:
                Handle::<StandardMaterial>::default(),
            particles_map: HashMap::<Entity, Vec<Entity>>::default(),

            // initially fixed parameters
            dimx: 14.0,
            dimy: 8.0,
            dimz: 0.0,
            particle_radius: 0.1,

            // dynamically applicable parameters
            equalizing_force_factor: 2.0,
            applying_force_frequency: 3.5,
            applying_force_factor: 0.1,
            sysnthetic_energy_loss_factor: 0.997,
        }
    }
}

pub struct WaveInPanelPlugin;

impl Plugin for WaveInPanelPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<UiEvents>()
            .insert_resource(WaveStopwatch::default())
            .insert_resource(WaveInPanelParameters::default())
            .add_system_set(
                SystemSet::on_enter(AppState::WaveInPanel).with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::WaveInPanel)
                    .with_system(update_equalizing_forces)
                    .with_system(apply_external_force)
                    .with_system(on_ui_events)
                    .with_system(apply_synthetic_energy_loss)
                    .with_system(on_input_events)
                    .with_system(update_pan_orbit_camera),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::WaveInPanel).with_system(cleanup),
            );
    }
}

// setup

#[allow(clippy::too_many_arguments)]
fn setup(
    mut commands: Commands,
    cameras: Query<Entity, With<AppCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut parameters: ResMut<WaveInPanelParameters>,
    mut rapier_debug_config: ResMut<DebugRenderContext>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    rapier_debug_config.enabled = false;
    rapier_config.gravity = Vec3::ZERO;
    rapier_config.scaled_shape_subdivision = 1;

    if let Ok(camera_entity) = cameras.get_single() {
        commands.entity(camera_entity).despawn();
    }

    // mesh
    parameters.particle_mesh_handle =
        meshes.add(Mesh::from(shape::Icosphere {
            radius: parameters.particle_radius * 1.3,
            subdivisions: 1,
        }));

    // material
    parameters.passive_particle_material_handle =
        materials.add(StandardMaterial::from(Color::rgb(0.3, 0.1, 0.1)));
    parameters.active_particle_material_handle =
        materials.add(StandardMaterial::from(Color::rgb(0.6, 0.0, 0.0)));

    // koordinate system points
    spawn_koordinate_system_helper(&mut commands, &mut meshes, &mut materials);

    // light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform::from_translation(Vec3::new(27.0, 27.0, 4.0))
            .looking_at(Vec3::splat(0.0), Vec3::Y),
        ..default()
    });

    // camera
    let camera_translation = Vec3::new(1.8, 2.7, 8.0);
    let focus = Vec3::new(0.0, 0.0, 0.0);
    commands.spawn((
        AppCamera,
        Camera3dBundle {
            transform: Transform::from_translation(camera_translation)
                .looking_at(focus, Vec3::Y),
            ..default()
        },
        PanOrbitCamera {
            focus,
            radius: camera_translation.length(),
            ..default()
        },
    ));

    // spawn particles
    let entities_and_positions = spawn_particles(&mut commands, &parameters);

    // find nearby particles
    store_nearby_particles(&entities_and_positions, &mut parameters);
}

fn spawn_particles(
    commands: &mut Commands,
    parameters: &WaveInPanelParameters,
) -> Vec<(Entity, Vec3)> {
    let particle_size = parameters.particle_radius * 2.1;
    let stepsx = (parameters.dimx / particle_size).floor() as usize;
    let stepsy = (parameters.dimy / particle_size).floor() as usize;
    let stepsz = (parameters.dimz / particle_size).floor() as usize;

    let mut entities_and_positions = Vec::new();

    for x in 0..=stepsx {
        for y in 0..=stepsy {
            for z in 0..=stepsz {
                let position =
                    Vec3::new(x as f32, y as f32, z as f32) * particle_size;

                let mut entity = commands.spawn((
                    PbrBundle {
                        transform: Transform::from_translation(position),
                        mesh: parameters.particle_mesh_handle.clone(),
                        material: parameters
                            .passive_particle_material_handle
                            .clone(),
                        ..default()
                    },
                    Collider::ball(parameters.particle_radius),
                    Velocity::default(),
                    ExternalForce::default(),
                    Particle::Passive,
                ));

                if x == 0 || x == stepsx || y == 0 || y == stepsy {
                    entity.insert(RigidBody::Fixed);
                } else {
                    entity.insert(RigidBody::Dynamic);
                }

                entities_and_positions.push((entity.id(), position));
            }
        }
    }

    entities_and_positions
}

fn store_nearby_particles(
    entities_and_positions: &[(Entity, Vec3)],
    parameters: &mut WaveInPanelParameters,
) {
    for combinations in entities_and_positions.iter().combinations(2) {
        let xz1 = combinations[0].1;
        let xz2 = combinations[1].1;
        let distance = xz1.distance(xz2);

        if distance < 1.1 {
            parameters
                .particles_map
                .entry(combinations[0].0)
                .and_modify(|n| n.push(combinations[1].0))
                .or_insert(vec![combinations[1].0]);

            parameters
                .particles_map
                .entry(combinations[1].0)
                .and_modify(|n| n.push(combinations[0].0))
                .or_insert(vec![combinations[0].0]);
        }
    }
}

fn apply_synthetic_energy_loss(
    parameters: Res<WaveInPanelParameters>,
    mut particles: Query<&mut Velocity, With<Particle>>,
) {
    if parameters.sysnthetic_energy_loss_factor == 1.0 {
        return;
    }

    for mut particle in particles.iter_mut() {
        particle.linvel *= parameters.sysnthetic_energy_loss_factor;
    }
}

fn update_equalizing_forces(
    parameters: Res<WaveInPanelParameters>,
    mut particles: Query<(Entity, &Particle, &Transform, &mut Velocity)>,
    particles_transforms: Query<&Transform, With<Particle>>,
) {
    for (entity, particle, transform, mut velocity) in particles.iter_mut() {
        if let Particle::Active = particle {
            continue;
        }

        let neighbors =
            if let Some(neighbors) = parameters.particles_map.get(&entity) {
                neighbors
            } else {
                continue;
            };

        for neighbor in neighbors.iter() {
            if let Ok(neighbour_transform) = particles_transforms.get(*neighbor)
            {
                let equalizing_force = (neighbour_transform.translation
                    - transform.translation)
                    * Vec3::new(0.0, 0.0, parameters.equalizing_force_factor);

                velocity.linvel += equalizing_force;
            } else {
                continue;
            };
        }
    }
}

fn apply_external_force(
    time: Res<Time>,
    mut stopwatch: ResMut<WaveStopwatch>,
    mut particles: Query<(&Particle, &mut Transform)>,
    parameters: Res<WaveInPanelParameters>,
) {
    stopwatch.0.tick(time.delta());

    for (particle, mut transform) in particles.iter_mut() {
        if let Particle::Active = particle {
            let elapsed_time = stopwatch.0.elapsed();
            let amplitude = (TAU
                * parameters.applying_force_frequency
                * elapsed_time.as_secs_f32())
            .sin()
                * parameters.applying_force_factor;

            transform.translation.z = amplitude;
        }
    }
}

fn on_input_events(
    windows: Res<Windows>,
    input_mouse: Res<Input<MouseButton>>,
    camera: Query<(&Camera, &GlobalTransform), With<AppCamera>>,
    rapier_context: Res<RapierContext>,
    parameters: Res<WaveInPanelParameters>,
    mut particles: Query<(&mut Handle<StandardMaterial>, &mut Particle)>,
) {
    if input_mouse.just_pressed(MouseButton::Left) {
        let (camera, camera_transform) = camera.get_single().unwrap();
        let window = windows.get_primary().unwrap();

        if let Some(ray) = window
            .cursor_position()
            .and_then(|p| camera.viewport_to_world(camera_transform, p))
        {
            if let Some(entity) = rapier_context.cast_ray(
                ray.origin,
                ray.direction,
                Real::MAX,
                true,
                QueryFilter::default(),
            ) {
                if let Ok((mut material, mut particle)) =
                    particles.get_mut(entity.0)
                {
                    if let Particle::Active = particle.as_ref() {
                        *material =
                            parameters.passive_particle_material_handle.clone();
                        *particle = Particle::Passive;
                    } else {
                        *material =
                            parameters.active_particle_material_handle.clone();
                        *particle = Particle::Active;
                    }
                }
            }
        }
    }
}

fn on_ui_events(
    mut time: ResMut<Time>,
    mut commands: Commands,
    mut ui_events: EventReader<UiEvents>,
    particles: Query<Entity, With<Particle>>,
    mut parameters: ResMut<WaveInPanelParameters>,
) {
    let mut cleanup = false;
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
                cleanup = true;
            }
        }
    }

    if cleanup {
        cleanup_particles(&mut commands, &mut parameters, particles);

        let entities_and_positions =
            spawn_particles(&mut commands, &parameters);

        store_nearby_particles(&entities_and_positions, &mut parameters);
    }
}

// cleanup

fn cleanup_particles(
    commands: &mut Commands,
    parameters: &mut WaveInPanelParameters,
    particles: Query<Entity, With<Particle>>,
) {
    for entity in particles.iter() {
        commands.entity(entity).despawn();
    }

    parameters.particles_map.clear();
}

fn cleanup(
    mut commands: Commands,
    mut rapier_debug_config: ResMut<DebugRenderContext>,
    mut rapier_config: ResMut<RapierConfiguration>,
    entities: Query<Entity>,
) {
    for entity in entities.iter() {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.despawn();
        }
    }

    *rapier_debug_config = DebugRenderContext::default();
    *rapier_config = RapierConfiguration::default();
}

// ui

pub enum UiEvents {
    StartStopTime,
    Reset,
}

pub fn show_ui(
    ui: &mut egui::Ui,
    rapier_debug_config: &mut DebugRenderContext,
    mut ui_events: EventWriter<UiEvents>,
    parameters: &mut WaveInPanelParameters,
) {
    ui.allocate_space(egui::vec2(1.0, 10.0));

    ui.label("equalizing force factor");
    ui.add(
        egui::Slider::new(&mut parameters.equalizing_force_factor, 0.0..=10.0)
            .step_by(0.1),
    );

    ui.label("applying force frequency in Hz");
    ui.add(
        egui::Slider::new(&mut parameters.applying_force_frequency, 0.0..=20.0)
            .step_by(0.1),
    );

    ui.label("applying force factor");
    ui.add(
        egui::Slider::new(&mut parameters.applying_force_factor, 0.0..=0.4)
            .step_by(0.01),
    );

    ui.label("synthetic velocity loss factor:");
    ui.add(
        egui::Slider::new(
            &mut parameters.sysnthetic_energy_loss_factor,
            0.5..=1.0,
        )
        .step_by(0.01),
    );

    ui.horizontal(|ui| {
        if ui.button("Start / Stop time").clicked() {
            ui_events.send(UiEvents::StartStopTime);
        }
        if ui.button("Reset").clicked() {
            ui_events.send(UiEvents::Reset);
        }
    });

    ui.allocate_space(egui::vec2(1.0, 2.0));
    ui.separator();
    ui.allocate_space(egui::vec2(1.0, 2.0));

    ui.add(egui::Checkbox::new(
        &mut rapier_debug_config.enabled,
        "rapier debug",
    ));
}
