//! Splash bomb strategy that extends cover strategy with explosive tactics.

use crate::{
    actions::{Action, MoveAndShootAction, MoveAndHunkerAction, MoveAndThrowAction, ShootAction, HunkerDownAction, ThrowAction},
    ai::{cover::CoverEvaluator, strategy::AgentStrategy},
    core::{Agent, Position},
    game::Game,
};

/// Enhanced strategy that combines cover tactics with splash bomb usage.
/// 
/// This strategy extends the cover strategy with splash bomb capabilities:
/// - Prioritizes splash bombs when multiple enemies can be hit effectively
/// - Maintains existing cover evaluation and positioning logic
/// - Falls back to shooting or movement when splash bombs aren't optimal
/// 
/// ## Splash Bomb Target Selection
/// The strategy evaluates splash bomb effectiveness using:
/// 1. **Enemy Clustering**: Prefers targets that hit multiple enemies
/// 2. **Distance**: Must be within 4 tiles (splash bomb range)
/// 3. **Friendly Fire**: Avoids hitting friendly agents
/// 4. **Damage Efficiency**: Prioritizes high-damage scenarios
pub struct SplashBombStrategy;

impl SplashBombStrategy {
    /// Creates a new splash bomb strategy instance.
    pub fn new() -> Self {
        Self
    }

    /// Finds the best splash bomb target that can hit the biggest group of enemies.
    /// Returns the target position and the number of enemies that would be hit.
    /// Prioritizes targets that maximize enemy group elimination.
    fn find_best_splash_target(
        &self,
        agent: &Agent,
        game: &Game,
        enemies: &[&Agent],
    ) -> Option<(Position, usize)> {
        if agent.get_splash_bombs() == 0 {
            return None;
        }

        let agent_pos = agent.get_position();
        let max_throw_distance = 4;
        let my_agents = game.get_my_agents();
        
        let mut best_targets: Vec<(Position, usize, u32)> = Vec::new(); // (position, enemies_hit, total_distance)

        // Check each possible target position within range
        // We'll check both enemy positions and empty spaces that could hit multiple enemies
        let mut candidate_positions = Vec::new();
        
        // Add all enemy positions as candidates
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
            let distance = agent_pos.distance_to(&enemy_pos);
            if distance <= max_throw_distance {
                candidate_positions.push(enemy_pos);
            }
        }
        
        // Add positions adjacent to enemies as candidates (to potentially hit multiple groups)
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
            let base_distance = agent_pos.distance_to(&enemy_pos);
            
            // Only consider adjacent positions if the enemy is close enough
            if base_distance <= max_throw_distance {
                let adjacent_positions = [
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x, enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x + 1, enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y),
                    Position::new(enemy_pos.x + 1, enemy_pos.y),
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y + 1),
                    Position::new(enemy_pos.x, enemy_pos.y + 1),
                    Position::new(enemy_pos.x + 1, enemy_pos.y + 1),
                ];
                
                for adj_pos in &adjacent_positions {
                    let adj_distance = agent_pos.distance_to(adj_pos);
                    if adj_distance <= max_throw_distance && 
                       game.grid.is_valid_position(adj_pos.x, adj_pos.y) &&
                       !candidate_positions.contains(adj_pos) {
                        candidate_positions.push(*adj_pos);
                    }
                }
            }
        }

        // Evaluate each candidate position
        for target_pos in candidate_positions {
            let distance_to_target = agent_pos.distance_to(&target_pos);
            if distance_to_target > max_throw_distance {
                continue;
            }

            // Count how many enemies would be hit by splash damage
            let mut enemies_hit = 0;
            let mut would_hit_friendly = false;
            let mut total_distance_to_enemies = 0u32;

            // Check the target position itself
            if let Some(enemy) = enemies.iter().find(|e| *e.get_position() == target_pos) {
                enemies_hit += 1;
                total_distance_to_enemies += agent_pos.distance_to(enemy.get_position());
            }

            // Check adjacent positions (8 directions)
            let splash_positions = [
                Position::new(target_pos.x.saturating_sub(1), target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x, target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x + 1, target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x.saturating_sub(1), target_pos.y),
                Position::new(target_pos.x + 1, target_pos.y),
                Position::new(target_pos.x.saturating_sub(1), target_pos.y + 1),
                Position::new(target_pos.x, target_pos.y + 1),
                Position::new(target_pos.x + 1, target_pos.y + 1),
            ];

            for splash_pos in &splash_positions {
                // Check if this position is valid (within grid bounds)
                if !game.grid.is_valid_position(splash_pos.x, splash_pos.y) {
                    continue;
                }

                // Check if any enemy is at this splash position
                if let Some(enemy) = enemies.iter().find(|e| e.get_position() == splash_pos) {
                    enemies_hit += 1;
                    total_distance_to_enemies += agent_pos.distance_to(enemy.get_position());
                }

                // Check if any friendly agent would be hit
                if my_agents.iter().any(|a| a.get_position() == splash_pos) {
                    would_hit_friendly = true;
                    break;
                }
            }

            // Skip if we would hit friendly agents
            if would_hit_friendly {
                continue;
            }

            // Only consider targets that hit at least one enemy
            if enemies_hit > 0 {
                best_targets.push((target_pos, enemies_hit, total_distance_to_enemies));
            }
        }

        if best_targets.is_empty() {
            return None;
        }

        // Sort by: 1) Most enemies hit (descending), 2) Shortest total distance (ascending)
        best_targets.sort_by(|a, b| {
            let enemies_cmp = b.1.cmp(&a.1); // More enemies is better
            if enemies_cmp != std::cmp::Ordering::Equal {
                enemies_cmp
            } else {
                a.2.cmp(&b.2) // Closer total distance is better
            }
        });

        // Return the best target (hits the most enemies, closest if tied)
        best_targets.first().map(|(pos, count, _)| (*pos, *count))
    }

    /// Finds the best splash bomb target from a specific position that can hit the biggest group of enemies.
    /// Returns the target position and the number of enemies that would be hit.
    /// Prioritizes targets that maximize enemy group elimination.
    fn find_best_splash_target_from_position(
        &self,
        agent: &Agent,
        position: &Position,
        game: &Game,
        enemies: &[&Agent],
    ) -> Option<(Position, usize)> {
        if agent.get_splash_bombs() == 0 {
            return None;
        }

        let max_throw_distance = 4;
        let my_agents = game.get_my_agents();
        
        let mut best_targets: Vec<(Position, usize, u32)> = Vec::new(); // (position, enemies_hit, total_distance)

        // Check each possible target position within range
        let mut candidate_positions = Vec::new();
        
        // Add all enemy positions as candidates
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
            let distance = position.distance_to(&enemy_pos);
            if distance <= max_throw_distance {
                candidate_positions.push(enemy_pos);
            }
        }
        
        // Add positions adjacent to enemies as candidates (to potentially hit multiple groups)
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
            let base_distance = position.distance_to(&enemy_pos);
            
            // Only consider adjacent positions if the enemy is close enough
            if base_distance <= max_throw_distance {
                let adjacent_positions = [
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x, enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x + 1, enemy_pos.y.saturating_sub(1)),
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y),
                    Position::new(enemy_pos.x + 1, enemy_pos.y),
                    Position::new(enemy_pos.x.saturating_sub(1), enemy_pos.y + 1),
                    Position::new(enemy_pos.x, enemy_pos.y + 1),
                    Position::new(enemy_pos.x + 1, enemy_pos.y + 1),
                ];
                
                for adj_pos in &adjacent_positions {
                    let adj_distance = position.distance_to(adj_pos);
                    if adj_distance <= max_throw_distance && 
                       game.grid.is_valid_position(adj_pos.x, adj_pos.y) &&
                       !candidate_positions.contains(adj_pos) {
                        candidate_positions.push(*adj_pos);
                    }
                }
            }
        }

        // Evaluate each candidate position
        for target_pos in candidate_positions {
            let distance_to_target = position.distance_to(&target_pos);
            if distance_to_target > max_throw_distance {
                continue;
            }

            // Count how many enemies would be hit by splash damage
            let mut enemies_hit = 0;
            let mut would_hit_friendly = false;
            let mut total_distance_to_enemies = 0u32;

            // Check the target position itself
            if let Some(enemy) = enemies.iter().find(|e| *e.get_position() == target_pos) {
                enemies_hit += 1;
                total_distance_to_enemies += position.distance_to(enemy.get_position());
            }

            // Check adjacent positions (8 directions)
            let splash_positions = [
                Position::new(target_pos.x.saturating_sub(1), target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x, target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x + 1, target_pos.y.saturating_sub(1)),
                Position::new(target_pos.x.saturating_sub(1), target_pos.y),
                Position::new(target_pos.x + 1, target_pos.y),
                Position::new(target_pos.x.saturating_sub(1), target_pos.y + 1),
                Position::new(target_pos.x, target_pos.y + 1),
                Position::new(target_pos.x + 1, target_pos.y + 1),
            ];

            for splash_pos in &splash_positions {
                // Check if this position is valid (within grid bounds)
                if !game.grid.is_valid_position(splash_pos.x, splash_pos.y) {
                    continue;
                }

                // Check if any enemy is at this splash position
                if let Some(enemy) = enemies.iter().find(|e| e.get_position() == splash_pos) {
                    enemies_hit += 1;
                    total_distance_to_enemies += position.distance_to(enemy.get_position());
                }

                // Check if any friendly agent would be hit
                if my_agents.iter().any(|a| a.get_position() == splash_pos) {
                    would_hit_friendly = true;
                    break;
                }
            }

            // Skip if we would hit friendly agents
            if would_hit_friendly {
                continue;
            }

            // Only consider targets that hit at least one enemy
            if enemies_hit > 0 {
                best_targets.push((target_pos, enemies_hit, total_distance_to_enemies));
            }
        }

        if best_targets.is_empty() {
            return None;
        }

        // Sort by: 1) Most enemies hit (descending), 2) Shortest total distance (ascending)
        best_targets.sort_by(|a, b| {
            let enemies_cmp = b.1.cmp(&a.1); // More enemies is better
            if enemies_cmp != std::cmp::Ordering::Equal {
                enemies_cmp
            } else {
                a.2.cmp(&b.2) // Closer total distance is better
            }
        });

        // Return the best target (hits the most enemies, closest if tied)
        best_targets.first().map(|(pos, count, _)| (*pos, *count))
    }

    /// Determines if an agent should prioritize moving to cover over attacking.
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

    /// Finds the best enemy target for shooting, considering distance and cover protection.
    fn find_best_shooting_target<'a>(
        &self,
        agent: &Agent,
        game: &Game,
        enemies: &[&'a Agent],
    ) -> Option<&'a Agent> {
        let agent_pos = agent.get_position();
        let max_range = agent.get_optimal_range() * 2;
        
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

        // Sort by priority: lowest cover first, then by distance
        targets_with_info.sort_by(|a, b| {
            let cover_cmp = a.2.cmp(&b.2);
            if cover_cmp != std::cmp::Ordering::Equal {
                cover_cmp
            } else {
                a.1.cmp(&b.1)
            }
        });

        targets_with_info.first().map(|(agent, _, _)| *agent)
    }

    /// Finds the best enemy target for shooting from a specific position.
    fn find_best_shooting_target_from_position<'a>(
        &self,
        agent: &Agent,
        position: &Position,
        game: &Game,
        enemies: &[&'a Agent],
    ) -> Option<&'a Agent> {
        let max_range = agent.get_optimal_range() * 2;
        
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

        targets_with_info.sort_by(|a, b| {
            let cover_cmp = a.2.cmp(&b.2);
            if cover_cmp != std::cmp::Ordering::Equal {
                cover_cmp
            } else {
                a.1.cmp(&b.1)
            }
        });

        targets_with_info.first().map(|(agent, _, _)| *agent)
    }
}

impl AgentStrategy for SplashBombStrategy {
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action> {
        let enemies = game.get_enemy_agents();
        
        if enemies.is_empty() {
            return Box::new(HunkerDownAction::new());
        }

        // First priority: Check for splash bomb opportunities
        if let Some((target_pos, enemies_hit)) = self.find_best_splash_target(agent, game, &enemies) {
            // Use splash bomb if we can hit 2 or more enemies
            if enemies_hit >= 2 {
                return Box::new(ThrowAction::new(target_pos.x, target_pos.y));
            }
        }

        // Second priority: Check if we should prioritize movement for better positioning
        if self.should_prioritize_movement(agent, game, &enemies) {
            if let Some(safest_pos) = CoverEvaluator::find_safest_position(game, agent, &enemies) {
                // Check if we can throw splash bombs from that position
                if let Some((target_pos, enemies_hit)) = self.find_best_splash_target_from_position(agent, &safest_pos, game, &enemies) {
                    if enemies_hit >= 2 {
                        // Perform combined move and throw action
                        return Box::new(MoveAndThrowAction::new(
                            safest_pos.x,
                            safest_pos.y,
                            target_pos.x,
                            target_pos.y,
                        ));
                    }
                }
                
                // Check if we can also shoot from that position
                if agent.can_shoot() {
                    if let Some(target) = self.find_best_shooting_target_from_position(agent, &safest_pos, game, &enemies) {
                        // Perform combined move and shoot action
                        return Box::new(MoveAndShootAction::new(
                            safest_pos.x,
                            safest_pos.y,
                            target.get_agent_id(),
                        ));
                    }
                }
                
                // Move to cover and hunker down if we can't attack
                return Box::new(MoveAndHunkerAction::new(safest_pos.x, safest_pos.y));
            }
        }

        // Third priority: Try to shoot from current position
        if agent.can_shoot() {
            if let Some(target) = self.find_best_shooting_target(agent, game, &enemies) {
                return Box::new(ShootAction::new(target.get_agent_id()));
            }
        }

        // Fourth priority: Use splash bomb even for single targets if no other options
        if let Some((target_pos, _)) = self.find_best_splash_target(agent, game, &enemies) {
            return Box::new(ThrowAction::new(target_pos.x, target_pos.y));
        }

        // Default: Hunker down for defensive benefits
        Box::new(HunkerDownAction::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_splash_bomb_strategy_creation() {
        let strategy = SplashBombStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let game = Game::new(0, 15, 10);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should hunker down when no enemies present
        assert_eq!(command, "HUNKER_DOWN");
    }

    #[test]
    fn test_splash_bomb_target_finding() {
        let strategy = SplashBombStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let mut game = Game::new(0, 15, 10);

        // Add some enemy agents close to each other
        let enemy1 = Agent::new(2, 1, 7, 7, 3, 5, 10, 0);
        let enemy2 = Agent::new(3, 1, 8, 7, 3, 5, 10, 0);
        game.add_agent(enemy1);
        game.add_agent(enemy2);

        let enemies = game.get_enemy_agents();
        let result = strategy.find_best_splash_target(&agent, &game, &enemies);
        
        // Should find a target that can hit multiple enemies
        assert!(result.is_some());
        if let Some((_, enemies_hit)) = result {
            assert!(enemies_hit >= 2);
        }
    }

    #[test]
    fn test_splash_bomb_targets_biggest_group() {
        let strategy = SplashBombStrategy::new();
        let agent = Agent::new(1, 0, 6, 6, 3, 5, 10, 3); // Agent closer to enemies
        let mut game = Game::new(0, 15, 10);

        // Create two groups of enemies:
        // Group 1: 2 enemies close together at (7,7) and (8,7)
        let enemy1 = Agent::new(2, 1, 7, 7, 3, 5, 10, 0);
        let enemy2 = Agent::new(3, 1, 8, 7, 3, 5, 10, 0);
        
        // Group 2: 3 enemies that can all be hit by one splash bomb
        // Center at (9,9), with enemies at (9,9), (8,9), and (9,8) - all within splash range
        let enemy3 = Agent::new(4, 1, 9, 9, 3, 5, 10, 0);  // Center
        let enemy4 = Agent::new(5, 1, 8, 9, 3, 5, 10, 0);  // Adjacent left
        let enemy5 = Agent::new(6, 1, 9, 8, 3, 5, 10, 0);  // Adjacent up

        game.add_agent(enemy1);
        game.add_agent(enemy2);
        game.add_agent(enemy3);
        game.add_agent(enemy4);
        game.add_agent(enemy5);

        let enemies = game.get_enemy_agents();
        let result = strategy.find_best_splash_target(&agent, &game, &enemies);
        
        // Should target the bigger group (3 enemies) over the smaller group (2 enemies)
        assert!(result.is_some());
        if let Some((target_pos, enemies_hit)) = result {
            assert!(enemies_hit >= 3, "Should target the group with 3+ enemies, got {}. Target position: ({}, {})", 
                    enemies_hit, target_pos.x, target_pos.y);
        }
    }

    #[test]
    fn test_splash_bomb_prioritizes_group_size_over_distance() {
        let strategy = SplashBombStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 3);
        let mut game = Game::new(0, 15, 10);

        // Closer group with 2 enemies at (6,6) and (7,6)
        let enemy1 = Agent::new(2, 1, 6, 6, 3, 5, 10, 0);
        let enemy2 = Agent::new(3, 1, 7, 6, 3, 5, 10, 0);
        
        // Farther group with 3 enemies at (7,8), (8,8), (7,9) - within range but farther
        let enemy3 = Agent::new(4, 1, 7, 8, 3, 5, 10, 0);
        let enemy4 = Agent::new(5, 1, 8, 8, 3, 5, 10, 0);
        let enemy5 = Agent::new(6, 1, 7, 9, 3, 5, 10, 0);

        game.add_agent(enemy1);
        game.add_agent(enemy2);
        game.add_agent(enemy3);
        game.add_agent(enemy4);
        game.add_agent(enemy5);

        let enemies = game.get_enemy_agents();
        let result = strategy.find_best_splash_target(&agent, &game, &enemies);
        
        assert!(result.is_some());
        if let Some((_, enemies_hit)) = result {
            // Should prioritize the bigger group even if it's farther
            assert!(enemies_hit >= 3, "Should prioritize group size over distance");
        }
    }

    #[test]
    fn test_splash_bomb_hits_clustered_enemies() {
        let strategy = SplashBombStrategy::new();
        let agent = Agent::new(1, 0, 6, 6, 3, 5, 10, 3); // Agent at (6,6) with 3 splash bombs
        let mut game = Game::new(0, 15, 10);

        // Simple test: 3 enemies in a tight cluster that should all be hit by one bomb
        let enemy1 = Agent::new(2, 1, 8, 8, 3, 5, 10, 0);  // Center
        let enemy2 = Agent::new(3, 1, 8, 9, 3, 5, 10, 0);  // Adjacent down
        let enemy3 = Agent::new(4, 1, 9, 8, 3, 5, 10, 0);  // Adjacent right

        game.add_agent(enemy1);
        game.add_agent(enemy2);
        game.add_agent(enemy3);

        let enemies = game.get_enemy_agents();
        let result = strategy.find_best_splash_target(&agent, &game, &enemies);
        
        assert!(result.is_some());
        if let Some((_, enemies_hit)) = result {
            assert_eq!(enemies_hit, 3, "Should hit all 3 enemies in the cluster");
        }
    }
}
