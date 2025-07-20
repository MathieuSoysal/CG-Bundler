//! Combat strategies for targeting and engagement.

use crate::{
    actions::{Action, HunkerDownAction, ShootAction},
    ai::strategy::AgentStrategy,
    core::Agent,
    game::Game,
};

/// Strategy for combat situations - targets enemies with highest wetness.
pub struct CombatStrategy;

impl CombatStrategy {
    /// Creates a new combat strategy.
    pub fn new() -> Self {
        Self
    }

    /// Finds the enemy agent with the highest wetness level.
    fn find_wettest_enemy<'a>(&self, my_id: u32, game: &'a Game) -> Option<&'a Agent> {
        game.agents
            .iter()
            .filter(|agent| !agent.is_my_agent(my_id))
            .max_by_key(|agent| agent.get_wetness())
    }
}

impl AgentStrategy for CombatStrategy {
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action> {
        // For combat league: shoot the enemy agent with highest wetness
        if agent.can_shoot() {
            if let Some(target) = self.find_wettest_enemy(game.get_my_id(), game) {
                let distance = agent.get_distance_to(target.get_position());
                let max_range = agent.get_optimal_range() * 2; // Double optimal range is max range

                if distance <= max_range {
                    return Box::new(ShootAction::new(target.get_agent_id()));
                }
            }
        }

        // Default to hunkering down if can't shoot or no targets
        Box::new(HunkerDownAction::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_combat_strategy() {
        let strategy = CombatStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let game = Game::new(0, 12, 8);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        assert_eq!(command, "HUNKER_DOWN");
    }
}
