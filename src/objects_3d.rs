use std::f32::consts::TAU;

use bevy::prelude::shape::Box;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

#[derive(Default, Bundle)]
pub struct BallBundle {
    pub collider: Collider,
    pub rigid_body: RigidBody,
    pub pbr: PbrBundle,
    pub restitution: Restitution,
    pub velocity: Velocity,
    pub external_impulse: ExternalImpulse,
    pub external_force: ExternalForce,
}

impl BallBundle {
    pub fn new_from_xyz(x: f32, y: f32, z: f32, r: f32) -> Self {
        Self {
            collider: Collider::ball(r),
            pbr: PbrBundle {
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            rigid_body: RigidBody::Dynamic,
            restitution: Restitution::default(),
            ..default()
        }
    }
}

#[derive(Default, Bundle)]
pub struct ContainerBundle {
    pub collider: Collider,
    pub pbr: PbrBundle,
    pub velocity: Velocity,
    pub external_force: ExternalForce,
    pub external_impulse: ExternalImpulse,
}

#[allow(dead_code)]
impl ContainerBundle {
    pub fn new_from_xyz(
        x: f32,
        y: f32,
        z: f32,
        meshes: &mut Assets<Mesh>,
    ) -> Self {
        Self {
            collider: Self::collider(x, y, z),
            pbr: PbrBundle {
                mesh: meshes.add(Self::mesh(x, y, z)),
                transform: Transform::from_xyz(x, y, z),
                ..default()
            },
            ..default()
        }
    }

    fn collider(x: f32, y: f32, z: f32) -> Collider {
        let panel_thickness = x.max(y).max(z) / 10.0;

        let collider_xz = Collider::cuboid(
            x + panel_thickness,
            panel_thickness + panel_thickness,
            z + panel_thickness,
        );
        let collider_xy = Collider::cuboid(
            x + panel_thickness,
            y + panel_thickness,
            panel_thickness + panel_thickness,
        );
        let collider_zy = Collider::cuboid(
            panel_thickness + panel_thickness,
            y + panel_thickness,
            z + panel_thickness,
        );

        let p_bottom = Vec3::new(0.0, -y - 2.0 * panel_thickness, 0.0);
        let p_top = Vec3::new(0.0, y + 2.0 * panel_thickness, 0.0);
        let p_left = Vec3::new(0.0, 0.0, -z - 2.0 * panel_thickness);
        let p_right = Vec3::new(0.0, 0.0, z + 2.0 * panel_thickness);
        let p_near = Vec3::new(-x - 2.0 * panel_thickness, 0.0, 0.0);
        let p_far = Vec3::new(x + 2.0 * panel_thickness, 0.0, 0.0);

        Collider::compound(vec![
            (p_bottom, Quat::IDENTITY, collider_xz.clone()),
            (p_top, Quat::IDENTITY, collider_xz),
            (p_left, Quat::IDENTITY, collider_xy.clone()),
            (p_right, Quat::IDENTITY, collider_xy),
            (p_near, Quat::IDENTITY, collider_zy.clone()),
            (p_far, Quat::IDENTITY, collider_zy),
        ])
    }

    fn mesh(x: f32, y: f32, z: f32) -> Mesh {
        Mesh::from(Box::new(x * 2.0, y * 2.0, z * 2.0))
    }
}

#[allow(dead_code)]
pub fn bowl() -> (
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
        Handle::<StandardMaterial>::default(),
        Visibility::default(),
        ComputedVisibility::default(),
    )
}

#[allow(dead_code)]
pub fn rect(x: f32, y: f32, z: f32) -> (Box, Collider) {
    (
        shape::Box::new(x, y, z),
        Collider::cuboid(x / 2.0, y / 2.0, z / 2.0),
    )
}

pub fn spawn_koordinate_system_helper(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
) {
    let debug_mesh = meshes.add(Mesh::from(shape::Icosphere {
        radius: 0.09,
        subdivisions: 1,
    }));
    // 0/0/0 black
    commands.spawn(PbrBundle {
        mesh: debug_mesh.clone(),
        material: materials.add(Color::BLACK.into()),
        transform: Transform::from_xyz(0.0, 0.0, 0.0),
        ..default()
    });
    // x red
    commands.spawn(PbrBundle {
        mesh: debug_mesh.clone(),
        material: materials.add(Color::RED.into()),
        transform: Transform::from_xyz(1.0, 0.0, 0.0),
        ..default()
    });
    // y green
    commands.spawn(PbrBundle {
        mesh: debug_mesh.clone(),
        material: materials.add(Color::GREEN.into()),
        transform: Transform::from_xyz(0.0, 1.0, 0.0),
        ..default()
    });
    // z blue
    commands.spawn(PbrBundle {
        mesh: debug_mesh,
        material: materials.add(Color::BLUE.into()),
        transform: Transform::from_xyz(0.0, 0.0, 1.0),
        ..default()
    });
}
