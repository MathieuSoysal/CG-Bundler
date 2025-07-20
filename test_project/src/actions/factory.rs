//! Factory pattern for creating actions and command handling.

use crate::{
    actions::{
        action::Action,
        combat::{HunkerDownAction, ShootAction, ThrowAction},
        movement::MoveAction,
        utility::MessageAction,
    },
    core::Agent,
};

/// Factory for creating action instances.
pub struct ActionFactory;

impl ActionFactory {
    /// Creates a move action.
    pub fn create_move(x: u32, y: u32) -> Box<dyn Action> {
        Box::new(MoveAction::new(x, y))
    }

    /// Creates a shoot action.
    pub fn create_shoot(target_id: u32) -> Box<dyn Action> {
        Box::new(ShootAction::new(target_id))
    }

    /// Creates a throw action.
    pub fn create_throw(x: u32, y: u32) -> Box<dyn Action> {
        Box::new(ThrowAction::new(x, y))
    }

    /// Creates a hunker down action.
    pub fn create_hunker_down() -> Box<dyn Action> {
        Box::new(HunkerDownAction::new())
    }

    /// Creates a message action.
    pub fn create_message(text: String) -> Box<dyn Action> {
        Box::new(MessageAction::new(text))
    }
}

/// Manages multiple actions for a single agent.
pub struct CommandHandler {
    agent_id: u32,
    actions: Vec<Box<dyn Action>>,
}

impl CommandHandler {
    /// Creates a new command handler for the specified agent.
    pub fn new(agent_id: u32) -> Self {
        Self {
            agent_id,
            actions: Vec::new(),
        }
    }

    /// Adds an action to the command queue.
    pub fn add_action(&mut self, action: Box<dyn Action>) {
        self.actions.push(action);
    }

    /// Executes all queued actions and returns the formatted command string.
    pub fn execute_all(&self, agent: &Agent) -> String {
        let commands: Vec<String> = self.actions.iter().map(|action| action.execute(agent)).collect();
        format!("{};{}", self.agent_id, commands.join(";"))
    }

    /// Clears all queued actions.
    pub fn clear(&mut self) {
        self.actions.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_action_factory() {
        let move_action = ActionFactory::create_move(5, 5);
        let shoot_action = ActionFactory::create_shoot(1);
        let throw_action = ActionFactory::create_throw(3, 3);
        let hunker_action = ActionFactory::create_hunker_down();
        let message_action = ActionFactory::create_message("Hello".to_string());
        
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        
        assert_eq!(move_action.execute(&agent), "MOVE 5 5");
        assert_eq!(shoot_action.execute(&agent), "SHOOT 1");
        assert_eq!(throw_action.execute(&agent), "THROW 3 3");
        assert_eq!(hunker_action.execute(&agent), "HUNKER_DOWN");
        assert_eq!(message_action.execute(&agent), "MESSAGE Hello");
    }

    #[test]
    fn test_command_handler() {
        let agent = Agent::new(1, 0, 0, 0, 3, 5, 10, 2);
        let mut handler = CommandHandler::new(1);
        
        handler.add_action(ActionFactory::create_move(5, 5));
        
        let result = handler.execute_all(&agent);
        assert!(result.contains("1;MOVE 5 5"));
    }
}
