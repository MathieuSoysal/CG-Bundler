//! Combined actions for Run & Gun gameplay.

use crate::{
    actions::action::Action,
    core::{Agent, position::Position},
    game::Game,
};

/// Combined action that performs both MOVE and SHOOT in a single turn.
pub struct MoveAndShootAction {
    target_position: Position,
    target_agent_id: u32,
}

impl MoveAndShootAction {
    /// Creates a new combined move and shoot action.
    pub fn new(move_x: u32, move_y: u32, target_agent_id: u32) -> Self {
        Self {
            target_position: Position::new(move_x, move_y),
            target_agent_id,
        }
    }
}

impl Action for MoveAndShootAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!(
            "MOVE {} {};SHOOT {}",
            self.target_position.x, self.target_position.y, self.target_agent_id
        )
    }

    fn is_valid(&self, agent: &Agent, game: &Game) -> bool {
        // Check if the move position is valid
        if !game.grid.is_valid_position(self.target_position.x, self.target_position.y) {
            return false;
        }
        
        // Check if agent can shoot
        agent.can_shoot()
    }
}

/// Combined action that performs both MOVE and HUNKER_DOWN in a single turn.
pub struct MoveAndHunkerAction {
    target_position: Position,
}

impl MoveAndHunkerAction {
    /// Creates a new combined move and hunker action.
    pub fn new(move_x: u32, move_y: u32) -> Self {
        Self {
            target_position: Position::new(move_x, move_y),
        }
    }
}

impl Action for MoveAndHunkerAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!(
            "MOVE {} {};HUNKER_DOWN",
            self.target_position.x, self.target_position.y
        )
    }

    fn is_valid(&self, _agent: &Agent, game: &Game) -> bool {
        // Check if the move position is valid
        game.grid.is_valid_position(self.target_position.x, self.target_position.y)
    }
}

/// Combined action that performs both MOVE and THROW in a single turn.
pub struct MoveAndThrowAction {
    move_target: Position,
    throw_target: Position,
}

impl MoveAndThrowAction {
    /// Creates a new combined move and throw action.
    pub fn new(move_x: u32, move_y: u32, throw_x: u32, throw_y: u32) -> Self {
        Self {
            move_target: Position::new(move_x, move_y),
            throw_target: Position::new(throw_x, throw_y),
        }
    }
}

impl Action for MoveAndThrowAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!(
            "MOVE {} {};THROW {} {}",
            self.move_target.x, self.move_target.y, 
            self.throw_target.x, self.throw_target.y
        )
    }

    fn is_valid(&self, agent: &Agent, game: &Game) -> bool {
        // Check if the move position is valid
        if !game.grid.is_valid_position(self.move_target.x, self.move_target.y) {
            return false;
        }
        
        // Check if the throw target is valid
        if !game.grid.is_valid_position(self.throw_target.x, self.throw_target.y) {
            return false;
        }
        
        // Check if agent has splash bombs
        if agent.get_splash_bombs() == 0 {
            return false;
        }
        
        // Check if throw target is within range from the move destination
        let distance = self.move_target.distance_to(&self.throw_target);
        distance <= 4
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_move_and_shoot_action() {
        let action = MoveAndShootAction::new(5, 3, 2);
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        
        assert_eq!(action.execute(&agent), "MOVE 5 3;SHOOT 2");
    }

    #[test]
    fn test_move_and_hunker_action() {
        let action = MoveAndHunkerAction::new(4, 2);
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        
        assert_eq!(action.execute(&agent), "MOVE 4 2;HUNKER_DOWN");
    }

    #[test]
    fn test_move_and_throw_action() {
        let action = MoveAndThrowAction::new(2, 3, 5, 6);
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        
        assert_eq!(action.execute(&agent), "MOVE 2 3;THROW 5 6");
    }
}
