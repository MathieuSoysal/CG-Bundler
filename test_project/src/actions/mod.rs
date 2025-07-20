//! Action system implementing the Command pattern.

pub mod action;
pub mod combat;
pub mod combined;
pub mod factory;
pub mod movement;
pub mod utility;
pub mod validation;

pub use action::Action;
pub use combat::{HunkerDownAction, ShootAction, ThrowAction};
pub use combined::{MoveAndShootAction, MoveAndHunkerAction, MoveAndThrowAction};
pub use factory::{ActionFactory, CommandHandler};
pub use movement::MoveAction;
pub use utility::MessageAction;
pub use validation::ActionValidator;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Agent;

    #[test]
    fn test_move_action() {
        let action = MoveAction::new(5, 5);
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        assert_eq!(action.execute(&agent), "MOVE 5 5");
    }
}
