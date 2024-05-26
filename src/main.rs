use std::io::{self, BufRead, Write};

use tictactoe::game::Game;

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
    let mut game = Game::default();

    while !game.is_filled() {
        println!("--- {}'s turn ---", game.current_player());
        try_move_until_valid(&mut game);

        println!("{}", game.grid());

        if let Some(p) = game.find_winner() {
            println!("Player {} won the game!", p);
            return;
        }
    }

    println!("Draw!");
}

/// Reads from stdin until we receive a number between 1 and 3
fn read_valid_number(prompt: impl AsRef<str>) -> usize {
    let mut stdin = io::stdin().lock();
    let mut buffer = String::new();
    loop {
        println!("{}", prompt.as_ref());
        print!("Enter a number [1-3]: ");
        io::stdout().flush().unwrap();
        stdin
            .read_line(&mut buffer)
            .expect("Error reading from stdin");

        if let Ok(i) = buffer.trim().parse::<usize>() {
            if (1..=3).contains(&i) {
                return i;
            }
        }

        println!("Invalid value");
        buffer = String::new();
    }
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

/// Asks the player for a move until it receives a valid move
fn try_move_until_valid(game: &mut Game) {
    loop {
        let row = read_valid_number("Select a row") - 1;
        let col = read_valid_number("Select a column") - 1;

        if let Err(e) = game.try_move(row, col) {
            println!("Invalid move: {}", e);
        } else {
            return;
        }
    }
}
