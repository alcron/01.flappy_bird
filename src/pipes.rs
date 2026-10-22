use std::collections::VecDeque;

use avian2d::prelude::*;
use bevy::prelude::*;

use crate::{
    MAX_VIEW_WIDTH, VIEW_HEIGHT, bird::Bird, loading::GameAssets, score::Score, state::GameState,
};

// pipe.png is 400x2914; keep its aspect ratio when scaling it down.
const PIPE_WIDTH: f32 = 40.;
const PIPE_HEIGHT: f32 = PIPE_WIDTH * 2914. / 400.; // ~291
const PIPE_GAP: f32 = 100.; // vertical space between the two pipes
const PIPE_SPACING: f32 = 160.; // horizontal distance between neighbouring pairs

// Each pair's gap center is picked at random within this distance from the screen center.
// The margin keeps the gap from touching the screen edge, so it never sits half off screen.
const GAP_EDGE_MARGIN: f32 = 20.;
const MAX_GAP_OFFSET: f32 = VIEW_HEIGHT / 2. - PIPE_GAP / 2. - GAP_EDGE_MARGIN;
// Even with the gap at its highest or lowest, the pipes must still reach past the screen edges.
const _: () = assert!(PIPE_HEIGHT >= VIEW_HEIGHT / 2. + MAX_GAP_OFFSET - PIPE_GAP / 2.);

// How far ahead of / behind the bird pairs appear and disappear. The camera is centered
// on the bird, so this is just outside the edges of the widest view.
const SPAWN_DISTANCE: f32 = MAX_VIEW_WIDTH / 2. + PIPE_WIDTH / 2.;
const DESPAWN_DISTANCE: f32 = SPAWN_DISTANCE;

pub struct PipesPlugin;

impl Plugin for PipesPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PipePairs>()
            // Fresh pipes for every new round: when the menu opens and on restart.
            .add_systems(OnEnter(GameState::MainMenu), reset_pipes)
            .add_systems(
                OnTransition {
                    exited: GameState::GameOver,
                    entered: GameState::Playing,
                },
                reset_pipes,
            )
            .add_systems(
                Update,
                (score_pipes, spawn_pipes, despawn_pipes)
                    .chain()
                    .run_if(in_state(GameState::Playing)),
            );
    }
}

/// Live pipe pairs ordered from left to right.
/// New pairs are always placed further right, so they go to the back,
/// and the front is always the first one the bird leaves behind.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct PipePairs(VecDeque<Entity>);

impl PipePairs {
    pub fn leftmost(&self) -> Option<Entity> {
        self.0.front().copied()
    }

    pub fn rightmost(&self) -> Option<Entity> {
        self.0.back().copied()
    }
}

/// Parent of a top and bottom pipe; positions both.
#[derive(Component)]
struct PipePair;

#[derive(Component)]
struct Pipe;

/// Added to a pair once the bird has passed it, so it only scores once.
#[derive(Component)]
struct Passed;

/// Clears all pipes; `spawn_pipes` then starts the round with a pair
/// just off the right edge, giving the player time before the first one.
fn reset_pipes(mut commands: Commands, mut pairs: ResMut<PipePairs>) {
    for entity in pairs.drain(..) {
        commands.entity(entity).despawn();
    }
}

fn score_pipes(
    mut commands: Commands,
    mut score: ResMut<Score>,
    birds: Query<&Transform, With<Bird>>,
    query: Query<(Entity, &Transform), (With<PipePair>, Without<Passed>)>,
) {
    let Ok(bird) = birds.single() else {
        return;
    };
    for (entity, transform) in &query {
        if transform.translation.x < bird.translation.x {
            **score += 1;
            commands.entity(entity).insert(Passed);
        }
    }
}

fn spawn_pipes(
    mut commands: Commands,
    assets: Res<GameAssets>,
    mut pairs: ResMut<PipePairs>,
    birds: Query<&Transform, With<Bird>>,
    query: Query<&Transform, With<PipePair>>,
) {
    let Ok(bird) = birds.single() else {
        return;
    };
    let spawn_x = bird.translation.x + SPAWN_DISTANCE;
    let rightmost_x = pairs
        .rightmost()
        .and_then(|entity| query.get(entity).ok())
        .map(|transform| transform.translation.x);

    let new_x = match rightmost_x {
        // Place the new pair relative to the last one so spacing stays exact.
        Some(x) if x <= spawn_x - PIPE_SPACING => x + PIPE_SPACING,
        Some(_) => return,
        None => spawn_x,
    };
    let entity = spawn_pipe_pair(&mut commands, &assets, new_x);
    pairs.push_back(entity);
}

fn despawn_pipes(
    mut commands: Commands,
    mut pairs: ResMut<PipePairs>,
    birds: Query<&Transform, With<Bird>>,
    query: Query<&Transform, With<PipePair>>,
) {
    let Ok(bird) = birds.single() else {
        return;
    };
    let despawn_x = bird.translation.x - DESPAWN_DISTANCE;
    // Only the leftmost pairs can be off screen, so stop at the first one that isn't.
    while let Some(entity) = pairs.leftmost() {
        let Ok(transform) = query.get(entity) else {
            break;
        };
        if transform.translation.x >= despawn_x {
            break;
        }
        // Also despawns the child pipes.
        commands.entity(entity).despawn();
        pairs.pop_front();
    }
}

fn spawn_pipe_pair(commands: &mut Commands, assets: &GameAssets, x: f32) -> Entity {
    let image = assets.pipe.clone();
    // Pipe centers sit half a pipe beyond the gap edges, so the gap is centered on the pair.
    let offset_y = PIPE_GAP / 2. + PIPE_HEIGHT / 2.;
    let gap_y = rand::random_range(-MAX_GAP_OFFSET..=MAX_GAP_OFFSET);

    commands
        .spawn((
            PipePair,
            Transform::from_xyz(x, gap_y, 0.),
            Visibility::default(),
            // No rigid body: colliders without one are static, which suits pipes that never move.
            children![
                // Bottom pipe: texture as is, opening faces up toward the gap.
                (
                    Pipe,
                    Sprite {
                        image: image.clone(),
                        custom_size: Some(Vec2::new(PIPE_WIDTH, PIPE_HEIGHT)),
                        ..default()
                    },
                    Transform::from_xyz(0., -offset_y, 0.),
                    Collider::rectangle(PIPE_WIDTH, PIPE_HEIGHT),
                    // Only detects the hit, the bird doesn't bounce off.
                    Sensor,
                ),
                // Top pipe: flipped vertically so its opening faces down toward the gap.
                (
                    Pipe,
                    Sprite {
                        image,
                        custom_size: Some(Vec2::new(PIPE_WIDTH, PIPE_HEIGHT)),
                        flip_y: true,
                        ..default()
                    },
                    Transform::from_xyz(0., offset_y, 0.),
                    Collider::rectangle(PIPE_WIDTH, PIPE_HEIGHT),
                    Sensor,
                ),
            ],
        ))
        .id()
}
