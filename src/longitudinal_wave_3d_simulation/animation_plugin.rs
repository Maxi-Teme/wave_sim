use std::f32::consts::{PI, TAU};

use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_rapier3d::prelude::*;

use crate::pan_orbit_camera::{update_pan_orbit_camera, PanOrbitCamera};
use crate::{AppCamera, AppState, UiCamera};

use super::SimulationParameters;

#[derive(Default, Resource)]
struct Entities(Vec<Entity>);

#[derive(Resource)]
struct AnimationTimer(Stopwatch);

#[derive(Component)]
struct Particle {
    initial_translation: Vec3,
}

#[derive(Component)]
struct ApplyingForce;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(Entities::default())
            .insert_resource(AnimationTimer(Stopwatch::new()))
            .insert_resource(RapierConfiguration {
                gravity: Vec3::ZERO,
                ..default()
            })
            .add_plugin(RapierPhysicsPlugin::<()>::default())
            .add_plugin(RapierDebugRenderPlugin::default())
            .add_system_set(
                SystemSet::on_enter(AppState::LongitudinalWaveSimulation3d)
                    .with_system(setup),
            )
            .add_system_set(
                SystemSet::on_update(AppState::LongitudinalWaveSimulation3d)
                    .with_system(update_pan_orbit_camera)
                    .with_system(apply_impulse)
                    .with_system(apply_equilibrium_force),
            )
            .add_system_set(
                SystemSet::on_exit(AppState::LongitudinalWaveSimulation3d)
                    .with_system(cleanup),
            );
    }
}

fn setup(
    mut commands: Commands,
    cameras: Query<Entity, (With<AppCamera>, Without<UiCamera>)>,
    mut mouse_button: ResMut<Input<MouseButton>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    parameters: Res<SimulationParameters>,
    mut entities: ResMut<Entities>,
) {
    mouse_button.reset_all();

    if let Ok(camera_entity) = cameras.get_single() {
        commands.entity(camera_entity).despawn();
    }

    let max_x_z = parameters.dimx.max(parameters.dimz) as f32 * 2.0;

    let plane = commands.spawn((
        PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: max_x_z * 2.0,
            })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            transform: Transform::from_xyz(
                parameters.dimx as f32 / 2.0,
                -2.0,
                parameters.dimz as f32 / 2.0,
            ),
            ..default()
        },
        Collider::cuboid(max_x_z, 0.1, max_x_z),
    ));

    entities.0.push(plane.id());

    let mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: parameters.radius,
        subdivisions: 6,
    }));

    let material1_handle = materials.add(Color::rgb(0.6, 0.6, 0.6).into());
    let material2_handle = materials.add(Color::rgb(0.7, 0.5, 0.5).into());

    // spheres
    for x in 0..parameters.dimx {
        for y in 0..parameters.dimy {
            for z in 0..parameters.dimz {
                let material = if z == 0 {
                    material2_handle.clone()
                } else {
                    material1_handle.clone()
                };

                let translation = Vec3::new(x as f32, y as f32, z as f32);

                let mut particle = commands.spawn((
                    Particle {
                        initial_translation: translation,
                    },
                    PbrBundle {
                        mesh: mesh.clone(),
                        material,
                        transform: Transform::from_translation(translation),
                        ..default()
                    },
                    Collider::ball(parameters.radius),
                    Restitution::coefficient(0.7),
                    ExternalImpulse::default(),
                    ExternalForce::default(),
                ));

                if z == 0 {
                    particle.insert(ApplyingForce);
                    particle.insert(RigidBody::Fixed);
                } else {
                    particle.insert(RigidBody::Dynamic);
                }

                entities.0.push(particle.id());
            }
        }
    }

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
    let translation = Vec3::new(-22.0, 17.0, 19.0);
    let radius = translation.length();

    commands
        .spawn((
            AppCamera,
            Camera3dBundle {
                transform: Transform::from_translation(translation)
                    .looking_at(Vec3::ZERO, Vec3::Y),
                ..default()
            },
        ))
        .insert(PanOrbitCamera {
            radius,
            ..Default::default()
        });
}

fn apply_impulse(
    time: Res<Time>,
    mut animation_timer: ResMut<AnimationTimer>,
    mut force_sources: Query<
        (&Particle, &mut ExternalImpulse, &mut Transform),
        With<ApplyingForce>,
    >,
    parameters: Res<SimulationParameters>,
) {
    let elapsed = animation_timer.0.elapsed();
    let z =
        (elapsed.as_secs_f32() * parameters.applying_force_freq * TAU).sin();

    // Apply impulses.
    // for (mut ext_impulse, _) in force_sources.iter_mut() {
    //     ext_impulse.impulse = Vec3::new(0.0, 0.0, z);
    // }

    for (particle, _, mut transform) in force_sources.iter_mut() {
        transform.translation.z = particle.initial_translation.z
            + (z * parameters.applying_force_factor);
    }

    animation_timer.0.tick(time.delta());
}

fn apply_equilibrium_force(
    mut force_sources: Query<(&Particle, &Transform, &mut ExternalForce)>,
    parameters: Res<SimulationParameters>,
) {
    for (particle, transform, mut external_force) in force_sources.iter_mut() {
        let equilizing_force_direction =
            particle.initial_translation - transform.translation;

        external_force.force =
            equilizing_force_direction * parameters.equilibrium_force_factor;
    }
}

fn cleanup(mut commands: Commands, mut entities: ResMut<Entities>) {
    for entity in entities.0.drain(..) {
        commands.entity(entity).despawn();
    }
}
