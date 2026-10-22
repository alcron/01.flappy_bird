mod background;
mod bird;
mod interface;
mod loading;
mod pipes;
mod score;
mod state;

use avian2d::prelude::*;
use bevy::prelude::*;

use background::BackgroundPlugin;
use bird::BirdPlugin;
use interface::InterfacePlugin;
use loading::LoadingPlugin;
use pipes::PipesPlugin;
use score::ScorePlugin;
use state::StatePlugin;

const VIEW_WIDTH: f32 = 480.;
const VIEW_HEIGHT: f32 = 270.;
// Base scroll speed in world units per second; each layer scales it.
const SCROLL_SPEED: f32 = 60.;
// Downward acceleration in world units per second squared.
const GRAVITY: f32 = 600.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            PhysicsPlugins::default(),
            StatePlugin,
            ScorePlugin,
            LoadingPlugin,
            InterfacePlugin,
            BackgroundPlugin,
            PipesPlugin,
            BirdPlugin,
        ))
        .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: bevy::camera::ScalingMode::AutoMax {
                max_width: VIEW_WIDTH,
                max_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
