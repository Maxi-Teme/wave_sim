use bevy::prelude::*;
use bevy::render::render_phase::SetItemPipeline;
use bevy::sprite::{DrawMesh2d, SetMesh2dBindGroup, SetMesh2dViewBindGroup};

mod pipeline;
mod plugin;

pub use plugin::ColoredMesh2dPlugin;

/// A marker component for colored 2d meshes
#[derive(Component, Default)]
pub struct ColoredMesh2d;

// This specifies how to render a colored 2d mesh
pub type DrawColoredMesh2d = (
    // Set the pipeline
    SetItemPipeline,
    // Set the view uniform as bind group 0
    SetMesh2dViewBindGroup<0>,
    // Set the mesh uniform as bind group 1
    SetMesh2dBindGroup<1>,
    // Draw the mesh
    DrawMesh2d,
);
