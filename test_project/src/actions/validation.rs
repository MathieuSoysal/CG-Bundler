//! Action validation utilities and common validation logic.

use crate::{
    core::{Agent, GameError, GameResult},
    game::Game,
};

/// Validation utilities for actions.
pub struct ActionValidator;

impl ActionValidator {
    /// Validates if an agent can perform any action (not on cooldown).
    pub fn can_act(agent: &Agent) -> GameResult<()> {
        if agent.get_shoot_cooldown() > 0 {
            Err(GameError::OnCooldown {
                remaining: agent.get_shoot_cooldown(),
            })
        } else {
            Ok(())
        }
    }

    /// Validates if a position is within the game grid.
    pub fn is_valid_position(game: &Game, x: u32, y: u32) -> GameResult<()> {
        if game.grid.is_valid_position(x, y) {
            Ok(())
        } else {
            Err(GameError::InvalidPosition { x, y })
        }
    }

    /// Validates if an agent has sufficient splash bombs for throwing.
    pub fn has_splash_bombs(agent: &Agent, required: u32) -> GameResult<()> {
        let available = agent.get_splash_bombs();
        if available >= required {
            Ok(())
        } else {
            Err(GameError::InsufficientResources {
                required,
                available,
            })
        }
    }

    /// Validates if an agent can shoot (not on cooldown and has ammo).
    pub fn can_shoot(agent: &Agent) -> GameResult<()> {
        Self::can_act(agent)?;
        if agent.can_shoot() {
            Ok(())
        } else {
            Err(GameError::OnCooldown {
                remaining: agent.get_shoot_cooldown(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Agent;
    use crate::game::Game;

    #[test]
    fn test_action_validation() {
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        let game = Game::new(0, 15, 7);

        // Test position validation
        assert!(ActionValidator::is_valid_position(&game, 5, 3).is_ok());
        assert!(ActionValidator::is_valid_position(&game, 20, 10).is_err());

        // Test splash bomb validation
        assert!(ActionValidator::has_splash_bombs(&agent, 1).is_ok());
        assert!(ActionValidator::has_splash_bombs(&agent, 15).is_err());
    }
}
