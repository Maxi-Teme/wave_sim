use std::f32::consts::{PI, TAU};

use bevy::ecs::component::StorageType;
use bevy::ecs::system::EntityCommands;
use bevy::prelude::shape::Box;
use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_egui::egui;
use bevy_rapier3d::prelude::*;
use rand::rngs::ThreadRng;
use rand::Rng;

use crate::pan_orbit_camera::{update_pan_orbit_camera, PanOrbitCamera};
use crate::{AppCamera, AppState};

#[derive(Default, Resource)]
struct Entities(Vec<Entity>);

#[derive(Resource)]
pub struct ParticleMessParameters {
    dimx: f32,
    dimy: f32,
    dimz: f32,
    origin: Vec3,
    particle_radius: f32,
    restitution_coefficient: f32,
    number_of_particles: usize,

    particle_mesh: Handle<Mesh>,
    default_particle_material: Handle<StandardMaterial>,
    marked_particle_material: Handle<StandardMaterial>,

    spawn_particles: bool,
    spawn_frequency_hz: f32,
    spawn_particles_num: usize,
    max_entities: usize,
    gravitation_on_particle: f32,
    heat: f32,
    energy_conservation_factor: f32,
}

impl Default for ParticleMessParameters {
    fn default() -> Self {
        let origin = Vec3::new(1.0, 0.2, 1.0);
        Self {
            dimx: origin.x,
            dimy: origin.y,
            dimz: origin.z,
            origin,
            particle_radius: 0.01,
            number_of_particles: 0,

            particle_mesh: Handle::<Mesh>::default(),
            default_particle_material: Handle::<StandardMaterial>::default(),
            marked_particle_material: Handle::<StandardMaterial>::default(),

            max_entities: 1000,
            spawn_particles: false,
            spawn_frequency_hz: 20.0,
            spawn_particles_num: 1,
            restitution_coefficient: 0.5,
            gravitation_on_particle: 0.0,
            heat: 0.0,
            energy_conservation_factor: 1.0,
        }
    }
}

#[derive(Default, Resource)]
struct ParticleMessStopwatch(Stopwatch);

#[derive(Default, Component)]
struct Particle;

#[derive(Default, Bundle)]
struct ParticleBundle {
    _particle: Particle,
    pbr_bundle: PbrBundle,
    collider: Collider,
    rigid_body: RigidBody,
    restitution: Restitution,
    velocity: Velocity,
    external_impulse: ExternalImpulse,
    external_force: ExternalForce,
}

pub struct ParticleMessPlugin;

impl Plugin for ParticleMessPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Entities::default())
            .insert_resource(ParticleMessParameters::default())
            .insert_resource(ParticleMessStopwatch::default())
            .add_system_set(
                SystemSet::on_enter(AppState::ParticleMess).with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::ParticleMess)
                    .with_system(update_pan_orbit_camera)
                    .with_system(update)
                    .with_system(update_global_parameters)
                    .with_system(apply_gravity)
                    .with_system(apply_heat),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::ParticleMess).with_system(cleanup),
            );
    }
}

fn setup(
    mut commands: Commands,
    cameras: Query<Entity, With<AppCamera>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut parameters: ResMut<ParticleMessParameters>,
    mut entities: ResMut<Entities>,
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
    parameters.particle_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: parameters.particle_radius,
        subdivisions: 6,
    }));

    // materials
    // parameters.default_particle_material =
    //     materials.add(Color::rgb(0.4, 0.4, 0.4).into());
    parameters.default_particle_material =
        materials.add(Color::rgb(0.3, 0.1, 0.1).into());

    parameters.marked_particle_material =
        materials.add(Color::rgb(1.0, 0.0, 0.0).into());

    // plane
    entities.0.push(
        commands
            .spawn(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: 1000.0 })),
                material: materials.add(Color::rgba(0.3, 0.3, 0.6, 0.8).into()),
                transform: Transform::from_translation(Vec3::new(
                    0.0, -4.0, 0.0,
                )),
                ..default()
            })
            .id(),
    );

    // origin
    entities.0.push(
        commands
            .spawn(PbrBundle {
                mesh: parameters.particle_mesh.clone(),
                material: parameters.marked_particle_material.clone(),
                transform: Transform::from_translation(parameters.origin),
                ..default()
            })
            .id(),
    );

    // container
    // let container_collider = container_collider(&parameters);
    // let container_mesh = container_mesh(&parameters);

    // let container = commands.spawn((
    //     container_collider,
    //     meshes.add(container_mesh),
    //     materials.add(Color::rgba(0.8, 0.8, 0.8, 0.2).into()),
    //     TransformBundle::from_transform(Transform::from_xyz(
    //         parameters.dimx,
    //         parameters.dimy,
    //         parameters.dimz,
    //     )),
    //     Visibility::default(),
    //     ComputedVisibility::default(),
    // ));
    // entities.0.push(container.id());

    // directional 'sun' light
    let sunlight = commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            illuminance: 10000.0,
            ..default()
        },
        transform: Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.0)
                .mul_quat(Quat::from_rotation_y(PI / 4.0)),
            ..default()
        },
        ..default()
    });
    entities.0.push(sunlight.id());

    // camera
    let translation =
        Vec3::new(parameters.dimx, parameters.dimy * 0.1, parameters.dimz);
    let radius = translation.length();

    commands
        .spawn((
            AppCamera,
            Camera3dBundle {
                transform: Transform::from_translation(translation)
                    .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
                ..default()
            },
        ))
        .insert(PanOrbitCamera {
            radius,
            ..Default::default()
        });
}

fn update(
    time: Res<Time>,
    mut stopwatch: ResMut<ParticleMessStopwatch>,
    mut commands: Commands,
    parameters: Res<ParticleMessParameters>,
    particles: Query<&Transform, With<Particle>>,
    mut entities: ResMut<Entities>,
) {
    stopwatch.0.tick(time.delta());

    let period = 1000.0 / parameters.spawn_frequency_hz;

    if stopwatch.0.elapsed().as_millis() >= period as u128
        && particles.iter().len() < parameters.max_entities
        && parameters.spawn_particles
    {
        stopwatch.0.reset();
        let mut rng = rand::thread_rng();

        for _ in 0..parameters
            .spawn_particles_num
            .min(parameters.max_entities - parameters.number_of_particles)
        {
            let particle = commands.spawn(particle(&parameters, &mut rng));
            entities.0.push(particle.id());
        }
    }
}

fn apply_gravity(
    parameters: Res<ParticleMessParameters>,
    mut particles: Query<
        (&mut ExternalForce, &mut Velocity, &Transform),
        With<Particle>,
    >,
) {
    for (mut particle, mut velocity, transform) in particles.iter_mut() {
        velocity.linvel *= parameters.energy_conservation_factor;

        particle.force = (parameters.origin - transform.translation)
            .normalize()
            * parameters.gravitation_on_particle;
    }
}

fn apply_heat(
    parameters: Res<ParticleMessParameters>,
    // mut particles: Query<&mut ExternalForce, With<Particle>>,
    mut particles: Query<&mut ExternalImpulse, With<Particle>>,
) {
    let mut rng = rand::thread_rng();

    let dimx = parameters.particle_radius * (parameters.heat / 1000.0);
    let dimy = parameters.particle_radius * (parameters.heat / 1000.0);
    let dimz = parameters.particle_radius * (parameters.heat / 1000.0);

    for mut particle in particles.iter_mut() {
        if parameters.heat > 0.0 {
            let x: f32 = rng.gen_range(-dimx..dimx);
            let y: f32 = rng.gen_range(-dimy..dimy);
            let z: f32 = rng.gen_range(-dimz..dimz);

            particle.impulse = Vec3::new(x, y, z);
        } else {
            particle.impulse = Vec3::splat(0.0);
        }
    }
}

fn update_global_parameters(
    mut parameters: ResMut<ParticleMessParameters>,
    particles: Query<&Particle>,
) {
    parameters.number_of_particles = particles.iter().len();
}

fn cleanup(
    mut commands: Commands,
    mut entities: ResMut<Entities>,
    mut rapier_debug_config: ResMut<DebugRenderContext>,
    mut rapier_config: ResMut<RapierConfiguration>,
) {
    for entity in entities.0.drain(..) {
        if let Some(mut entity) = commands.get_entity(entity) {
            entity.despawn();
        }
    }

    *rapier_debug_config = DebugRenderContext::default();
    *rapier_config = RapierConfiguration::default();
}

fn particle(
    parameters: &ParticleMessParameters,
    rng: &mut ThreadRng,
) -> ParticleBundle {
    let x: f32 = rng.gen_range(0.001..parameters.dimx * 1.99);
    let y: f32 = rng.gen_range(0.001..parameters.dimy * 1.99);
    let z: f32 = rng.gen_range(0.001..parameters.dimz * 1.99);
    let translation = Vec3::new(x, y, z);

    ParticleBundle {
        pbr_bundle: PbrBundle {
            mesh: parameters.particle_mesh.clone(),
            material: parameters.default_particle_material.clone(),
            transform: Transform::from_translation(translation),
            ..default()
        },
        collider: Collider::ball(parameters.particle_radius),
        rigid_body: RigidBody::Dynamic,
        restitution: Restitution::coefficient(
            parameters.restitution_coefficient,
        ),
        ..default()
    }
}

fn container_collider(parameters: &ParticleMessParameters) -> Collider {
    let panel_thickness = parameters.particle_radius * 40.0;
    // parameters.dimx.max(parameters.dimy).max(parameters.dimz) / 10.0;

    let collider_xz = Collider::cuboid(
        parameters.dimx + panel_thickness,
        panel_thickness + panel_thickness,
        parameters.dimz + panel_thickness,
    );
    let collider_xy = Collider::cuboid(
        parameters.dimx + panel_thickness,
        parameters.dimy + panel_thickness,
        panel_thickness + panel_thickness,
    );
    let collider_zy = Collider::cuboid(
        panel_thickness + panel_thickness,
        parameters.dimy + panel_thickness,
        parameters.dimz + panel_thickness,
    );

    let p_bottom =
        Vec3::new(0.0, -parameters.dimy - 2.0 * panel_thickness, 0.0);
    let p_top = Vec3::new(0.0, parameters.dimy + 2.0 * panel_thickness, 0.0);
    let p_left = Vec3::new(0.0, 0.0, -parameters.dimz - 2.0 * panel_thickness);
    let p_right = Vec3::new(0.0, 0.0, parameters.dimz + 2.0 * panel_thickness);
    let p_near = Vec3::new(-parameters.dimx - 2.0 * panel_thickness, 0.0, 0.0);
    let p_far = Vec3::new(parameters.dimx + 2.0 * panel_thickness, 0.0, 0.0);

    Collider::compound(vec![
        (p_bottom, Quat::IDENTITY, collider_xz.clone()),
        (p_top, Quat::IDENTITY, collider_xz),
        (p_left, Quat::IDENTITY, collider_xy.clone()),
        (p_right, Quat::IDENTITY, collider_xy),
        (p_near, Quat::IDENTITY, collider_zy.clone()),
        (p_far, Quat::IDENTITY, collider_zy),
    ])
}

fn container_mesh(parameters: &ParticleMessParameters) -> Mesh {
    Mesh::from(Box::new(
        parameters.dimx * 2.0,
        parameters.dimy * 2.0,
        parameters.dimz * 2.0,
    ))
}

fn rect(xyz: Vec3) -> (Box, Collider) {
    (
        shape::Box::new(xyz.x * 2.0, xyz.y * 2.0, xyz.z * 2.0),
        Collider::cuboid(xyz.x, xyz.y, xyz.z),
    )
}

fn bowl(
    material: Handle<StandardMaterial>,
) -> (
    TransformBundle,
    Collider,
    Handle<StandardMaterial>,
    Visibility,
    ComputedVisibility,
) {
    let mut vertices: Vec<Vec3> = Vec::new();
    let mut indices: Vec<[u32; 3]> = Vec::new();

    let segments = 32;
    let bowl_size = Vec3::new(10.0, 3.0, 10.0);

    for ix in 0..=segments {
        for iz in 0..=segments {
            let shifted_z = (iz as f32 / segments as f32 - 0.5) * 2.0;
            let shifted_x = (ix as f32 / segments as f32 - 0.5) * 2.0;
            let clamped_radius =
                (shifted_z.powi(2) + shifted_x.powi(2)).sqrt().min(1.0);
            let x = shifted_x * bowl_size.x / 2.0;
            let z = shifted_z * bowl_size.z / 2.0;
            let y =
                ((clamped_radius - 0.5) * TAU / 2.0).sin() * bowl_size.y / 2.0;
            vertices.push(Vec3::new(x, y, z));
        }
    }
    for ix in 0..segments {
        for iz in 0..segments {
            let row0 = ix * (segments + 1);
            let row1 = (ix + 1) * (segments + 1);
            indices.push([row0 + iz, row0 + iz + 1, row1 + iz]);
            indices.push([row1 + iz, row0 + iz + 1, row1 + iz + 1]);
        }
    }

    (
        TransformBundle::from(Transform::from_translation(bowl_size / 2.0)),
        Collider::trimesh(vertices, indices),
        material,
        Visibility::default(),
        ComputedVisibility::default(),
    )
}

// ui

pub fn show_ui(
    ui: &mut egui::Ui,
    parameters: &mut ParticleMessParameters,
    rapier_debug_config: &mut DebugRenderContext,
) {
    ui.allocate_space(egui::Vec2::new(1.0, 10.0));

    ui.add(
        egui::Slider::new(&mut parameters.max_entities, 0..=10000)
            .step_by(500.0)
            .text("max particles"),
    );

    ui.add(egui::Checkbox::new(
        &mut parameters.spawn_particles,
        "spawn particles",
    ));

    ui.add(
        egui::Slider::new(&mut parameters.spawn_particles_num, 1..=500)
            .step_by(1.0)
            .text("spawn this many particles at once"),
    );

    ui.add(
        egui::Slider::new(&mut parameters.spawn_frequency_hz, 0.0..=100.0)
            .step_by(1.0)
            .text("spawn frequency"),
    );

    ui.add(
        egui::Slider::new(&mut parameters.restitution_coefficient, 0.0..=1.0)
            .step_by(0.1)
            .text("restitution coefficient"),
    );

    ui.add(
        egui::Slider::new(
            &mut parameters.gravitation_on_particle,
            0.0..=0.0001,
        )
        .step_by(0.00001)
        .text("factor of gravitation on the particle"),
    );

    ui.add(
        egui::Slider::new(&mut parameters.heat, 0.0..=0.2)
            .step_by(0.001)
            .text("heat"),
    );

    ui.label("synthetic velocity loss factor:");
    ui.add(
        egui::Slider::new(
            &mut parameters.energy_conservation_factor,
            0.95..=1.0,
        )
        .step_by(0.0001),
    );

    ui.separator();

    ui.label(format!(
        "number of particles: {}",
        parameters.number_of_particles
    ));

    ui.separator();

    ui.add(egui::Checkbox::new(
        &mut rapier_debug_config.enabled,
        "rapier debug",
    ));
}
