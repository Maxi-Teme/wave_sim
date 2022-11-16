use std::time::Duration;

use bevy::math::ivec3;
use bevy::math::vec2;
use bevy::prelude::*;
use bevy::render::camera::WindowOrigin;
use bevy_simple_tilemap::prelude::TileMapBundle;
use bevy_simple_tilemap::Tile;
use bevy_simple_tilemap::TileMap;

use crate::finite_difference::sigmoid;
use crate::SimulationGrid;
use crate::CELLSIZE;
use crate::DIMX;
use crate::DIMY;
use crate::FRAMES;

#[derive(Resource)]
struct AnimationTimer(Timer);

pub struct AnimationPlugin;

impl Plugin for AnimationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(init_camera)
            .add_startup_system(init_tiles)
            .insert_resource(AnimationTimer(Timer::new(
                Duration::from_millis(FRAMES),
                TimerMode::Repeating,
            )))
            .add_system(update_tiles);
    }
}

fn init_camera(mut commands: Commands) {
    let mut camera_bundle = Camera2dBundle::default();

    camera_bundle.projection = OrthographicProjection {
        window_origin: WindowOrigin::BottomLeft,
        ..default()
    };

    commands.spawn(camera_bundle);
}

fn init_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    let texture_handle = asset_server.load("textures/tilesheet.png");
    let texture_atlas = TextureAtlas::from_grid(
        texture_handle,
        vec2(16.0, 16.0),
        4,
        1,
        Some(vec2(1.0, 1.0)),
        None,
    );
    let texture_atlas_handle = texture_atlases.add(texture_atlas);

    let mut tiles = Vec::new();

    for x in 0..DIMX {
        for y in 0..DIMY {
            tiles.push((
                ivec3(x.try_into().unwrap(), y.try_into().unwrap(), 0),
                Some(Tile {
                    sprite_index: 3,
                    color: Color::WHITE,
                    ..Default::default()
                }),
            ));
        }
    }

    let mut tilemap = TileMap::default();
    tilemap.set_tiles(tiles);

    let tilemap_bundle = TileMapBundle {
        tilemap,
        texture_atlas: texture_atlas_handle,
        transform: Transform {
            translation: Vec3::new(0.0, 0.0, 0.0),
            scale: Vec3::splat(CELLSIZE),
            ..default()
        },
        ..default()
    };

    commands.spawn(tilemap_bundle);
}

fn update_tiles(
    time: Res<Time>,
    mut timer: ResMut<AnimationTimer>,
    u: Res<SimulationGrid>,
    mut tilemaps: Query<&mut TileMap>,
) {
    if timer.0.tick(time.delta()).just_finished() {
        let mut tilemap = tilemaps.get_single_mut().unwrap();
        tilemap.clear();

        let mut tiles = Vec::new();

        for x in 0..DIMX {
            for y in 0..DIMY {
                let amplitude = u.0.get((0, x, y)).unwrap();
                let r = sigmoid(amplitude, 0.8);

                tiles.push((
                    ivec3(x.try_into().unwrap(), y.try_into().unwrap(), 0),
                    Some(Tile {
                        sprite_index: 3,
                        color: Color::rgb(r, 0.0, 1.0),
                        ..Default::default()
                    }),
                ));
            }
        }

        tilemap.set_tiles(tiles);
    }
}
