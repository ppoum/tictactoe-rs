use std::io::{self, BufRead, Write};

use tictactoe::{game::Game, player};

fn main() {
    loop {
        play_game();

        if !read_bool("Do you want to play again [Y/n]? ") {
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

/// Reads from stdin until we receive a boolean answer
fn read_bool(prompt: impl AsRef<str>) -> bool {
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();
    loop {
        print!("{}", prompt.as_ref());
        io::stdout().flush().unwrap();
        stdin
            .read_line(&mut buffer)
            .expect("Error reading from stdin");

        match buffer.trim().to_lowercase().as_ref() {
            "" | "yes" | "y" | "1" => return true,
            "no" | "n" | "0" => return false,
            _ => {}
        }

        println!("Invalid value");
        buffer = String::new();
    }
}
