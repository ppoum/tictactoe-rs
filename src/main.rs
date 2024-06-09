use tictactoe::{
    game::Game,
    player::{self, BotPlayerDifficulty, Player},
    utils,
};

fn main() {
    loop {
        play_game();

        if !utils::read_bool("Do you want to play again?", false) {
            println!("Goodbye!");
            return;
        }
    }
}

/// Game loop: Plays a game until there's a winner or there's a draw
fn play_game() {
    let player_x = prompt_player_selection("Select the player type for X");
    let player_y = prompt_player_selection("Select the player type for O");
    let mut game = Game::new(player_x, player_y);

    while !game.is_filled() {
        println!("--- {}'s turn ---", game.current_player());
        if let Err(e) = game.try_move() {
            panic!("Error while executing move: {}", e);
        }

        println!("{}", game.grid());

        if let Some(p) = game.find_winner() {
            println!("Player {} won the game!", p);
            return;
        }
    }

    println!("Draw!");
}

fn prompt_player_selection(prompt: impl AsRef<str>) -> Box<dyn Player> {
    let player_options = vec![
        "Local Player", // 0
        "Local Bot",    // 1
    ];

    match utils::read_list(prompt, &player_options) {
        0 => {
            // Local Player
            Box::new(player::LocalPlayer)
        }
        1 => {
            // Local Bot
            let diff = prompt_bot_difficulty_selection();
            Box::new(player::BotPlayer::from_difficulty(diff))
        }
        _ => unreachable!(),
    }
}

fn prompt_bot_difficulty_selection() -> BotPlayerDifficulty {
    let diff_options = vec![
        "Easy",       // 0
        "Normal",     // 1
        "Impossible", // 2
    ];

    match utils::read_list("Choose a bot difficulty", &diff_options) {
        0 => BotPlayerDifficulty::Easy,
        1 => BotPlayerDifficulty::Normal,
        2 => todo!(),
        _ => unreachable!(),
    }
}
