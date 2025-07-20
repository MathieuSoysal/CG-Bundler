//! Advanced adaptive strategy that changes behavior based on enemy proximity and tactical situation.

use crate::{
    actions::{Action, HunkerDownAction, MoveAction, MoveAndHunkerAction, MoveAndShootAction, ShootAction, ThrowAction},
    ai::strategy::AgentStrategy,
    core::{Agent, Position},
    game::Game,
};

/// Adaptive strategy that switches between exploration, combat, and defensive modes.
pub struct AdaptiveStrategy {
    far_threshold: u32,      // Distance considered "far" for exploration mode
    near_threshold: u32,     // Distance considered "near" for combat mode
    bomb_threshold: u32,     // Max distance for effective bomb throws
}

impl AdaptiveStrategy {
    /// Creates a new adaptive strategy with default thresholds.
    pub fn new() -> Self {
        Self {
            far_threshold: 8,      // Agents farther than 8 tiles trigger exploration
            near_threshold: 6,     // Agents within 6 tiles trigger combat mode
            bomb_threshold: 4,     // Splash bombs effective within 4 tiles
        }
    }

    /// Creates a new adaptive strategy with custom thresholds.
    pub fn with_thresholds(far_threshold: u32, near_threshold: u32, bomb_threshold: u32) -> Self {
        Self {
            far_threshold,
            near_threshold,
            bomb_threshold,
        }
    }

    /// Finds the closest enemy agent to the current agent.
    fn find_closest_enemy<'a>(&self, agent: &Agent, game: &'a Game) -> Option<(&'a Agent, u32)> {
        game.get_enemy_agents()
            .iter()
            .map(|enemy| (*enemy, agent.get_distance_to(enemy.get_position())))
            .min_by_key(|(_, distance)| *distance)
    }

    /// Finds the best enemy target (prioritizes low health, close distance).
    fn find_best_target<'a>(&self, agent: &Agent, game: &'a Game) -> Option<&'a Agent> {
        game.get_enemy_agents()
            .iter()
            .filter(|enemy| {
                let distance = agent.get_distance_to(enemy.get_position());
                distance <= agent.get_optimal_range() * 2 // Within shooting range
            })
            .max_by_key(|enemy| {
                // Prioritize high wetness enemies (easier to eliminate)
                let wetness_score = enemy.get_wetness() * 2;
                // Subtract distance to prefer closer targets
                let distance_penalty = agent.get_distance_to(enemy.get_position());
                wetness_score.saturating_sub(distance_penalty)
            })
            .copied()
    }

    /// Finds the best position to move to for exploration.
    fn find_exploration_position(&self, agent: &Agent, game: &Game) -> Option<Position> {
        let agent_pos = agent.get_position();
        let center_x = game.get_width() / 2;
        let center_y = game.get_height() / 2;
        
        // Move towards center of map to control territory
        let target_x = if agent_pos.x < center_x { agent_pos.x + 1 } else if agent_pos.x > center_x { agent_pos.x.saturating_sub(1) } else { agent_pos.x };
        let target_y = if agent_pos.y < center_y { agent_pos.y + 1 } else if agent_pos.y > center_y { agent_pos.y.saturating_sub(1) } else { agent_pos.y };
        
        let target_pos = Position::new(target_x, target_y);
        
        // Ensure the position is valid and not on cover
        if game.grid.is_valid_position(target_pos.x, target_pos.y) {
            if let Some(tile) = game.grid.get_tile(target_pos.x, target_pos.y) {
                if !tile.provides_cover() {
                    return Some(target_pos);
                }
            }
        }
        
        None
    }

    /// Finds the best cover position near the agent.
    fn find_cover_position(&self, agent: &Agent, game: &Game) -> Option<Position> {
        let agent_pos = agent.get_position();
        
        // Look for nearby cover positions
        for distance in 1..=3 {
            for dx in -(distance as i32)..=(distance as i32) {
                for dy in -(distance as i32)..=(distance as i32) {
                    if dx.abs() + dy.abs() != distance as i32 {
                        continue; // Only check positions at exact Manhattan distance
                    }
                    
                    let new_x = agent_pos.x as i32 + dx;
                    let new_y = agent_pos.y as i32 + dy;
                    
                    if new_x >= 0 && new_y >= 0 {
                        let new_pos = Position::new(new_x as u32, new_y as u32);
                        
                        if game.grid.is_valid_position(new_pos.x, new_pos.y) {
                            // Check if this position is adjacent to cover
                            if self.is_adjacent_to_cover(&new_pos, game) {
                                return Some(new_pos);
                            }
                        }
                    }
                }
            }
        }
        
        None
    }

    /// Checks if a position is adjacent to cover tiles.
    fn is_adjacent_to_cover(&self, pos: &Position, game: &Game) -> bool {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        
        for (dx, dy) in directions.iter() {
            let check_x = pos.x as i32 + dx;
            let check_y = pos.y as i32 + dy;
            
            if check_x >= 0 && check_y >= 0 {
                let check_pos = Position::new(check_x as u32, check_y as u32);
                
                if game.grid.is_valid_position(check_pos.x, check_pos.y) {
                    if let Some(tile) = game.grid.get_tile(check_pos.x, check_pos.y) {
                        if tile.provides_cover() {
                            return true;
                        }
                    }
                }
            }
        }
        
        false
    }

    /// Calculates potential damage from a splash bomb at the given position.
    fn calculate_bomb_damage(&self, bomb_pos: &Position, game: &Game) -> u32 {
        let mut total_damage = 0;
        
        // Check center tile and all 8 adjacent tiles
        for dx in -1..=1 {
            for dy in -1..=1 {
                let check_x = bomb_pos.x as i32 + dx;
                let check_y = bomb_pos.y as i32 + dy;
                
                if check_x >= 0 && check_y >= 0 {
                    let check_pos = Position::new(check_x as u32, check_y as u32);
                    
                    // Check if any enemy agents are at this position
                    for enemy in game.get_enemy_agents() {
                        if enemy.get_position() == &check_pos {
                            total_damage += 30; // Splash bomb damage
                        }
                    }
                }
            }
        }
        
        total_damage
    }

    /// Finds the best position to throw a splash bomb.
    fn find_best_bomb_position(&self, agent: &Agent, game: &Game) -> Option<Position> {
        let mut best_pos = None;
        let mut best_damage = 0;
        
        // Check all positions within bomb throw range
        for enemy in game.get_enemy_agents() {
            let enemy_pos = enemy.get_position();
            let distance = agent.get_distance_to(enemy_pos);
            
            if distance <= self.bomb_threshold {
                let potential_damage = self.calculate_bomb_damage(enemy_pos, game);
                
                if potential_damage > best_damage {
                    best_damage = potential_damage;
                    best_pos = Some(*enemy_pos);
                }
                
                // Also check adjacent positions for better splash damage
                let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
                for (dx, dy) in directions.iter() {
                    let bomb_x = enemy_pos.x as i32 + dx;
                    let bomb_y = enemy_pos.y as i32 + dy;
                    
                    if bomb_x >= 0 && bomb_y >= 0 {
                        let bomb_pos = Position::new(bomb_x as u32, bomb_y as u32);
                        let bomb_distance = agent.get_distance_to(&bomb_pos);
                        
                        if bomb_distance <= self.bomb_threshold && game.grid.is_valid_position(bomb_pos.x, bomb_pos.y) {
                            let potential_damage = self.calculate_bomb_damage(&bomb_pos, game);
                            
                            if potential_damage > best_damage {
                                best_damage = potential_damage;
                                best_pos = Some(bomb_pos);
                            }
                        }
                    }
                }
            }
        }
        
        // Only return position if it would do significant damage
        if best_damage >= 30 {
            best_pos
        } else {
            None
        }
    }

    /// Checks if the agent is likely being targeted by enemies.
    fn is_being_targeted(&self, agent: &Agent, game: &Game) -> bool {
        // Consider agent targeted if:
        // 1. There are enemies within their optimal range
        // 2. Agent has high wetness (50+)
        
        if agent.get_wetness() >= 50 {
            return true;
        }
        
        for enemy in game.get_enemy_agents() {
            let distance = agent.get_distance_to(enemy.get_position());
            if distance <= enemy.get_optimal_range() && enemy.can_shoot() {
                return true;
            }
        }
        
        false
    }

    /// Determines if agent can effectively kill a target this turn.
    fn can_kill_target(&self, agent: &Agent, target: &Agent) -> bool {
        if !agent.can_shoot() {
            return false;
        }
        
        let distance = agent.get_distance_to(target.get_position());
        let damage = if distance <= agent.get_optimal_range() {
            agent.get_soaking_power()
        } else if distance <= agent.get_optimal_range() * 2 {
            agent.get_soaking_power() / 2
        } else {
            0
        };
        
        // Can kill if damage would bring target to 100+ wetness
        target.get_wetness() + damage >= 100
    }
}

impl AgentStrategy for AdaptiveStrategy {
    fn decide_action(&self, agent: &Agent, game: &Game) -> Box<dyn Action> {
        // Priority 1: Use splash bombs if effective
        if agent.get_splash_bombs() > 0 {
            if let Some(bomb_pos) = self.find_best_bomb_position(agent, game) {
                return Box::new(ThrowAction::new(bomb_pos.x, bomb_pos.y));
            }
        }

        // Find closest enemy for tactical decision making
        let closest_enemy_info = self.find_closest_enemy(agent, game);
        
        if let Some((_closest_enemy, distance)) = closest_enemy_info {
            // Priority 2: If enemies are far, explore for better positioning
            if distance > self.far_threshold {
                if let Some(explore_pos) = self.find_exploration_position(agent, game) {
                    return Box::new(MoveAction::new(explore_pos.x, explore_pos.y));
                }
            }
            
            // Priority 3: If enemies are near, engage in combat
            if distance <= self.near_threshold {
                // Try to find a good target to shoot
                if agent.can_shoot() {
                    if let Some(target) = self.find_best_target(agent, game) {
                        // Check if we can kill the target
                        if self.can_kill_target(agent, target) {
                            return Box::new(ShootAction::new(target.get_agent_id()));
                        }
                        
                        // If we can't kill but can shoot, try to move to cover and shoot
                        if let Some(cover_pos) = self.find_cover_position(agent, game) {
                            return Box::new(MoveAndShootAction::new(
                                cover_pos.x, 
                                cover_pos.y, 
                                target.get_agent_id()
                            ));
                        }
                        
                        // No good cover, just shoot
                        return Box::new(ShootAction::new(target.get_agent_id()));
                    }
                }
                
                // Can't shoot, try to move to cover
                if let Some(cover_pos) = self.find_cover_position(agent, game) {
                    return Box::new(MoveAndHunkerAction::new(cover_pos.x, cover_pos.y));
                }
            }
        }

        // Priority 4: If being targeted but can't kill enemy, hunker down
        if self.is_being_targeted(agent, game) {
            if let Some(target) = self.find_best_target(agent, game) {
                if !self.can_kill_target(agent, target) {
                    return Box::new(HunkerDownAction::new());
                }
            } else {
                // Being targeted but no valid targets, hunker down
                return Box::new(HunkerDownAction::new());
            }
        }

        // Default: Explore or hunker down
        if let Some(explore_pos) = self.find_exploration_position(agent, game) {
            Box::new(MoveAction::new(explore_pos.x, explore_pos.y))
        } else {
            Box::new(HunkerDownAction::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_adaptive_strategy_creation() {
        let strategy = AdaptiveStrategy::new();
        assert_eq!(strategy.far_threshold, 8);
        assert_eq!(strategy.near_threshold, 6);
        assert_eq!(strategy.bomb_threshold, 4);
    }

    #[test]
    fn test_adaptive_strategy_with_custom_thresholds() {
        let strategy = AdaptiveStrategy::with_thresholds(10, 5, 3);
        assert_eq!(strategy.far_threshold, 10);
        assert_eq!(strategy.near_threshold, 5);
        assert_eq!(strategy.bomb_threshold, 3);
    }

    #[test]
    fn test_adaptive_strategy_hunker_when_no_enemies() {
        let strategy = AdaptiveStrategy::new();
        let agent = Agent::new(1, 0, 5, 5, 3, 5, 10, 2);
        let game = Game::new(0, 12, 8);

        let action = strategy.decide_action(&agent, &game);
        let command = action.execute(&agent);
        
        // Should move towards center or hunker down when no enemies
        assert!(command.contains("MOVE") || command == "HUNKER_DOWN");
    }
}
