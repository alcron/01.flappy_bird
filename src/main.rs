mod background;
mod bird;
mod interface;
mod loading;
mod pipes;
mod score;
mod state;

use avian2d::prelude::*;
use bevy::{prelude::*, window::WindowMode};

use background::BackgroundPlugin;
use bird::BirdPlugin;
use interface::InterfacePlugin;
use loading::LoadingPlugin;
use pipes::PipesPlugin;
use score::ScorePlugin;
use state::StatePlugin;

// The camera always shows exactly this height; the width follows the screen's aspect ratio.
const VIEW_HEIGHT: f32 = 270.;
// Widest view the game supports (21:9, wider than any iPhone at ~19.5:9).
// The background is sized and pipes spawn/despawn based on it, so nothing pops in on wide screens.
const MAX_VIEW_WIDTH: f32 = VIEW_HEIGHT * 21. / 9.;
// Phones and tablets have no keyboard and apps there shouldn't quit themselves.
const DESKTOP: bool = cfg!(not(any(target_os = "ios", target_os = "android")));
// Base scroll speed in world units per second; each layer scales it.
const SCROLL_SPEED: f32 = 60.;
// Downward acceleration in world units per second squared.
const GRAVITY: f32 = 600.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    // Otherwise iOS creates the window at Bevy's default 1280x720 points,
                    // larger than the phone screen.
                    mode: if DESKTOP {
                        WindowMode::Windowed
                    } else {
                        WindowMode::BorderlessFullscreen(MonitorSelection::Primary)
                    },
                    ..default()
                }),
                ..default()
            }),
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
            scaling_mode: bevy::camera::ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}
