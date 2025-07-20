//! Game domain - grid, state management, and game logic.

pub mod grid;
pub mod rules;
pub mod state;

pub use grid::{Grid, Tile};
pub use rules::{GameRules, WoodenLeagueRules};
pub use state::Game;

// Re-export TileType from core for convenience
pub use crate::core::TileType;
