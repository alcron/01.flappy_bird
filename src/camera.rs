use bevy::{camera::ScalingMode, prelude::*};

use crate::{VIEW_HEIGHT, bird::Bird};

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_camera).add_systems(
            PostUpdate,
            // The bird's `Transform` is already smoothed between physics steps by now;
            // moving the camera before propagation means both are drawn from the same frame.
            follow_bird.before(TransformSystems::Propagate),
        );
    }
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((
        Camera2d,
        Projection::Orthographic(OrthographicProjection {
            scaling_mode: ScalingMode::FixedVertical {
                viewport_height: VIEW_HEIGHT,
            },
            ..OrthographicProjection::default_2d()
        }),
    ));
}

/// Keeps the bird horizontally centered; the camera doesn't follow it up and down.
/// Not a child of the bird: the bird is despawned on every reset and would take the camera with it.
pub fn follow_bird(
    birds: Query<&Transform, With<Bird>>,
    mut cameras: Query<&mut Transform, (With<Camera2d>, Without<Bird>)>,
) {
    let Ok(bird) = birds.single() else {
        return;
    };
    for mut camera in &mut cameras {
        camera.translation.x = bird.translation.x;
    }
}
