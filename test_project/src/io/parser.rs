//! Input/output parsing for CodinGame communication.

use crate::{
    core::Agent,
    game::{Game, TileType},
};
use std::io;

/// Handles parsing of input data and formatting of output commands.
pub struct GameParser;

impl GameParser {
    /// Parses the initial game setup from standard input.
    pub fn parse_initialization() -> Result<Game, Box<dyn std::error::Error>> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let my_id = input.trim().parse::<u32>()?;

        input.clear();
        io::stdin().read_line(&mut input)?;
        let agent_data_count = input.trim().parse::<u32>()?;

        let mut game_agents = Vec::new();

        for _ in 0..agent_data_count {
            input.clear();
            io::stdin().read_line(&mut input)?;
            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            let agent_id = parts[0].parse::<u32>()?;
            let player = parts[1].parse::<u32>()?;
            let shoot_cooldown = parts[2].parse::<u32>()?;
            let optimal_range = parts[3].parse::<u32>()?;
            let soaking_power = parts[4].parse::<u32>()?;
            let splash_bombs = parts[5].parse::<u32>()?;

            let agent = Agent::new(
                agent_id,
                player,
                0,
                0,
                shoot_cooldown,
                optimal_range,
                soaking_power,
                splash_bombs,
            );
            game_agents.push(agent);
        }

        input.clear();
        io::stdin().read_line(&mut input)?;
        let parts: Vec<&str> = input.trim().split_whitespace().collect();
        let width = parts[0].parse::<u32>()?;
        let height = parts[1].parse::<u32>()?;

        let mut game = Game::new(my_id, width, height);

        for _ in 0..height {
            input.clear();
            io::stdin().read_line(&mut input)?;
            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            for j in 0..width {
                let idx = (j * 3) as usize;
                if idx + 2 < parts.len() {
                    let x = parts[idx].parse::<u32>()?;
                    let y = parts[idx + 1].parse::<u32>()?;
                    let tile_type = TileType::from(parts[idx + 2].parse::<u32>()?);
                    game.set_tile(x, y, tile_type);
                }
            }
        }

        for agent in game_agents {
            game.add_agent(agent);
        }

        Ok(game)
    }

    /// Parses turn input and updates the game state.
    pub fn parse_turn_input(game: &mut Game) -> Result<(), Box<dyn std::error::Error>> {
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let agent_count = input.trim().parse::<u32>()?;

        let mut updated_agents = Vec::new();

        for _ in 0..agent_count {
            input.clear();
            io::stdin().read_line(&mut input)?;
            let parts: Vec<&str> = input.trim().split_whitespace().collect();

            let agent_id = parts[0].parse::<u32>()?;
            let x = parts[1].parse::<u32>()?;
            let y = parts[2].parse::<u32>()?;
            let cooldown = parts[3].parse::<u32>()?;
            let splash_bombs = parts[4].parse::<u32>()?;
            let wetness = parts[5].parse::<u32>()?;

            // Find the agent and update its state
            if let Some(mut agent) = game
                .agents
                .iter()
                .find(|a| a.get_agent_id() == agent_id)
                .cloned()
            {
                agent.update_state(x, y, cooldown, splash_bombs, wetness);
                updated_agents.push(agent);
            }
        }

        game.update_agents(updated_agents);

        input.clear();
        io::stdin().read_line(&mut input)?;
        let _my_agent_count = input.trim().parse::<u32>()?;

        Ok(())
    }

    /// Formats a list of commands into the output format.
    pub fn format_output(commands: Vec<String>) -> String {
        if commands.is_empty() {
            String::new()
        } else {
            format!("{}\n", commands.join("\n"))
        }
    }
}
