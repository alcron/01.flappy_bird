use std::fs;

use bevy::prelude::*;
use serde::{Deserialize, Serialize};

use crate::state::GameState;

/// Relative to the working directory, i.e. the project root under `cargo run`.
const HIGH_SCORE_PATH: &str = "high_score.ron";

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Score>()
            // .add_plugins(RonAsse)
            .insert_resource(load_high_score())
            // Same moments the bird and pipes are reset: a new round starts at zero.
            .add_systems(OnEnter(GameState::MainMenu), reset_score)
            .add_systems(
                OnTransition {
                    exited: GameState::GameOver,
                    entered: GameState::Playing,
                },
                reset_score,
            )
            .add_systems(OnEnter(GameState::GameOver), update_high_score);
    }
}

/// Pipe pairs passed in the current round.
#[derive(Resource, Default, Deref, DerefMut)]
pub struct Score(pub u32);

/// Best score across all runs, stored in `HIGH_SCORE_PATH`.
#[derive(Resource, Serialize, Deserialize, Default)]
pub struct HighScore {
    pub best: u32,
}

fn load_high_score() -> HighScore {
    let Ok(contents) = fs::read_to_string(HIGH_SCORE_PATH) else {
        // No file yet: first run.
        return HighScore::default();
    };
    ron::from_str(&contents).unwrap_or_else(|error| {
        warn!("Couldn't parse {HIGH_SCORE_PATH}, starting from 0: {error}");
        HighScore::default()
    })
}

fn save_high_score(high_score: &HighScore) {
    let result = ron::ser::to_string_pretty(high_score, ron::ser::PrettyConfig::default())
        .map_err(|error| error.to_string())
        .and_then(|contents| {
            fs::write(HIGH_SCORE_PATH, contents).map_err(|error| error.to_string())
        });
    if let Err(error) = result {
        warn!("Couldn't save {HIGH_SCORE_PATH}: {error}");
    }
}

fn reset_score(mut score: ResMut<Score>) {
    **score = 0;
}

/// Public so the game over screen can run after it and show the updated best.
pub fn update_high_score(score: Res<Score>, mut high_score: ResMut<HighScore>) {
    if **score > high_score.best {
        high_score.best = **score;
        save_high_score(&high_score);
    }
}
