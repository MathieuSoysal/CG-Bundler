//! Agent entities and their state management.

use crate::core::position::Position;

/// Represents a game agent with all its properties and state.
#[derive(Debug, Clone)]
pub struct Agent {
    agent_id: u32,
    player: u32,
    position: Position,
    shoot_cooldown: u32,
    optimal_range: u32,
    soaking_power: u32,
    splash_bombs: u32,
    cooldown: u32,
    wetness: u32,
}

impl Agent {
    /// Creates a new agent with the specified properties.
    pub fn new(
        agent_id: u32,
        player: u32,
        x: u32,
        y: u32,
        shoot_cooldown: u32,
        optimal_range: u32,
        soaking_power: u32,
        splash_bombs: u32,
    ) -> Self {
        Self {
            agent_id,
            player,
            position: Position::new(x, y),
            shoot_cooldown,
            optimal_range,
            soaking_power,
            splash_bombs,
            cooldown: 0,
            wetness: 0,
        }
    }

    /// Updates the agent's dynamic state from turn input.
    pub fn update_state(&mut self, x: u32, y: u32, cooldown: u32, splash_bombs: u32, wetness: u32) {
        self.position = Position::new(x, y);
        self.cooldown = cooldown;
        self.splash_bombs = splash_bombs;
        self.wetness = wetness;
    }

    /// Returns true if the agent can shoot (cooldown is 0).
    pub fn can_shoot(&self) -> bool {
        self.cooldown == 0
    }

    /// Returns true if this agent belongs to the specified player.
    pub fn is_my_agent(&self, my_id: u32) -> bool {
        self.player == my_id
    }

    /// Calculates the Manhattan distance to a target position.
    pub fn get_distance_to(&self, target: &Position) -> u32 {
        self.position.distance_to(target)
    }

    /// Returns the agent's unique identifier.
    pub fn get_agent_id(&self) -> u32 {
        self.agent_id
    }

    /// Returns a reference to the agent's current position.
    pub fn get_position(&self) -> &Position {
        &self.position
    }

    /// Returns the number of splash bombs available.
    pub fn get_splash_bombs(&self) -> u32 {
        self.splash_bombs
    }

    /// Returns the agent's shoot cooldown period.
    pub fn get_shoot_cooldown(&self) -> u32 {
        self.shoot_cooldown
    }

    /// Returns the agent's optimal shooting range.
    pub fn get_optimal_range(&self) -> u32 {
        self.optimal_range
    }

    /// Returns the agent's soaking power (damage output).
    pub fn get_soaking_power(&self) -> u32 {
        self.soaking_power
    }

    /// Returns the agent's current wetness level.
    pub fn get_wetness(&self) -> u32 {
        self.wetness
    }

    /// Returns the player ID that owns this agent.
    pub fn get_player(&self) -> u32 {
        self.player
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_can_shoot() {
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        assert!(agent.can_shoot());
    }

    #[test]
    fn test_agent_getters() {
        let agent = Agent::new(1, 0, 5, 5, 3, 4, 10, 2);

        assert_eq!(agent.get_agent_id(), 1);
        assert_eq!(agent.get_position(), &Position::new(5, 5));
        assert_eq!(agent.get_shoot_cooldown(), 3);
        assert_eq!(agent.get_optimal_range(), 4);
        assert_eq!(agent.get_soaking_power(), 10);
        assert_eq!(agent.get_splash_bombs(), 2);
    }
}
