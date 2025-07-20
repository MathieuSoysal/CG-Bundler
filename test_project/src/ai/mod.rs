//! AI strategies and decision-making systems.

pub mod adaptive_strategy;
pub mod adaptive_strategy_demo;
pub mod combat;
pub mod cover;
pub mod cover_strategy;
pub mod objective;
pub mod simple_bomb_strategy;
pub mod splash_bomb_strategy;
pub mod strategy;

pub use adaptive_strategy::AdaptiveStrategy;
pub use combat::CombatStrategy;
pub use cover::CoverEvaluator;
pub use cover_strategy::CoverStrategy;
pub use objective::ObjectiveStrategy;
pub use simple_bomb_strategy::SimpleBombStrategy;
pub use splash_bomb_strategy::SplashBombStrategy;
pub use strategy::AgentStrategy;
