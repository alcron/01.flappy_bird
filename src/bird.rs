use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{FLIGHT_SPEED, VIEW_HEIGHT, loading::GameAssets, state::GameState};

// bird.png is 641x427; keep its aspect ratio when scaling it down.
const BIRD_WIDTH: f32 = 34.;
const BIRD_HEIGHT: f32 = BIRD_WIDTH * 427. / 641.; // ~23
const BIRD_RADIUS: f32 = BIRD_HEIGHT / 2.;
// With a mass of 1, an impulse changes the velocity by exactly its value.
const BIRD_MASS: f32 = 1.;
// Upward impulse per second of holding, so the lift doesn't depend on frame rate.
// Equal to the old 20 per frame at 60 FPS; at mass 1 it's an upward acceleration
// in world units per second squared, twice the strength of gravity.
const FLAP_IMPULSE_PER_SECOND: f32 = 1200.;
// Tilt limits, so the bird never points straight up or flips over.
const MAX_UP_ANGLE: f32 = 30_f32.to_radians();
const MAX_DOWN_ANGLE: f32 = 90_f32.to_radians();

pub struct BirdPlugin;

impl Plugin for BirdPlugin {
    fn build(&self, app: &mut App) {
        app
            // Fresh bird for every new round: when the menu opens and on restart.
            .add_systems(OnEnter(GameState::MainMenu), reset_bird)
            .add_systems(
                OnTransition {
                    exited: GameState::GameOver,
                    entered: GameState::Playing,
                },
                reset_bird,
            )
            .add_systems(
                Update,
                (flap, tilt_bird, check_out_of_bounds).run_if(in_state(GameState::Playing)),
            );
    }
}

/// Public so the camera can follow it and pipes can be placed and scored relative to it.
#[derive(Component)]
pub struct Bird;

/// Child of the bird holding its sprite; only this is rotated,
/// so tilting never touches the physics body.
#[derive(Component)]
struct BirdSprite;

fn reset_bird(mut commands: Commands, assets: Res<GameAssets>, birds: Query<Entity, With<Bird>>) {
    for entity in &birds {
        commands.entity(entity).despawn();
    }
    commands
        .spawn((
            Bird,
            // Every round starts at the world origin.
            Transform::from_xyz(0., 0., 1.),
            Visibility::default(),
            RigidBody::Dynamic,
            // Nothing slows it down (no damping, pipes are sensors), so this speed stays constant.
            LinearVelocity(Vec2::X * FLIGHT_SPEED),
            // Physics runs in fixed steps; without this the camera following the bird
            // would make the whole world stutter.
            TransformInterpolation,
            Collider::circle(BIRD_RADIUS),
            Mass(BIRD_MASS),
            // No spinning; the visible tilt is on the sprite child.
            LockedAxes::ROTATION_LOCKED,
            // Needed for the `CollisionStart` event that ends the game.
            CollisionEventsEnabled,
            children![(
                BirdSprite,
                Sprite {
                    image: assets.bird.clone(),
                    custom_size: Some(Vec2::new(BIRD_WIDTH, BIRD_HEIGHT)),
                    ..default()
                },
            )],
        ))
        .observe(crash);
}

/// The bird only collides with pipes, so any collision ends the game.
fn crash(_: On<CollisionStart>, mut next_state: ResMut<NextState<GameState>>) {
    next_state.set(GameState::GameOver);
}

fn check_out_of_bounds(
    birds: Query<&Transform, With<Bird>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for transform in &birds {
        if transform.translation.y.abs() > VIEW_HEIGHT / 2. {
            next_state.set(GameState::GameOver);
        }
    }
}

fn tilt_bird(
    birds: Query<(&LinearVelocity, &Children), With<Bird>>,
    mut sprites: Query<&mut Transform, With<BirdSprite>>,
) {
    for (velocity, children) in &birds {
        let angle = velocity
            .y
            .atan2(velocity.x)
            .clamp(-MAX_DOWN_ANGLE, MAX_UP_ANGLE);
        for child in children.iter() {
            if let Ok(mut transform) = sprites.get_mut(child) {
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

fn flap(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    touches: Res<Touches>,
    mut query: Query<Forces, With<Bird>>,
) {
    // Holding a finger anywhere on the screen works like holding Space.
    let touching = touches.iter().next().is_some();
    if !keyboard_input.pressed(KeyCode::Space) && !touching {
        return;
    }
    for mut forces in &mut query {
        forces.apply_linear_impulse(Vec2::Y * FLAP_IMPULSE_PER_SECOND * time.delta_secs());
    }
}
