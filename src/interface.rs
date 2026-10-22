use bevy::{
    picking::hover::Hovered,
    prelude::*,
    ui::Pressed,
    ui_widgets::{Activate, Button, observe},
};

use crate::{
    DESKTOP,
    score::{HighScore, Score, update_high_score},
    state::GameState,
};

const TITLE_FONT_SIZE: f32 = 48.;
const BUTTON_FONT_SIZE: f32 = 24.;
const HINT_FONT_SIZE: f32 = 16.;
const SCORE_FONT_SIZE: f32 = 32.;

const TEXT_COLOR: Color = Color::srgb(0.95, 0.95, 0.95);
const OVERLAY_COLOR: Color = Color::srgba(0., 0., 0., 0.5);
const NORMAL_BUTTON: Color = Color::srgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::srgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::srgb(0.35, 0.75, 0.35);

pub struct InterfacePlugin;

impl Plugin for InterfacePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(OnEnter(GameState::Loading), spawn_loading_screen)
            .add_systems(OnEnter(GameState::MainMenu), spawn_main_menu)
            .add_systems(OnEnter(GameState::Playing), spawn_pause_button)
            .add_systems(OnEnter(GameState::Paused), spawn_pause_menu)
            // After the high score update, so the menu shows the new best.
            .add_systems(
                OnEnter(GameState::GameOver),
                spawn_game_over_menu.after(update_high_score),
            )
            .add_systems(OnExit(GameState::Loading), spawn_score_display)
            .add_systems(
                Update,
                (
                    update_button_appearance,
                    update_score_display.run_if(resource_changed::<Score>),
                    show_score_display.run_if(state_changed::<GameState>),
                ),
            );
    }
}

/// Current score at the top of the screen, shown during a round.
#[derive(Component)]
struct ScoreDisplay;

/// What a menu button does when clicked.
#[derive(Clone, Copy)]
enum ButtonAction {
    GoTo(GameState),
    Quit,
}

fn spawn_loading_screen(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Loading),
        screen(Color::BLACK),
        children![text("Loading...", BUTTON_FONT_SIZE)],
    ));
}

fn spawn_score_display(mut commands: Commands) {
    commands.spawn((
        ScoreDisplay,
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            width: percent(100),
            justify_content: JustifyContent::Center,
            ..default()
        },
        // Hidden until a round starts.
        Visibility::Hidden,
        children![text("0", SCORE_FONT_SIZE)],
    ));
}

fn update_score_display(
    score: Res<Score>,
    displays: Query<&Children, With<ScoreDisplay>>,
    mut texts: Query<&mut Text>,
) {
    for children in &displays {
        for child in children.iter() {
            if let Ok(mut text) = texts.get_mut(child) {
                **text = score.to_string();
            }
        }
    }
}

/// The game over screen shows the score itself, so the display is only needed mid-round.
fn show_score_display(
    state: Res<State<GameState>>,
    mut displays: Query<&mut Visibility, With<ScoreDisplay>>,
) {
    let visible = matches!(state.get(), GameState::Playing | GameState::Paused);
    for mut visibility in &mut displays {
        *visibility = if visible {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
}

/// Small button in the top-right corner; the only way to pause without a keyboard.
fn spawn_pause_button(mut commands: Commands) {
    commands.spawn((
        DespawnOnExit(GameState::Playing),
        Node {
            position_type: PositionType::Absolute,
            top: px(16),
            right: px(16),
            ..default()
        },
        children![sized_button(
            "II",
            ButtonAction::GoTo(GameState::Paused),
            px(50),
            px(50)
        )],
    ));
}

fn spawn_main_menu(mut commands: Commands, high_score: Res<HighScore>) {
    let menu = commands
        .spawn((
            DespawnOnExit(GameState::MainMenu),
            screen(OVERLAY_COLOR),
            children![
                text("Flappy Bird", TITLE_FONT_SIZE),
                text(&format!("Best: {}", high_score.best), BUTTON_FONT_SIZE),
                button("Play", ButtonAction::GoTo(GameState::Playing)),
            ],
        ))
        .id();
    if DESKTOP {
        commands.entity(menu).with_children(|menu| {
            menu.spawn(button("Quit", ButtonAction::Quit));
            menu.spawn(text("Space - play, Esc - quit", HINT_FONT_SIZE));
        });
    }
}

fn spawn_pause_menu(mut commands: Commands) {
    let menu = commands
        .spawn((
            DespawnOnExit(GameState::Paused),
            screen(OVERLAY_COLOR),
            children![
                text("Paused", TITLE_FONT_SIZE),
                button("Resume", ButtonAction::GoTo(GameState::Playing)),
                button("Main menu", ButtonAction::GoTo(GameState::MainMenu)),
            ],
        ))
        .id();
    if DESKTOP {
        commands
            .entity(menu)
            .with_child(text("Esc - resume", HINT_FONT_SIZE));
    }
}

fn spawn_game_over_menu(mut commands: Commands, score: Res<Score>, high_score: Res<HighScore>) {
    let menu = commands
        .spawn((
            DespawnOnExit(GameState::GameOver),
            screen(OVERLAY_COLOR),
            children![
                text("Game Over", TITLE_FONT_SIZE),
                text(
                    &format!("Score: {}   Best: {}", **score, high_score.best),
                    BUTTON_FONT_SIZE
                ),
                button("Restart", ButtonAction::GoTo(GameState::Playing)),
                button("Main menu", ButtonAction::GoTo(GameState::MainMenu)),
            ],
        ))
        .id();
    if DESKTOP {
        commands
            .entity(menu)
            .with_child(text("Space - restart", HINT_FONT_SIZE));
    }
}

/// Full-window node that stacks its children vertically in the center.
fn screen(background: Color) -> impl Bundle {
    (
        Node {
            width: percent(100),
            height: percent(100),
            flex_direction: FlexDirection::Column,
            align_items: AlignItems::Center,
            justify_content: JustifyContent::Center,
            row_gap: px(16),
            ..default()
        },
        BackgroundColor(background),
    )
}

fn text(content: &str, font_size: f32) -> impl Bundle {
    (
        Text::new(content),
        TextFont {
            font_size: FontSize::Px(font_size),
            ..default()
        },
        TextColor(TEXT_COLOR),
        TextShadow::default(),
    )
}

fn button(label: &str, action: ButtonAction) -> impl Bundle {
    sized_button(label, action, px(200), px(50))
}

fn sized_button(label: &str, action: ButtonAction, width: Val, height: Val) -> impl Bundle {
    (
        Button,
        Hovered::default(),
        Node {
            width,
            height,
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            border_radius: BorderRadius::all(px(8)),
            ..default()
        },
        BackgroundColor(NORMAL_BUTTON),
        observe(
            move |_: On<Activate>,
                  mut next_state: ResMut<NextState<GameState>>,
                  mut app_exit_events: MessageWriter<AppExit>| match action {
                ButtonAction::GoTo(state) => next_state.set(state),
                ButtonAction::Quit => {
                    app_exit_events.write(AppExit::Success);
                }
            },
        ),
        children![text(label, BUTTON_FONT_SIZE)],
    )
}

fn update_button_appearance(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<Button>>,
) {
    for (hovered, pressed, mut color) in &mut buttons {
        *color = match (hovered.get(), pressed) {
            (_, true) => PRESSED_BUTTON,
            (true, false) => HOVERED_BUTTON,
            (false, false) => NORMAL_BUTTON,
        }
        .into();
    }
}
