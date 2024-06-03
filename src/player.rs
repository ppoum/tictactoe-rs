use std::{
    fmt::Debug,
    io::{self, BufRead, Write},
};

use crate::game::Game;

pub trait Player: Debug {
    // Gets the player's next move. Strategy dependent on player implementation.
    fn get_move(&self, game: &Game) -> (usize, usize);
}

#[derive(Debug, Copy, Clone)]
pub struct LocalPlayer;

impl LocalPlayer {
    /// Reads from stdin until we receive a number between 1 and 3
    fn stdin_read_valid_number(&self, prompt: impl AsRef<str>) -> usize {
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
}

impl Player for LocalPlayer {
    /// Asks the player to enter their next move.
    fn get_move(&self, game: &Game) -> (usize, usize) {
        loop {
            let row = self.stdin_read_valid_number("Select a row") - 1;
            let col = self.stdin_read_valid_number("Select a column") - 1;

            if !game.grid().get_cell(row, col).is_empty() {
                println!("Invalid cell, already in use");
            } else {
                return (row, col);
            }
        }
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;

    #[derive(Debug, Default)]
    pub struct MockPlayer(pub usize, pub usize);

    impl MockPlayer {
        pub fn set_next_move(&mut self, row: usize, col: usize) {
            self.0 = row;
            self.1 = col;
        }
    }

    impl Player for MockPlayer {
        fn get_move(&self, _: &Game) -> (usize, usize) {
            (self.0, self.1)
        }
    }
}
