use poker_engine::{game::{GameState, GamePhase}, player::Player, Action};
use std::io::{self, Write};

fn main() {
    println!("=== Poker Engine CLI - Heads-up Test ===\n");
    
    // Create two players for heads-up game
    let players = vec![
        Player::new(0, "Human".to_string(), 1000),
        Player::new(1, "Computer".to_string(), 1000),
    ];
    
    let mut game = GameState::new(players, 10, 20, 0);
    
    loop {
        // Start a new hand
        game.start_new_hand();
        println!("\n--- New Hand ---");
        println!("Your chips: ${}", game.players[0].chips);
        println!("Computer chips: ${}", game.players[1].chips);
        
        // Game loop
        while !matches!(game.current_phase, GamePhase::Showdown) 
            && game.active_player_count() > 1 {
            
            // Show current state
            println!("\n{}", format_game_state(&game));
            
            if game.current_player_index == 0 {
                // Human player's turn
                let valid_actions = game.get_valid_actions();
                let amount_to_call = game.betting_round.amount_to_call(0);
                
                println!("\nYour turn! Valid actions:");
                for (i, action) in valid_actions.iter().enumerate() {
                    match action {
                        Action::Call => println!("  {}: Call ${}", i + 1, amount_to_call),
                        Action::Bet(min_bet) => println!("  {}: Bet (minimum ${}, or type 'bet <amount>')", i + 1, min_bet),
                        Action::Raise(min_raise) => println!("  {}: Raise (minimum ${}, or type 'raise <amount>')", i + 1, min_raise),
                        _ => println!("  {}: {}", i + 1, format_action(action)),
                    }
                }
                
                // Get player input
                print!("Choose action (number, 'bet <amount>', or 'raise <amount>'): ");
                io::stdout().flush().unwrap();
                
                let mut input = String::new();
                io::stdin().read_line(&mut input).unwrap();
                let input = input.trim();
                
                // Check if it's a custom bet command
                if input.starts_with("bet ") {
                    if let Ok(amount) = input[4..].parse::<u64>() {
                        // Check if bet is a valid action
                        if valid_actions.iter().any(|a| matches!(a, Action::Bet(_))) {
                            let action = Action::Bet(amount);
                            match game.process_action(action) {
                                Ok(_) => {},
                                Err(e) => println!("Error: {:?}", e),
                            }
                        } else {
                            println!("Bet is not a valid action right now! Use 'raise' instead.");
                        }
                    } else {
                        println!("Invalid bet amount!");
                    }
                } else if input.starts_with("raise ") {
                    if let Ok(amount) = input[6..].parse::<u64>() {
                        // Check if raise is a valid action
                        if valid_actions.iter().any(|a| matches!(a, Action::Raise(_))) {
                            let action = Action::Raise(amount);
                            match game.process_action(action) {
                                Ok(_) => {},
                                Err(e) => println!("Error: {:?}", e),
                            }
                        } else {
                            println!("Raise is not a valid action right now!");
                        }
                    } else {
                        println!("Invalid raise amount!");
                    }
                } else if let Ok(choice) = input.parse::<usize>() {
                    if choice > 0 && choice <= valid_actions.len() {
                        let action = valid_actions[choice - 1].clone();
                        match game.process_action(action) {
                            Ok(_) => {},
                            Err(e) => println!("Error: {:?}", e),
                        }
                    } else {
                        println!("Invalid choice!");
                    }
                } else {
                    println!("Invalid input! Enter a number, 'bet <amount>', or 'raise <amount>'");
                }
            } else {
                // Simple AI for computer player
                println!("\nComputer is thinking...");
                std::thread::sleep(std::time::Duration::from_secs(1));
                
                let valid_actions = game.get_valid_actions();
                // Simple strategy: always call/check, fold if need to bet more than 100
                let action = if valid_actions.contains(&Action::Check) {
                    Action::Check
                } else if valid_actions.contains(&Action::Call) {
                    let call_amount = game.betting_round.amount_to_call(1);
                    if call_amount > 100 {
                        Action::Fold
                    } else {
                        Action::Call
                    }
                } else {
                    Action::Fold
                };
                
                println!("Computer chooses: {}", format_action(&action));
                game.process_action(action).unwrap();
            }
        }
        
        // Handle hand completion
        if game.is_hand_complete() {
            if matches!(game.current_phase, GamePhase::Showdown) {
                println!("\n=== SHOWDOWN ===");
                show_hands(&game);
            }
            
            match game.complete_hand() {
                Ok(winnings) => {
                    for (player_idx, amount) in winnings {
                        println!("{} wins ${}!", game.players[player_idx].name, amount);
                    }
                },
                Err(e) => {
                    println!("Hand completion error: {:?}", e);
                    break;
                }
            }
            
            // Check if game is over
            if game.is_game_over() {
                if let Some(winner) = game.get_winner() {
                    println!("\n🎉 {} wins the game! 🎉", winner.name);
                } else {
                    println!("\nGame ended.");
                }
                break;
            }
            
            // Ask to continue
            print!("\nPlay another hand? (y/n): ");
            io::stdout().flush().unwrap();
            let mut input = String::new();
            io::stdin().read_line(&mut input).unwrap();
            if !input.trim().eq_ignore_ascii_case("y") {
                break;
            }
            
            continue; // Start next hand
        }
    }
    
    println!("\nThanks for playing!");
}

fn format_game_state(game: &GameState) -> String {
    let mut output = String::new();
    
    output.push_str(&format!("Phase: {:?}\n", game.current_phase));
    output.push_str(&format!("Pot: ${}\n", game.pot_manager.total_pot()));
    output.push_str(&format!("Current bet: ${}\n", game.betting_round.current_bet));
    
    if !game.community_cards.is_empty() {
        output.push_str("Community cards: ");
        for card in &game.community_cards {
            output.push_str(&format!("{} ", card));
        }
        output.push('\n');
    }
    
    if game.current_player_index == 0 {
        if let Some(hole_cards) = &game.players[0].hole_cards {
            output.push_str("Your cards: ");
            for card in hole_cards {
                output.push_str(&format!("{} ", card));
            }
        }
    }
    
    output
}

fn format_action(action: &Action) -> String {
    match action {
        Action::Fold => "Fold".to_string(),
        Action::Check => "Check".to_string(),
        Action::Call => {
            format!("Call")
        },
        Action::Bet(amount) => format!("Bet ${}", amount),
        Action::Raise(amount) => format!("Raise ${}", amount),
        Action::AllIn => "All-in".to_string(),
    }
}

fn show_hands(game: &GameState) {
    for (i, player) in game.players.iter().enumerate() {
        if player.is_active() {
            if let Some(hole_cards) = &player.hole_cards {
                print!("{}'s cards: ", player.name);
                for card in hole_cards {
                    print!("{} ", card);
                }
                println!();
            }
        }
    }
} 