//! Simple bomb strategy focused on maximum damage to biggest groups.

use crate::{
    actions::{Action, MoveAndThrowAction, ThrowAction, HunkerDownAction},
    ai::strategy::AgentStrategy,
    core::{Agent, Position},
    game::Game,
};

/// Simple strategy that focuses exclusively on splash bomb damage to biggest enemy groups.
/// 
/// This strategy is designed for maximum simplicity and damage output:
/// - Finds the biggest group of enemies within range
/// - Moves closer if needed to get within throwing range
/// - Throws splash bombs at the position that hits the most enemies
/// - No cover considerations, no shooting, just pure bombing
pub struct SimpleBombStrategy;

impl SimpleBombStrategy {
    /// Creates a new simple bomb strategy instance.
    pub fn new() -> Self {
        Self
    }

    /// Finds the position that would hit the maximum number of enemies with a splash bomb.
    /// Returns the target position, number of enemies hit, and distance from agent.
    fn find_max_damage_target(
        &self,
        agent: &Agent,
        enemies: &[&Agent],
        game: &Game,
    ) -> Option<(Position, usize, u32)> {
        if agent.get_splash_bombs() == 0 || enemies.is_empty() {
            return None;
        }

        let agent_pos = agent.get_position();
        let my_agents = game.get_my_agents();
        
        let mut best_targets: Vec<(Position, usize, u32)> = Vec::new();

        // Generate all possible target positions
        let mut candidate_positions = Vec::new();
        
        // Add all enemy positions as candidates
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
            candidate_positions.push(enemy_pos);
        }
        
        // Add adjacent positions to enemies to potentially hit multiple groups
        for enemy in enemies {
            let enemy_pos = *enemy.get_position();
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
                if game.grid.is_valid_position(adj_pos.x, adj_pos.y) &&
                   !candidate_positions.contains(adj_pos) {
                    candidate_positions.push(*adj_pos);
                }
            }
        }

        // Evaluate each candidate position
        for target_pos in candidate_positions {
            let distance_to_target = agent_pos.distance_to(&target_pos);
            
            // Count enemies that would be hit
            let mut enemies_hit = 0;
            let mut would_hit_friendly = false;

            // Check target position itself
            if enemies.iter().any(|e| *e.get_position() == target_pos) {
                enemies_hit += 1;
            }

            // Check splash damage positions (8 adjacent tiles)
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
                if !game.grid.is_valid_position(splash_pos.x, splash_pos.y) {
                    continue;
                }

                // Count enemies in splash zone
                if enemies.iter().any(|e| e.get_position() == splash_pos) {
                    enemies_hit += 1;
                }

                // Check for friendly fire
                if my_agents.iter().any(|a| a.get_position() == splash_pos) {
                    would_hit_friendly = true;
                    break;
                }
            }

            // Skip if friendly fire risk
            if would_hit_friendly {
                continue;
            }

            // Only consider targets that hit at least one enemy
            if enemies_hit > 0 {
                best_targets.push((target_pos, enemies_hit, distance_to_target));
            }
        }

        if best_targets.is_empty() {
            return None;
        }

        // Sort by: 1) Most enemies hit (descending), 2) Closest distance (ascending)
        best_targets.sort_by(|a, b| {
            let enemies_cmp = b.1.cmp(&a.1); // More enemies is better
            if enemies_cmp != std::cmp::Ordering::Equal {
                enemies_cmp
            } else {
                a.2.cmp(&b.2) // Closer is better
            }
        });

        best_targets.first().copied()
    }

    /// Finds the best position to move to in order to get within throwing range of the biggest group.
    fn find_best_move_position(
        &self,
        agent: &Agent,
        target_pos: &Position,
        game: &Game,
    ) -> Option<Position> {
        let agent_pos = agent.get_position();
        let max_throw_distance = 4;
        
        // If already in range, no need to move
        if agent_pos.distance_to(target_pos) <= max_throw_distance {
            return None;
        }

        // Find positions that would put us within throwing range
        let mut move_candidates = Vec::new();
        
        // Generate positions in a diamond pattern around the target at max throw distance
        for dx in -(max_throw_distance as i32)..=(max_throw_distance as i32) {
            for dy in -(max_throw_distance as i32)..=(max_throw_distance as i32) {
                let manhattan_dist = dx.abs() + dy.abs();
                if manhattan_dist > max_throw_distance as i32 {
                    continue;
                }
                
                let new_x = (target_pos.x as i32 + dx).max(0) as u32;
                let new_y = (target_pos.y as i32 + dy).max(0) as u32;
                let candidate_pos = Position::new(new_x, new_y);
                
                if game.grid.is_valid_position(candidate_pos.x, candidate_pos.y) {
                    let distance_from_agent = agent_pos.distance_to(&candidate_pos);
                    move_candidates.push((candidate_pos, distance_from_agent));
                }
            }
        }

        if move_candidates.is_empty() {
            return None;
        }

        // Sort by distance from agent (prefer closer moves)
        move_candidates.sort_by_key(|(_, dist)| *dist);
        
        // Return the closest position that gets us in range
        move_candidates.first().map(|(pos, _)| *pos)
    }
}

impl AgentStrategy for SimpleBombStrategy {
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action> {
        let enemies = game.get_enemy_agents();
        
        if enemies.is_empty() {
            return Box::new(HunkerDownAction::new());
        }

        // Find the target that would cause maximum damage
        if let Some((target_pos, _enemies_hit, distance)) = self.find_max_damage_target(agent, &enemies, game) {
            let max_throw_distance = 4;
            
            // If target is within range, throw immediately
            if distance <= max_throw_distance {
                return Box::new(ThrowAction::new(target_pos.x, target_pos.y));
            }
            
            // If target is out of range, move closer and throw
            if let Some(move_pos) = self.find_best_move_position(agent, &target_pos, game) {
                // Check if we can throw from the new position
                let new_distance = move_pos.distance_to(&target_pos);
                if new_distance <= max_throw_distance {
                    return Box::new(MoveAndThrowAction::new(
                        move_pos.x,
                        move_pos.y,
                        target_pos.x,
                        target_pos.y,
                    ));
                }
            }
        }

        // Default: hunker down if no valid targets or moves
        Box::new(HunkerDownAction::new())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_bomb_strategy_creation() {
        let strategy = SimpleBombStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let game = Game::new(0, 15, 10);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should hunker down when no enemies present
        assert_eq!(command, "HUNKER_DOWN");
    }

    #[test]
    fn test_throws_at_biggest_group() {
        let strategy = SimpleBombStrategy::new();
        let agent = Agent::new(1, 0, 6, 6, 3, 5, 10, 3);
        let mut game = Game::new(0, 15, 10);

        // Create a big group of 3 enemies
        let enemy1 = Agent::new(2, 1, 8, 8, 3, 5, 10, 0);
        let enemy2 = Agent::new(3, 1, 8, 9, 3, 5, 10, 0);
        let enemy3 = Agent::new(4, 1, 9, 8, 3, 5, 10, 0);

        game.add_agent(enemy1);
        game.add_agent(enemy2);
        game.add_agent(enemy3);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should throw at the group
        assert!(command.contains("THROW"));
    }

    #[test]
    fn test_moves_closer_if_needed() {
        let strategy = SimpleBombStrategy::new();
        let agent = Agent::new(1, 0, 1, 1, 3, 5, 10, 3); // Far from enemies
        let mut game = Game::new(0, 15, 10);

        // Create enemies far away
        let enemy1 = Agent::new(2, 1, 10, 10, 3, 5, 10, 0);
        let enemy2 = Agent::new(3, 1, 10, 11, 3, 5, 10, 0);

        game.add_agent(enemy1);
        game.add_agent(enemy2);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should move and throw to get closer
        assert!(command.contains("MOVE") && command.contains("THROW"));
    }
}
