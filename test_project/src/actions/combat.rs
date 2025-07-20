//! Combat-related actions.

use crate::{
    actions::action::Action,
    core::{Agent, position::Position},
    game::Game,
};

/// Action to shoot at another agent.
pub struct ShootAction {
    target_id: u32,
}

impl ShootAction {
    /// Creates a new shoot action targeting the agent with the given ID.
    pub fn new(target_id: u32) -> Self {
        Self { target_id }
    }
}

impl Action for ShootAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!("SHOOT {}", self.target_id)
    }

    fn is_valid(&self, agent: &Agent, _game: &Game) -> bool {
        agent.can_shoot()
    }
}

/// Action to throw a splash bomb at a target location.
pub struct ThrowAction {
    target: Position,
}

impl ThrowAction {
    /// Creates a new throw action targeting the specified coordinates.
    pub fn new(x: u32, y: u32) -> Self {
        Self {
            target: Position::new(x, y),
        }
    }
}

impl Action for ThrowAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!("THROW {} {}", self.target.x, self.target.y)
    }

    fn is_valid(&self, agent: &Agent, game: &Game) -> bool {
        // Check if agent has splash bombs
        if agent.get_splash_bombs() == 0 {
            return false;
        }
        
        // Check if target position is valid
        if !game.grid.is_valid_position(self.target.x, self.target.y) {
            return false;
        }
        
        // Check if target is within throw range (max 4 tiles)
        let distance = agent.get_position().distance_to(&self.target);
        distance <= 4
    }
}

/// Action to hunker down for defensive benefits.
pub struct HunkerDownAction;

impl HunkerDownAction {
    /// Creates a new hunker down action.
    pub fn new() -> Self {
        Self
    }
}

impl Action for HunkerDownAction {
    fn execute(&self, _agent: &Agent) -> String {
        "HUNKER_DOWN".to_string()
    }

    fn is_valid(&self, _agent: &Agent, _game: &Game) -> bool {
        true
    }
}
