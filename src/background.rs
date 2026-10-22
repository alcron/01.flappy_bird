use bevy::prelude::*;

use crate::{SCROLL_SPEED, VIEW_HEIGHT, VIEW_WIDTH, loading::GameAssets, state::GameState};

// background.jpg is 6400x3600; scale it so one tile exactly fills the view height.
const BACKGROUND_SCALE: f32 = VIEW_HEIGHT / 3600.;
const BACKGROUND_TILE_WIDTH: f32 = 6400. * BACKGROUND_SCALE; // 480
const BACKGROUND_SPEED: f32 = SCROLL_SPEED * 0.5; // slower than pipes, so it looks farther away

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
            );
    }
}

#[derive(Component)]
struct Background;

fn spawn_background(mut commands: Commands, assets: Res<GameAssets>) {
    // One sprite, two tiles wide, so the view is always covered while it slides left.
    let width = VIEW_WIDTH + BACKGROUND_TILE_WIDTH;
    commands.spawn((
        Background,
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
        // Left edge aligned with the left edge of the view.
        Transform::from_xyz((width - VIEW_WIDTH) / 2., 0., -10.),
    ));
}

fn scroll_background(time: Res<Time>, mut query: Query<&mut Transform, With<Background>>) {
    let start_x = BACKGROUND_TILE_WIDTH / 2.;
    for mut transform in &mut query {
        transform.translation.x -= BACKGROUND_SPEED * time.delta_secs();
        // After moving one full tile the image looks identical, so jump back seamlessly.
        if transform.translation.x <= start_x - BACKGROUND_TILE_WIDTH {
            transform.translation.x += BACKGROUND_TILE_WIDTH;
        }
    }
}
