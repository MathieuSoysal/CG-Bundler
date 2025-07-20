extern crate codingame_summer_challenge_2025;

use codingame_summer_challenge_2025::GameParser;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut game = GameParser::parse_initialization()?;

    loop {
        GameParser::parse_turn_input(&mut game)?;
        
        // Use adaptive strategy for intelligent tactical gameplay
        // This strategy follows the logic:
        // 1. If opponent agents are far → explore for better positioning
        // 2. If opponent agents are near → take cover and shoot
        // 3. If can throw effectively → use splash bombs
        // 4. If agent is being targeted but can't kill enemy → use HUNKER_DOWN
        let commands = game.execute_turn_with_adaptive_strategy();
        
        // Alternative strategies available:
        // let commands = game.execute_turn(); // Simple bomb strategy
        // let commands = game.execute_turn_with_cover_strategy(); // Cover-focused strategy
        // let commands = game.execute_turn_with_splash_bombs(); // Splash bomb strategy
        
        let output = GameParser::format_output(commands);
        print!("{}", output);
    }
}
