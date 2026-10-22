use avian2d::prelude::*;
use bevy::prelude::*;

// Named `GameState` rather than `State`, which would clash with Bevy's own `State<S>` resource.
#[derive(States, Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Loading,
    MainMenu,
    Playing,
    Paused,
    GameOver,
}

pub struct StatePlugin;

impl Plugin for StatePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            // Physics only runs while playing, so the bird hangs still in menus and pauses.
            .add_systems(Startup, pause_physics)
            .add_systems(OnEnter(GameState::Playing), unpause_physics)
            .add_systems(OnExit(GameState::Playing), pause_physics)
            .add_systems(Update, handle_keyboard);
    }
}

fn pause_physics(mut time: ResMut<Time<Physics>>) {
    time.pause();
}

fn unpause_physics(mut time: ResMut<Time<Physics>>) {
    time.unpause();
}

/// Keyboard shortcuts for the same actions the menu buttons offer.
fn handle_keyboard(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
    mut app_exit_events: MessageWriter<AppExit>,
) {
    let space = keyboard_input.just_pressed(KeyCode::Space);
    let escape = keyboard_input.just_pressed(KeyCode::Escape);

    match state.get() {
        GameState::MainMenu if space => next_state.set(GameState::Playing),
        GameState::MainMenu if escape => {
            app_exit_events.write(AppExit::Success);
        }
        GameState::Playing if escape => next_state.set(GameState::Paused),
        GameState::Paused if escape => next_state.set(GameState::Playing),
        GameState::GameOver if space => next_state.set(GameState::Playing),
        _ => {}
    }
}
