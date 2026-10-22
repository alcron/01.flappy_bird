mod background;
mod bird;
mod camera;
mod interface;
mod loading;
mod physics;
mod pipes;
mod score;
mod state;

use bevy::prelude::*;

use background::BackgroundPlugin;
use bird::BirdPlugin;
use camera::CameraPlugin;
use interface::InterfacePlugin;
use loading::LoadingPlugin;
use physics::GamePhysicsPlugin;
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
// How fast the bird flies to the right, in world units per second.
const FLIGHT_SPEED: f32 = 60.;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            StatePlugin,
            GamePhysicsPlugin,
            CameraPlugin,
            ScorePlugin,
            LoadingPlugin,
            InterfacePlugin,
            BackgroundPlugin,
            PipesPlugin,
            BirdPlugin,
        ))
        .run();
}
