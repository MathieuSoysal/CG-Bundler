//! Shared types, enums, and constants used across the game.

/// Tile types representing different cover levels on the game grid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    /// Empty tile providing no cover.
    Empty,
    /// Low cover tile providing minimal protection.
    LowCover,
    /// High cover tile providing maximum protection.
    HighCover,
}

/// Game constants and configuration values.
pub mod constants {
    /// Default grid width for wooden league.
    pub const DEFAULT_GRID_WIDTH: u32 = 15;

    /// Default grid height for wooden league.
    pub const DEFAULT_GRID_HEIGHT: u32 = 7;

    /// Target positions for wooden league objective.
    pub const WOODEN_TARGETS: [(u32, u32); 2] = [(6, 1), (6, 3)];

    /// Maximum wetness level before agent is eliminated.
    pub const MAX_WETNESS: u32 = 100;

    /// Default agent shooting range.
    pub const DEFAULT_SHOOT_RANGE: u32 = 3;
}

/// Result types for game operations.
pub type GameResult<T> = Result<T, GameError>;

/// Errors that can occur during game operations.
#[derive(Debug, Clone, PartialEq)]
pub enum GameError {
    /// Invalid position on the grid.
    InvalidPosition { x: u32, y: u32 },
    /// Agent not found with the given ID.
    AgentNotFound { agent_id: u32 },
    /// Action cannot be performed due to cooldown.
    OnCooldown { remaining: u32 },
    /// Insufficient resources for the action.
    InsufficientResources { required: u32, available: u32 },
    /// Parse error from input.
    ParseError { message: String },
}

impl std::fmt::Display for GameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GameError::InvalidPosition { x, y } => write!(f, "Invalid position ({}, {})", x, y),
            GameError::AgentNotFound { agent_id } => write!(f, "Agent {} not found", agent_id),
            GameError::OnCooldown { remaining } => {
                write!(f, "Action on cooldown ({} turns remaining)", remaining)
            }
            GameError::InsufficientResources {
                required,
                available,
            } => {
                write!(
                    f,
                    "Insufficient resources (need {}, have {})",
                    required, available
                )
            }
            GameError::ParseError { message } => write!(f, "Parse error: {}", message),
        }
    }
}

impl std::error::Error for GameError {}
