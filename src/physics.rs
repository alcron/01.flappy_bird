use avian2d::prelude::*;
use bevy::prelude::*;

use crate::state::GameState;

// Downward acceleration in world units per second squared.
const GRAVITY: f32 = 600.;

/// Sets up the physics engine and decides when it runs.
/// Named `GamePhysicsPlugin` so it doesn't get mixed up with avian's `PhysicsPlugins`.
pub struct GamePhysicsPlugin;

impl Plugin for GamePhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default())
            .insert_resource(Gravity(Vec2::NEG_Y * GRAVITY))
            // Physics only runs while playing, so the bird hangs still in menus and pauses.
            .add_systems(Startup, pause_physics)
            .add_systems(OnEnter(GameState::Playing), unpause_physics)
            .add_systems(OnExit(GameState::Playing), pause_physics);
    }
}

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn unpause_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}
