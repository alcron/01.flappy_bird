use bevy::prelude::*;

use crate::{
    FLIGHT_SPEED, MAX_VIEW_WIDTH, VIEW_HEIGHT, camera::follow_bird, loading::GameAssets,
    state::GameState,
};

// Size of background.jpg in pixels; scale it so one tile exactly fills the view height.
const BACKGROUND_IMAGE_WIDTH: f32 = 1600.;
const BACKGROUND_IMAGE_HEIGHT: f32 = 900.;
const BACKGROUND_SCALE: f32 = VIEW_HEIGHT / BACKGROUND_IMAGE_HEIGHT;
const BACKGROUND_TILE_WIDTH: f32 = BACKGROUND_IMAGE_WIDTH * BACKGROUND_SCALE; // 480
// On-screen scroll speed: half the bird's, so it looks farther away than the pipes.
const BACKGROUND_SPEED: f32 = FLIGHT_SPEED * 0.5;

pub struct BackgroundPlugin;

impl Plugin for BackgroundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnExit(GameState::Loading), spawn_background)
            .add_systems(
                Update,
                // Keeps scrolling behind the main menu, stops when paused or after a crash.
                scroll_background.run_if(|state: Res<State<GameState>>| {
                    matches!(state.get(), GameState::MainMenu | GameState::Playing)
                }),
            )
            .add_systems(
                PostUpdate,
                place_background
                    .after(follow_bird)
                    .before(TransformSystems::Propagate),
            );
    }
}

/// `scroll` is how far the image has slid left on screen, wrapped to one tile.
/// Driven by time rather than the camera, so it doesn't jump when a new round
/// moves the camera back to the origin, and it still moves in the main menu.
#[derive(Component, Default)]
struct Background {
    scroll: f32,
}

fn spawn_background(mut commands: Commands, assets: Res<GameAssets>) {
    // The widest view plus one tile, so the view is always covered while it slides left.
    let width = MAX_VIEW_WIDTH + BACKGROUND_TILE_WIDTH;
    commands.spawn((
        Background::default(),
        Sprite {
            image: assets.background.clone(),
            custom_size: Some(Vec2::new(width, VIEW_HEIGHT)),
            image_mode: SpriteImageMode::Tiled {
                tile_x: true,
                tile_y: false,
                stretch_value: BACKGROUND_SCALE,
            },
            ..default()
        },
        // x is set every frame by `place_background`.
        Transform::from_xyz(0., 0., -10.),
    ));
}

fn scroll_background(time: Res<Time>, mut query: Query<&mut Background>) {
    for mut background in &mut query {
        background.scroll += BACKGROUND_SPEED * time.delta_secs();
        // After moving one full tile the image looks identical, so jump back seamlessly.
        background.scroll %= BACKGROUND_TILE_WIDTH;
    }
}

fn place_background(
    cameras: Query<&Transform, With<Camera2d>>,
    mut query: Query<(&Background, &mut Transform), Without<Camera2d>>,
) {
    let Ok(camera) = cameras.single() else {
        return;
    };
    // With no scroll, the left edge lines up with the left edge of the widest view.
    let start_x = BACKGROUND_TILE_WIDTH / 2.;
    for (background, mut transform) in &mut query {
        transform.translation.x = camera.translation.x + start_x - background.scroll;
    }
}
