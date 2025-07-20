//! # CodinGame Summer Challenge 2025 - Water Fight Bot
//!
//! This library implements a water fight game bot using Object-Oriented Design patterns
//! following the CodinGame Summer Challenge 2025 specifications.
//!
//! ## Architecture Overview
//!
//! The implementation follows several design patterns:
//! - **Strategy Pattern**: For AI decision making (`AgentStrategy`)
//! - **Command Pattern**: For actions (`Action` trait and implementations)
//! - **Factory Pattern**: For creating actions (`ActionFactory`)
//! - **State Pattern**: For agent state management
//!
//! ## Module Structure
//!
//! - **core**: Domain models (Position, Agent, Types, Constants)
//! - **game**: Game state, grid, rules, and mechanics
//! - **actions**: Command pattern implementations with validation
//! - **ai**: Strategy pattern for decision making
//! - **io**: Input/output parsing for CodinGame
//! - **utils**: Mathematical and collection utilities
//!
//! ## Key Features
//!
//! - **Test-Driven Development**: All components have comprehensive unit tests
//! - **Modular Design**: Clear separation of concerns with well-defined interfaces
//! - **Extensible AI**: Easy to add new strategies and behaviors
//! - **Memory Safe**: Leverages Rust's ownership system for safe concurrent access
//! - **Domain-Driven Design**: Organized by business domains and responsibilities
//!
//! ## Game Objective (Wooden League)
//!
//! In the wooden league, agents must reach target positions (6,1) and (6,3) on the grid.
//! The bot uses a simple movement strategy to navigate agents to these objectives.

// Module declarations - organized by domain
pub mod actions;
pub mod ai;
pub mod core;
pub mod game;
pub mod io;
pub mod utils;

/// This is a public struct for demonstration
/// It has documentation that should be filtered out
pub struct TestStruct {
    /// This field has documentation
    pub field1: i32,
    /// Another documented field
    pub field2: String,
}

/// A documented function that should lose its docs
pub fn documented_function() -> i32 {
    42
}

/// A trait with documentation
pub trait TestTrait {
    /// A documented method
    fn test_method(&self) -> bool;
}

impl TestTrait for TestStruct {
    /// Implementation method with docs
    fn test_method(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic() {
        assert_eq!(documented_function(), 42);
    }

    /// Test documentation that should be filtered
    #[test]
    fn test_with_docs() {
        let s = TestStruct {
            field1: 1,
            field2: "test".to_string(),
        };
        assert!(s.test_method());
    }
}

// Re-export commonly used types for convenience
pub use actions::{Action, ActionFactory, ActionValidator, CommandHandler, MoveAndShootAction, MoveAndHunkerAction};
pub use ai::{AgentStrategy, CombatStrategy, CoverStrategy, CoverEvaluator, ObjectiveStrategy};
pub use crate::core::{Agent, GameError, GameResult, Position, TileType, constants};
pub use game::{Game, GameRules, Grid, Tile, WoodenLeagueRules};
pub use io::GameParser;
