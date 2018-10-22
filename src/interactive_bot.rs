use console::Style;
use dialoguer::{theme::ColorfulTheme, Confirmation, Input, Select};
use indicatif::{ProgressBar, ProgressStyle};

use crate::game_engine::*;
use crate::DIALOG_THEME;

/// This bot ask on stdin the next action to realize.
#[derive(Default)]
pub struct InteractiveBot;

impl SnakeBot for InteractiveBot {
    fn get_next_action(&mut self, _: &SnakeState, _: &GameBoard) -> Action {
        // TODO: Colorize this prompt with the same color than the snakes
        let choice = Select::with_theme(&*DIALOG_THEME)
            .with_prompt("What is your next action?")
            .default(1)
            .item("left")
            .item("front")
            .item("right")
            .interact()
            .unwrap_or(1);

        match choice {
            0 => Action::Left,
            1 => Action::Front,
            2 => Action::Right,
            _ => unreachable!(),
        }
    }
}
