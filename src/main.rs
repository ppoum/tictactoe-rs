use tictactoe::{game::Game, player, utils};

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
    // NOTE: Assume 2 local users (until impl user choice)
    let player_x = player::LocalPlayer;
    let player_y = player::LocalPlayer;
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
