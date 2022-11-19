use bevy::prelude::*;

use crate::AppState;

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_enter(AppState::Particle3dSimulation)
                .with_system(setup),
        );
    }
}

fn setup() {}
