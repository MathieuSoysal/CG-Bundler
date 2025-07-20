//! Movement and positioning strategies.

use crate::{
    actions::{Action, HunkerDownAction, MoveAction},
    ai::strategy::AgentStrategy,
    core::{Agent, position::Position},
    game::Game,
};

/// Strategy for reaching specific objective positions (Wooden League).
pub struct ObjectiveStrategy {
    targets: Vec<Position>,
}

impl ObjectiveStrategy {
    /// Creates a new objective strategy with the specified target positions.
    pub fn new(targets: Vec<Position>) -> Self {
        Self { targets }
    }
}

impl AgentStrategy for ObjectiveStrategy {
    fn decide_action(&self, agent: &Agent, _game: &Game) -> Box<dyn Action> {
        let agent_pos = agent.get_position();

        // Find the closest target position
        if let Some(target) = self
            .targets
            .iter()
            .min_by_key(|&pos| agent_pos.distance_to(pos))
        {
            if agent_pos.distance_to(target) == 0 {
                // Already at target, hunker down
                Box::new(HunkerDownAction::new())
            } else {
                // Move towards target
                Box::new(MoveAction::new(target.x, target.y))
            }
        } else {
            Box::new(HunkerDownAction::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_objective_strategy() {
        let targets = vec![Position::new(6, 1), Position::new(6, 3)];
        let strategy = ObjectiveStrategy::new(targets);
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        let game = Game::new(0, 12, 8);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        assert!(command.contains("MOVE"));
    }
}
