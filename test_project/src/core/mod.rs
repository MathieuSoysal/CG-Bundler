//! Core domain models and shared utilities.

pub mod agent;
pub mod position;
pub mod types;

pub use agent::Agent;
pub use position::Position;
pub use types::{GameError, GameResult, TileType, constants};
