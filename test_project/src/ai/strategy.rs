//! Core strategy trait and base definitions.

use crate::{actions::Action, core::Agent, game::Game};

/// Trait for AI decision-making strategies.
pub trait AgentStrategy {
    /// Decides what action an agent should take based on the current game state.
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action>;
}
