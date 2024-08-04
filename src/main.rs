use tictactoe::{
    game::{Game, NetworkedGame, RemoteGame, ServerGame},
    player::{self, BotPlayerDifficulty, LocalPlayer, Player},
};

mod utils;

fn main() {
    let game_type = prompt_game_type("What type of game do you wish to play?");

    loop {
        match game_type {
            GameType::Local => play_local_game(),
            GameType::Remote => play_remote_game(),
            GameType::Host => play_hosted_game(),
        }

        if matches!(game_type, GameType::Local) {
            if !utils::read_bool("Do you want to play again?", false) {
                println!("Goodbye!");
                return;
            }
        } else {
            println!("Goodbye!");
            return;
        }
    }
}

enum GameType {
    Local,
    Remote,
    Host,
}

/// Game loop: Plays a game until there's a winner or there's a draw
fn play_local_game() {
    let player_x = prompt_player_selection("Select the player type for X");
    let player_y = prompt_player_selection("Select the player type for O");
    let mut game = Game::new(player_x, player_y);

    while !game.grid().is_full() {
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

/// Connect to remote server + game loop
fn play_remote_game() {
    let addr = utils::read_string_default("Server address", "127.0.0.1:8905");
    let mut game = RemoteGame::connect(addr).expect("Error while connecting to remote server.");
    let player = LocalPlayer;
    networked_game_loop(&mut game, &player)
}

/// Host a game + game loop
fn play_hosted_game() {
    let player = LocalPlayer;

    let addr = utils::read_string_default("Bind on address", "0.0.0.0:8905");
    let game = ServerGame::bind(addr, &Default::default()).expect("Error binding to socket");

    println!("Waiting for a player to connect.");
    let mut game = game.listen().expect("Error listening to connections");
    networked_game_loop(&mut game, &player);
}

fn networked_game_loop(game: &mut impl NetworkedGame, local_player: &dyn Player) {
    while !game.grid().is_full() {
        if game.is_local_turn() {
            println!("--- {}'s turn ---", game.local_mark());
            if let Err(e) = game.try_move(local_player) {
                panic!("Error while executing move: {}", e)
            }
        } else {
            println!("Waiting for remote player to play...");
            if let Err(e) = game.try_move(local_player) {
                panic!("Error while receiving remote move: {}", e)
            }
        }

        println!("{}", game.grid());

        if let Some(p) = game.grid().get_winning_mark() {
            if p == game.local_mark() {
                println!("You won the game!");
            } else {
                println!("Your opponent won the game.");
            }
            return;
        }
    }
    println!("Draw!")
}

fn prompt_game_type(prompt: impl AsRef<str>) -> GameType {
    let options = vec![
        "Local only",               // 0
        "Connect to a remote game", // 1
        "Host a game",              // 2
    ];

    match utils::read_list(prompt, &options) {
        0 => GameType::Local,
        1 => GameType::Remote,
        2 => GameType::Host,
        _ => unreachable!(),
    }
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
        2 => BotPlayerDifficulty::Impossible,
        _ => unreachable!(),
    }
}
