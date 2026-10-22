use bevy::{prelude::*, window::AppLifecycle};

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
            .add_systems(Update, (handle_keyboard, pause_on_suspend));
    }
}

/// On phones the app can be sent to the background at any moment (home gesture, a call),
/// so pause the round instead of letting the bird fall the moment the app comes back.
fn pause_on_suspend(
    mut lifecycle_events: MessageReader<AppLifecycle>,
    state: Res<State<GameState>>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    for event in lifecycle_events.read() {
        if *event == AppLifecycle::WillSuspend && *state.get() == GameState::Playing {
            next_state.set(GameState::Paused);
        }
    }
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
