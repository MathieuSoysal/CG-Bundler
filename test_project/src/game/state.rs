//! Core game logic and state management.

use crate::{
    ai::{AgentStrategy, SimpleBombStrategy},
    core::{agent::Agent, TileType},
    game::grid::Grid,
};

/// Main game state container and coordinator.
pub struct Game {
    my_id: u32,
    width: u32,
    height: u32,
    pub grid: Grid,
    pub agents: Vec<Agent>,
    turn: u32,
}

impl Game {
    /// Creates a new game instance with the specified parameters.
    pub fn new(my_id: u32, width: u32, height: u32) -> Self {
        Self {
            my_id,
            width,
            height,
            grid: Grid::new(width, height),
            agents: Vec::new(),
            turn: 0,
        }
    }

    /// Adds an agent to the game.
    pub fn add_agent(&mut self, agent: Agent) {
        self.agents.push(agent);
    }

    /// Updates the list of agents with new state information.
    pub fn update_agents(&mut self, agents: Vec<Agent>) {
        self.agents = agents;
    }

    /// Returns all agents belonging to the current player.
    pub fn get_my_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|agent| agent.is_my_agent(self.my_id))
            .collect()
    }

    /// Returns all enemy agents.
    pub fn get_enemy_agents(&self) -> Vec<&Agent> {
        self.agents
            .iter()
            .filter(|agent| !agent.is_my_agent(self.my_id))
            .collect()
    }

    /// Executes a game turn and returns the commands for all controlled agents.
    /// Now uses simple bomb strategy for maximum damage focused gameplay.
    pub fn execute_turn(&mut self) -> Vec<String> {
        let my_agents = self.get_my_agents();
        let mut commands = Vec::new();

        let strategy = SimpleBombStrategy::new();

        for agent in my_agents.iter() {
            let action = strategy.decide_action(agent, self);
            let command = format!("{};{}", agent.get_agent_id(), action.execute(agent));
            commands.push(command);
        }

        self.turn += 1;
        commands
    }

    /// Executes a game turn using the cover strategy for advanced tactical gameplay.
    /// This method should be used when cover tiles are present on the grid.
    pub fn execute_turn_with_cover_strategy(&mut self) -> Vec<String> {
        let my_agents = self.get_my_agents();
        let mut commands = Vec::new();

        let strategy = crate::ai::CoverStrategy::new();

        for agent in my_agents.iter() {
            let action = strategy.decide_action(agent, self);
            let command = format!("{};{}", agent.get_agent_id(), action.execute(agent));
            commands.push(command);
        }

        self.turn += 1;
        commands
    }

    /// Executes a game turn using the splash bomb strategy for explosive tactics.
    /// This method should be used in leagues where splash bombs are available.
    pub fn execute_turn_with_splash_bombs(&mut self) -> Vec<String> {
        let my_agents = self.get_my_agents();
        let mut commands = Vec::new();

        let strategy = SimpleBombStrategy::new();

        for agent in my_agents.iter() {
            let action = strategy.decide_action(agent, self);
            let command = format!("{};{}", agent.get_agent_id(), action.execute(agent));
            commands.push(command);
        }

        self.turn += 1;
        commands
    }

    /// Executes a game turn using the adaptive strategy for intelligent tactical gameplay.
    /// This method provides advanced decision-making based on enemy proximity and tactical situation.
    pub fn execute_turn_with_adaptive_strategy(&mut self) -> Vec<String> {
        let my_agents = self.get_my_agents();
        let mut commands = Vec::new();

        let strategy = crate::ai::AdaptiveStrategy::new();

        for agent in my_agents.iter() {
            let action = strategy.decide_action(agent, self);
            let command = format!("{};{}", agent.get_agent_id(), action.execute(agent));
            commands.push(command);
        }

        self.turn += 1;
        commands
    }

    /// Updates a tile on the grid.
    pub fn set_tile(&mut self, x: u32, y: u32, tile_type: TileType) {
        self.grid.set_tile(x, y, tile_type);
    }

    /// Returns the grid width.
    pub fn get_width(&self) -> u32 {
        self.width
    }

    /// Returns the grid height.
    pub fn get_height(&self) -> u32 {
        self.height
    }

    /// Returns the current turn number.
    pub fn get_turn(&self) -> u32 {
        self.turn
    }

    /// Returns the player ID.
    pub fn get_my_id(&self) -> u32 {
        self.my_id
    }
}
