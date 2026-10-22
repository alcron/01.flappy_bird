use bevy::prelude::*;

use crate::state::GameState;

/// Handles to every image the game uses, loaded once at startup.
#[derive(Resource)]
pub struct GameAssets {
    pub background: Handle<Image>,
    pub bird: Handle<Image>,
    pub pipe: Handle<Image>,
}

pub struct LoadingPlugin;

impl Plugin for LoadingPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, load_assets)
            .add_systems(Update, check_loaded.run_if(in_state(GameState::Loading)));
    }
}

fn load_assets(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.insert_resource(GameAssets {
        background: asset_server.load("background.jpg"),
        bird: asset_server.load("bird.png"),
        pipe: asset_server.load("pipe.png"),
    });
}

fn check_loaded(
    assets: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let loaded = [&assets.background, &assets.bird, &assets.pipe]
        .into_iter()
        .all(|handle| asset_server.is_loaded_with_dependencies(handle));
    if loaded {
        next_state.set(GameState::MainMenu);
    }
}
