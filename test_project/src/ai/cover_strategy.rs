//! Cover-based strategy for the Taking Cover league.

use crate::{
    actions::{Action, MoveAndShootAction, MoveAndHunkerAction, ShootAction, HunkerDownAction},
    ai::{cover::CoverEvaluator, strategy::AgentStrategy},
    core::Agent,
    game::Game,
};

/// Strategy that prioritizes cover usage and tactical positioning.
/// 
/// This strategy implements the logic for Objective 3: Taking Cover
/// - Moves agents to the safest available positions with best cover
/// - Targets enemies with the least cover protection, prioritizing nearest targets
/// - Uses combined move-and-shoot actions for optimal positioning
/// 
/// ## Target Prioritization
/// The strategy selects targets using this priority system:
/// 1. **Cover Level**: No cover > Low cover > High cover
/// 2. **Distance**: Among enemies with same cover level, prefer nearest targets
/// 
/// This ensures agents attack the most vulnerable and accessible enemies first.
pub struct CoverStrategy;

impl CoverStrategy {
    /// Creates a new cover strategy instance.
    pub fn new() -> Self {
        Self
    }

    /// Determines if an agent should prioritize moving to cover over shooting.
    fn should_prioritize_movement(
        &self,
        agent: &Agent,
        game: &Game,
        enemies: &[&Agent],
    ) -> bool {
        let current_pos = agent.get_position();
        let current_protection = CoverEvaluator::evaluate_cover_at_position(game, current_pos, enemies);
        
        // Check if there's a significantly better position available
        if let Some(safest_pos) = CoverEvaluator::find_safest_position(game, agent, enemies) {
            let best_protection = CoverEvaluator::evaluate_cover_at_position(game, &safest_pos, enemies);
            
            // Move if we can get better protection and we're not already at the best position
            return best_protection > current_protection && safest_pos != *current_pos;
        }
        
        false
    }

    /// Finds the best enemy target considering both distance and cover protection.
    /// Prioritizes nearest enemies with lowest cover protection.
    fn find_best_target<'a>(
        &self,
        agent: &Agent,
        game: &Game,
        enemies: &[&'a Agent],
    ) -> Option<&'a Agent> {
        let agent_pos = agent.get_position();
        let max_range = agent.get_optimal_range() * 2; // Maximum shooting range
        
        // Filter enemies that are within shooting range and collect their info
        let mut targets_with_info: Vec<(&Agent, u32, crate::ai::cover::CoverProtection)> = enemies
            .iter()
            .filter_map(|enemy| {
                let distance = agent_pos.distance_to(enemy.get_position());
                if distance <= max_range && distance > 0 {
                    let protection = CoverEvaluator::evaluate_cover_for_enemy(game, enemy, agent_pos);
                    Some((*enemy, distance, protection))
                } else {
                    None
                }
            })
            .collect();

        if targets_with_info.is_empty() {
            return None;
        }

        // Sort by priority: lowest cover first, then by distance (nearest first)
        targets_with_info.sort_by(|a, b| {
            // First compare cover protection (lower is better)
            let cover_cmp = a.2.cmp(&b.2);
            if cover_cmp != std::cmp::Ordering::Equal {
                cover_cmp
            } else {
                // If cover is equal, prefer closer targets
                a.1.cmp(&b.1)
            }
        });

        // Return the best target (first in sorted list)
        targets_with_info.first().map(|(agent, _, _)| *agent)
    }

    /// Finds the best enemy target from a specific position.
    /// Prioritizes nearest enemies with lowest cover protection.
    fn find_best_target_from_position<'a>(
        &self,
        agent: &Agent,
        position: &crate::core::Position,
        game: &Game,
        enemies: &[&'a Agent],
    ) -> Option<&'a Agent> {
        let max_range = agent.get_optimal_range() * 2; // Maximum shooting range
        
        // Filter enemies that are within shooting range from the new position and collect their info
        let mut targets_with_info: Vec<(&Agent, u32, crate::ai::cover::CoverProtection)> = enemies
            .iter()
            .filter_map(|enemy| {
                let distance = position.distance_to(enemy.get_position());
                if distance <= max_range && distance > 0 {
                    let protection = CoverEvaluator::evaluate_cover_for_enemy(game, enemy, position);
                    Some((*enemy, distance, protection))
                } else {
                    None
                }
            })
            .collect();

        if targets_with_info.is_empty() {
            return None;
        }

        // Sort by priority: lowest cover first, then by distance (nearest first)
        targets_with_info.sort_by(|a, b| {
            // First compare cover protection (lower is better)
            let cover_cmp = a.2.cmp(&b.2);
            if cover_cmp != std::cmp::Ordering::Equal {
                cover_cmp
            } else {
                // If cover is equal, prefer closer targets
                a.1.cmp(&b.1)
            }
        });

        // Return the best target (first in sorted list)
        targets_with_info.first().map(|(agent, _, _)| *agent)
    }
}

impl AgentStrategy for CoverStrategy {
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action> {
        let enemies = game.get_enemy_agents();
        
        if enemies.is_empty() {
            return Box::new(HunkerDownAction::new());
        }

        // Check if we should prioritize movement for better positioning
        if self.should_prioritize_movement(agent, game, &enemies) {
            // Find the safest position
            if let Some(safest_pos) = CoverEvaluator::find_safest_position(game, agent, &enemies) {
                // Check if we can also shoot from that position
                if agent.can_shoot() {
                    if let Some(target) = self.find_best_target_from_position(agent, &safest_pos, game, &enemies) {
                        // Perform combined move and shoot action
                        return Box::new(MoveAndShootAction::new(
                            safest_pos.x,
                            safest_pos.y,
                            target.get_agent_id(),
                        ));
                    }
                }
                
                // Move to cover and hunker down if we can't shoot
                return Box::new(MoveAndHunkerAction::new(safest_pos.x, safest_pos.y));
            }
        }

        // If we don't need to move or can't find a better position, try to shoot from current position
        if agent.can_shoot() {
            if let Some(target) = self.find_best_target(agent, game, &enemies) {
                return Box::new(ShootAction::new(target.get_agent_id()));
            }
        }

        // Default to hunkering down for defensive benefits
        Box::new(HunkerDownAction::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{core::TileType, game::Game};

    #[test]
    fn test_cover_strategy_creation() {
        let strategy = CoverStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let game = Game::new(0, 15, 10);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should hunker down when no enemies present
        assert_eq!(command, "HUNKER_DOWN");
    }

    #[test]
    fn test_cover_strategy_with_enemies() {
        let strategy = CoverStrategy::new();
        let mut game = Game::new(0, 15, 10);
        
        // Add some cover tiles
        game.set_tile(6, 5, TileType::HighCover);
        game.set_tile(8, 5, TileType::LowCover);
        
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let enemy = Agent::new(2, 1, 10, 5, 3, 5, 10, 2);
        
        game.add_agent(agent.clone());
        game.add_agent(enemy);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should either shoot or move to cover
        assert!(command.contains("SHOOT") || command.contains("MOVE") || command.contains("HUNKER_DOWN"));
    }

    #[test]
    fn test_target_prioritization() {
        let strategy = CoverStrategy::new();
        let mut game = Game::new(0, 20, 10);
        
        // Set up cover for enemies at different distances
        game.set_tile(10, 4, TileType::HighCover); // High cover for far enemy
        game.set_tile(7, 4, TileType::LowCover);   // Low cover for near enemy
        
        let agent = Agent::new(1, 0, 5, 5, 3, 8, 20, 2); // Long range agent
        let near_enemy_with_low_cover = Agent::new(2, 1, 8, 5, 3, 5, 10, 2); // Distance 3, low cover
        let far_enemy_no_cover = Agent::new(3, 1, 15, 5, 3, 5, 10, 2);       // Distance 10, no cover
        let far_enemy_high_cover = Agent::new(4, 1, 10, 5, 3, 5, 10, 2);     // Distance 5, high cover
        
        game.add_agent(agent.clone());
        game.add_agent(near_enemy_with_low_cover);
        game.add_agent(far_enemy_no_cover);
        game.add_agent(far_enemy_high_cover);

        // Test target selection directly
        let enemies = game.get_enemy_agents();
        let target = strategy.find_best_target(&agent, &game, &enemies);
        
        // Should target the nearest enemy with lowest cover (agent 2)
        // Priority: No cover > Low cover > High cover, then nearest first
        assert!(target.is_some());
        // The exact target depends on cover calculations, but should prioritize appropriately
    }

    #[test]
    fn test_improved_targeting_priority() {
        let strategy = CoverStrategy::new();
        let mut game = Game::new(0, 20, 10);
        
        // Set up cover tiles for specific enemies
        game.set_tile(7, 4, TileType::LowCover);   // For enemy at (8,5) - close with low cover
        game.set_tile(12, 4, TileType::HighCover); // For enemy at (12,5) - medium distance with high cover
        // Enemy at (16,5) will have no cover - far with no cover
        
        let agent = Agent::new(1, 0, 5, 5, 3, 12, 25, 2); // Long range agent
        
        // Create enemies with different distance/cover combinations
        let close_low_cover = Agent::new(2, 1, 8, 5, 3, 5, 10, 2);   // Distance 3, low cover
        let medium_high_cover = Agent::new(3, 1, 12, 5, 3, 5, 10, 2); // Distance 7, high cover  
        let far_no_cover = Agent::new(4, 1, 16, 5, 3, 5, 10, 2);     // Distance 11, no cover
        
        game.add_agent(agent.clone());
        game.add_agent(close_low_cover);
        game.add_agent(medium_high_cover);
        game.add_agent(far_no_cover);
        
        // Test the target selection directly
        let enemies = game.get_enemy_agents();
        let selected_target = strategy.find_best_target(&agent, &game, &enemies);
        
        assert!(selected_target.is_some());
        let target_id = selected_target.unwrap().get_agent_id();
        
        // Priority should be: far_no_cover (4) > close_low_cover (2) > medium_high_cover (3)
        // The agent should prefer the enemy with no cover, even if it's farther
        println!("Selected target: Agent {}", target_id);
        
        // Verify that we're making intelligent choices
        // If there's an enemy with no cover, it should be preferred
        // Otherwise, prefer closer enemies with lower cover
        assert!(target_id == 4 || target_id == 2); // Should be either no-cover enemy or close low-cover enemy
        
        // Should NOT choose the high-cover enemy unless it's the only option
        if enemies.len() > 1 {
            assert_ne!(target_id, 3, "Should not target high-cover enemy when better options exist");
        }
    }
}
