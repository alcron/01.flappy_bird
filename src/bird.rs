use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{SCROLL_SPEED, VIEW_HEIGHT, loading::GameAssets, state::GameState};

/// The bird never moves sideways, so this is its x for the whole round.
pub const BIRD_X: f32 = 0.;

// bird.png is 641x427; keep its aspect ratio when scaling it down.
const BIRD_WIDTH: f32 = 34.;
const BIRD_HEIGHT: f32 = BIRD_WIDTH * 427. / 641.; // ~23
const BIRD_RADIUS: f32 = BIRD_HEIGHT / 2.;
// With a mass of 1, an impulse changes the velocity by exactly its value.
const BIRD_MASS: f32 = 1.;
const FLAP_IMPULSE: f32 = 20.; // upward, in world units per second at mass 1
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

#[derive(Component)]
struct Bird;

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
            Transform::from_xyz(BIRD_X, 0., 1.),
            Visibility::default(),
            RigidBody::Dynamic,
            Collider::circle(BIRD_RADIUS),
            Mass(BIRD_MASS),
            // Only vertical movement: no sliding sideways, no spinning.
            LockedAxes::ROTATION_LOCKED.lock_translation_x(),
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
        // The bird only moves vertically, but relative to the world it flies
        // forward at the scroll speed, so use that as the horizontal part.
        let angle = velocity
            .y
            .atan2(SCROLL_SPEED)
            .clamp(-MAX_DOWN_ANGLE, MAX_UP_ANGLE);
        for child in children.iter() {
            if let Ok(mut transform) = sprites.get_mut(child) {
                transform.rotation = Quat::from_rotation_z(angle);
            }
        }
    }
}

fn flap(
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
        forces.apply_linear_impulse(Vec2::Y * FLAP_IMPULSE);
    }
}
