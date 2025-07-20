//! Game rules and mechanics implementation.

use crate::core::{Agent, Position, constants::*};

/// Game rules and mechanics validator.
pub struct GameRules;

impl GameRules {
    /// Calculates damage from a water gun shot based on distance and agent properties.
    pub fn calculate_shot_damage(shooter: &Agent, _target: &Agent, distance: u32) -> u32 {
        let base_damage = shooter.get_soaking_power();

        // Damage decreases with distance beyond optimal range
        if distance <= shooter.get_optimal_range() {
            base_damage
        } else {
            let falloff = distance - shooter.get_optimal_range();
            base_damage.saturating_sub(falloff * 2)
        }
    }

    /// Calculates splash bomb damage in an area.
    pub fn calculate_splash_damage(center: &Position, target: &Position, base_damage: u32) -> u32 {
        let distance = center.distance_to(target);

        match distance {
            0 => base_damage,         // Direct hit
            1 => base_damage * 3 / 4, // Adjacent
            2 => base_damage / 2,     // Close
            _ => 0,                   // No damage
        }
    }

    /// Checks if an agent is eliminated (wetness >= max).
    pub fn is_agent_eliminated(agent: &Agent) -> bool {
        agent.get_wetness() >= MAX_WETNESS
    }

    /// Calculates movement cost based on terrain and agent state.
    pub fn calculate_movement_cost(_agent: &Agent, _terrain_difficulty: u32) -> u32 {
        // Basic movement always costs 1 turn
        // Could be extended for different terrain types
        1
    }

    /// Validates if a shot is possible between two positions.
    pub fn can_shoot_at(shooter_pos: &Position, target_pos: &Position, max_range: u32) -> bool {
        let distance = shooter_pos.distance_to(target_pos);
        distance <= max_range && distance > 0
    }

    /// Determines cover bonus from a tile.
    pub fn get_cover_bonus(tile_type: &crate::core::TileType) -> u32 {
        match tile_type {
            crate::core::TileType::Empty => 0,
            crate::core::TileType::LowCover => 1,
            crate::core::TileType::HighCover => 2,
        }
    }
}

/// Wooden league specific rules and objectives.
pub struct WoodenLeagueRules;

impl WoodenLeagueRules {
    /// Checks if the wooden league objective is completed.
    pub fn is_objective_complete(agents: &[Agent], my_id: u32) -> bool {
        let my_agents: Vec<_> = agents
            .iter()
            .filter(|agent| agent.is_my_agent(my_id))
            .collect();

        if my_agents.len() < 2 {
            return false;
        }

        let target_positions = [Position::new(6, 1), Position::new(6, 3)];
        let agent_positions: Vec<Position> = my_agents
            .iter()
            .map(|agent| *agent.get_position())
            .collect();

        // Check if agents are at target positions
        target_positions
            .iter()
            .all(|target| agent_positions.iter().any(|pos| pos == target))
    }

    /// Gets the next target position for an agent.
    pub fn get_next_target(agent: &Agent, my_id: u32) -> Position {
        let targets = WOODEN_TARGETS;

        // Simple assignment: first agent goes to first target, second to second
        let target_index = (agent.get_agent_id() - my_id) % 2;
        let (x, y) = targets[target_index as usize];
        Position::new(x, y)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{Agent, Position};

    #[test]
    fn test_shot_damage_calculation() {
        let shooter = Agent::new(1, 0, 0, 0, 0, 3, 5, 10);
        let target = Agent::new(2, 1, 5, 5, 0, 3, 5, 10);

        // Within optimal range
        let damage = GameRules::calculate_shot_damage(&shooter, &target, 2);
        assert_eq!(damage, 5);

        // Beyond optimal range
        let damage = GameRules::calculate_shot_damage(&shooter, &target, 5);
        assert_eq!(damage, 1); // 5 - (5-3)*2 = 1
    }

    #[test]
    fn test_splash_damage() {
        let center = Position::new(5, 5);
        let target = Position::new(5, 6); // Distance 1

        let damage = GameRules::calculate_splash_damage(&center, &target, 8);
        assert_eq!(damage, 6); // 8 * 3/4 = 6
    }

    #[test]
    fn test_wooden_league_objective() {
        let agents = vec![
            Agent::new(1, 0, 6, 1, 0, 3, 5, 10), // At target (6,1)
            Agent::new(2, 0, 6, 3, 0, 3, 5, 10), // At target (6,3)
            Agent::new(3, 1, 0, 0, 0, 3, 5, 10), // Enemy agent
        ];

        assert!(WoodenLeagueRules::is_objective_complete(&agents, 0));
    }
}
