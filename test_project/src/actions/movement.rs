//! Movement action implementation.

use crate::{
    actions::action::Action,
    core::{Agent, position::Position},
    game::Game,
};

/// Action to move an agent towards a target position.
pub struct MoveAction {
    target: Position,
}

impl MoveAction {
    /// Creates a new move action targeting the specified coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            target: Position::new(x, y),
        }
    }
}

impl Action for MoveAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!("MOVE {} {}", self.target.x, self.target.y)
    }

    fn is_valid(&self, _agent: &Agent, game: &Game) -> bool {
        game.grid.is_valid_position(self.target.x, self.target.y)
    }
}
