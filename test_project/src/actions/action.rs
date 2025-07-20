//! Core action trait and base definitions.

use crate::{core::Agent, game::Game};

/// Trait for all game actions that can be executed by agents.
pub trait Action {
    /// Executes the action and returns the command string.
    fn execute(&self, agent: &Agent) -> String;

    /// Validates if the action can be performed by the agent in the current game state.
    fn is_valid(&self, agent: &Agent, game: &Game) -> bool;
}
