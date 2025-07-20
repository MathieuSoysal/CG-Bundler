//! Cover evaluation and tactical positioning utilities.

use crate::{
    core::{Agent, Position, TileType},
    game::Game,
};

/// Represents the cover protection value for a position.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Eq, Ord)]
pub enum CoverProtection {
    None = 0,
    Low = 50,  // 50% damage reduction
    High = 75, // 75% damage reduction
}

impl From<TileType> for CoverProtection {
    fn from(tile_type: TileType) -> Self {
        match tile_type {
            TileType::Empty => CoverProtection::None,
            TileType::LowCover => CoverProtection::Low,
            TileType::HighCover => CoverProtection::High,
        }
    }
}

/// Evaluates cover effectiveness and positioning tactics.
pub struct CoverEvaluator;

impl CoverEvaluator {
    /// Calculates the effective cover protection for an agent at a position against enemies.
    pub fn evaluate_cover_at_position(
        game: &Game,
        position: &Position,
        enemies: &[&Agent],
    ) -> CoverProtection {
        let mut best_protection = CoverProtection::None;

        // Check all adjacent cover tiles
        let adjacent_positions = Self::get_adjacent_positions(game, position);
        
        for cover_pos in adjacent_positions {
            if let Some(tile) = game.grid.get_tile(cover_pos.x, cover_pos.y) {
                if tile.provides_cover() {
                    let cover_protection = CoverProtection::from(tile.get_tile_type());
                    
                    // Check if this cover is effective against any enemy
                    for enemy in enemies {
                        if Self::is_cover_effective(position, &cover_pos, enemy.get_position()) {
                            if cover_protection > best_protection {
                                best_protection = cover_protection;
                            }
                        }
                    }
                }
            }
        }

        best_protection
    }

    /// Finds the safest position within movement range for an agent.
    pub fn find_safest_position(
        game: &Game,
        agent: &Agent,
        enemies: &[&Agent],
    ) -> Option<Position> {
        let current_pos = agent.get_position();

        // Check current position first
        let current_protection = Self::evaluate_cover_at_position(game, current_pos, enemies);
        let mut best_position = *current_pos;
        let mut best_protection = current_protection;

        // Check all adjacent positions (agents can move 1 tile per turn)
        let move_options = Self::get_adjacent_positions(game, current_pos);
        
        for position in move_options {
            // Skip if position is occupied by cover (impassable)
            if let Some(tile) = game.grid.get_tile(position.x, position.y) {
                if tile.provides_cover() {
                    continue;
                }
            }

            let protection = Self::evaluate_cover_at_position(game, &position, enemies);
            
            // Prefer higher protection, or closer positions if protection is equal
            if protection > best_protection || 
               (protection == best_protection && 
                current_pos.distance_to(&position) < current_pos.distance_to(&best_position)) {
                best_position = position;
                best_protection = protection;
            }
        }

        Some(best_position)
    }

    /// Finds the enemy with the least cover protection.
    pub fn find_least_protected_enemy<'a>(
        game: &Game,
        enemies: &[&'a Agent],
        shooter_position: &Position,
    ) -> Option<&'a Agent> {
        let mut least_protected = None;
        let mut lowest_protection = CoverProtection::High;

        for enemy in enemies {
            let protection = Self::evaluate_cover_for_enemy(game, enemy, shooter_position);
            
            if protection < lowest_protection {
                least_protected = Some(*enemy);
                lowest_protection = protection;
            }
        }

        least_protected
    }

    /// Evaluates cover protection for a specific enemy against a shooter position.
    pub fn evaluate_cover_for_enemy(
        game: &Game,
        enemy: &Agent,
        shooter_position: &Position,
    ) -> CoverProtection {
        let enemy_pos = enemy.get_position();
        let mut best_protection = CoverProtection::None;

        // Check all adjacent cover tiles for the enemy
        let adjacent_positions = Self::get_adjacent_positions(game, enemy_pos);
        
        for cover_pos in adjacent_positions {
            if let Some(tile) = game.grid.get_tile(cover_pos.x, cover_pos.y) {
                if tile.provides_cover() {
                    let cover_protection = CoverProtection::from(tile.get_tile_type());
                    
                    // Check if this cover is effective against the shooter
                    if Self::is_cover_effective(enemy_pos, &cover_pos, shooter_position) {
                        if cover_protection > best_protection {
                            best_protection = cover_protection;
                        }
                    }
                }
            }
        }

        best_protection
    }

    /// Checks if cover is effective between an agent, cover tile, and enemy position.
    /// Cover is effective if:
    /// 1. Agent is orthogonally adjacent to cover
    /// 2. Enemy shot comes from opposite side of cover
    /// 3. Both agents are not adjacent to the same cover
    fn is_cover_effective(
        agent_pos: &Position,
        cover_pos: &Position,
        enemy_pos: &Position,
    ) -> bool {
        // Check if agent is orthogonally adjacent to cover
        if agent_pos.distance_to(cover_pos) != 1 {
            return false;
        }

        // Check if both agents are adjacent to the same cover (invalidates cover)
        if enemy_pos.distance_to(cover_pos) == 1 {
            return false;
        }

        // Check if the cover is between agent and enemy
        Self::is_cover_between_positions(agent_pos, cover_pos, enemy_pos)
    }

    /// Determines if a cover tile is positioned between an agent and an enemy.
    fn is_cover_between_positions(
        agent_pos: &Position,
        cover_pos: &Position,
        enemy_pos: &Position,
    ) -> bool {
        // Calculate direction vectors
        let agent_to_cover_x = cover_pos.x as i32 - agent_pos.x as i32;
        let agent_to_cover_y = cover_pos.y as i32 - agent_pos.y as i32;
        
        let cover_to_enemy_x = enemy_pos.x as i32 - cover_pos.x as i32;
        let cover_to_enemy_y = enemy_pos.y as i32 - cover_pos.y as i32;

        // Check if vectors are in roughly the same direction (cover blocks line of sight)
        let same_direction_x = agent_to_cover_x.signum() == cover_to_enemy_x.signum();
        let same_direction_y = agent_to_cover_y.signum() == cover_to_enemy_y.signum();

        // Cover is effective if it's roughly between agent and enemy
        same_direction_x || same_direction_y
    }

    /// Gets all valid adjacent positions for a given position.
    fn get_adjacent_positions(game: &Game, position: &Position) -> Vec<Position> {
        let directions = [(0, 1), (1, 0), (0, -1), (-1, 0)];
        let mut adjacent = Vec::new();

        for (dx, dy) in directions.iter() {
            let new_x = position.x as i32 + dx;
            let new_y = position.y as i32 + dy;

            if new_x >= 0 && new_y >= 0 {
                let new_x = new_x as u32;
                let new_y = new_y as u32;
                
                if game.grid.is_valid_position(new_x, new_y) {
                    adjacent.push(Position::new(new_x, new_y));
                }
            }
        }

        adjacent
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::game::Game;

    #[test]
    fn test_cover_protection_from_tile_type() {
        assert_eq!(CoverProtection::from(TileType::Empty), CoverProtection::None);
        assert_eq!(CoverProtection::from(TileType::LowCover), CoverProtection::Low);
        assert_eq!(CoverProtection::from(TileType::HighCover), CoverProtection::High);
    }

    #[test]
    fn test_cover_effectiveness() {
        let agent_pos = Position::new(1, 1);
        let cover_pos = Position::new(2, 1);
        let enemy_pos = Position::new(4, 1);

        assert!(CoverEvaluator::is_cover_effective(&agent_pos, &cover_pos, &enemy_pos));
    }

    #[test]
    fn test_cover_ineffective_when_both_adjacent() {
        let agent_pos = Position::new(1, 1);
        let cover_pos = Position::new(2, 1);
        let enemy_pos = Position::new(3, 1); // Also adjacent to cover

        assert!(!CoverEvaluator::is_cover_effective(&agent_pos, &cover_pos, &enemy_pos));
    }
}
