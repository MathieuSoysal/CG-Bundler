//! Utility actions for debugging and messaging.

use crate::{actions::action::Action, core::Agent, game::Game};

/// Action to display a message for debugging purposes.
pub struct MessageAction {
    text: String,
}

impl MessageAction {
    /// Creates a new message action with the specified text.
    pub fn new(text: String) -> Self {
        Self { text }
    }
}

impl Action for MessageAction {
    fn execute(&self, _agent: &Agent) -> String {
        format!("MESSAGE {}", self.text)
    }

    fn is_valid(&self, _agent: &Agent, _game: &Game) -> bool {
        true
    }
}
