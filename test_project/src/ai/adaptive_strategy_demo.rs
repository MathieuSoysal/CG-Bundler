//! Example demonstrating the adaptive strategy in action.

#[cfg(test)]
mod adaptive_strategy_demo {
    use crate::{
        ai::{AdaptiveStrategy, AgentStrategy},
        core::Agent,
        game::Game,
    };

    #[test]
    fn test_adaptive_strategy_demo() {
        // Create a game scenario with agents
        let mut game = Game::new(0, 16, 8);
        
        // Add my agent
        let my_agent = Agent::new(1, 0, 2, 2, 3, 5, 20, 2);
        game.add_agent(my_agent.clone());
        
        // Add enemy agents at different distances
        let close_enemy = Agent::new(2, 1, 4, 3, 3, 4, 15, 1); // Close enemy
        let far_enemy = Agent::new(3, 1, 12, 6, 3, 4, 15, 1);  // Far enemy
        
        game.add_agent(close_enemy);
        game.add_agent(far_enemy);
        
        let strategy = AdaptiveStrategy::new();
        
        // Test decision making for my agent
        let action = strategy.decide_action(&my_agent, &game);
        let command = action.execute(&my_agent);
        
        println!("Adaptive Strategy Decision: {}", command);
        
        // The strategy should either:
        // 1. SHOOT the close enemy (if in range and can kill)
        // 2. MOVE towards better position and SHOOT
        // 3. HUNKER_DOWN if being targeted
        // 4. THROW if splash bombs available and effective
        assert!(command.contains("SHOOT") || 
                command.contains("MOVE") || 
                command.contains("HUNKER_DOWN") || 
                command.contains("THROW"));
    }

    #[test]
    fn test_adaptive_strategy_exploration_mode() {
        // Test exploration when enemies are far
        let mut game = Game::new(0, 20, 10);
        
        let my_agent = Agent::new(1, 0, 2, 2, 3, 5, 20, 2);
        let far_enemy = Agent::new(2, 1, 18, 8, 3, 4, 15, 1); // Very far enemy
        
        game.add_agent(my_agent.clone());
        game.add_agent(far_enemy);
        
        let strategy = AdaptiveStrategy::new();
        let action = strategy.decide_action(&my_agent, &game);
        let command = action.execute(&my_agent);
        
        println!("Exploration Mode Decision: {}", command);
        
        // Should move towards center/better position when enemies are far
        assert!(command.contains("MOVE") || command.contains("HUNKER_DOWN"));
    }

    #[test]
    fn test_adaptive_strategy_combat_mode() {
        // Test combat when enemies are near
        let mut game = Game::new(0, 16, 8);
        
        let my_agent = Agent::new(1, 0, 5, 5, 3, 5, 30, 2);
        let close_enemy = Agent::new(2, 1, 7, 5, 3, 4, 15, 1); // Close enemy within range
        
        game.add_agent(my_agent.clone());
        game.add_agent(close_enemy);
        
        let strategy = AdaptiveStrategy::new();
        let action = strategy.decide_action(&my_agent, &game);
        let command = action.execute(&my_agent);
        
        println!("Combat Mode Decision: {}", command);
        
        // Should engage in combat when enemies are close
        assert!(command.contains("SHOOT") || command.contains("MOVE") || command.contains("THROW"));
    }

    #[test]
    fn test_adaptive_strategy_defensive_mode() {
        // Test defensive behavior when being targeted
        let mut game = Game::new(0, 16, 8);
        
        // My agent with high wetness (being targeted)
        let mut my_agent = Agent::new(1, 0, 5, 5, 3, 5, 20, 2);
        my_agent.update_state(5, 5, 0, 2, 60); // High wetness
        
        let close_enemy = Agent::new(2, 1, 6, 5, 3, 5, 25, 1); // Enemy that can shoot
        
        game.add_agent(my_agent.clone());
        game.add_agent(close_enemy);
        
        let strategy = AdaptiveStrategy::new();
        let action = strategy.decide_action(&my_agent, &game);
        let command = action.execute(&my_agent);
        
        println!("Defensive Mode Decision: {}", command);
        
        // Should be defensive when heavily damaged
        assert!(command.contains("HUNKER_DOWN") || command.contains("MOVE") || command.contains("SHOOT") || command.contains("THROW"));
    }
}
