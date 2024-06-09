use std::{
    fmt::Debug,
    io::{self, BufRead, Write},
};

use rand::seq::SliceRandom;

use crate::{
    game::Game,
    grid::{Grid, Mark},
};

pub trait Player: Debug {
    // Gets the player's next move. Strategy dependent on player implementation.
    fn get_move(&self, game: &Game, mark: &Mark) -> (usize, usize);
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
    fn get_move(&self, game: &Game, _: &Mark) -> (usize, usize) {
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

#[derive(Debug, Clone, Copy)]
pub enum BotPlayerDifficulty {
    Easy,
    Normal,
    Impossible,
}

#[derive(Debug, Clone, Copy)]
pub struct BotPlayer(BotPlayerDifficulty);

impl BotPlayer {
    pub fn easy() -> Self {
        Self(BotPlayerDifficulty::Easy)
    }

    pub fn normal() -> Self {
        Self(BotPlayerDifficulty::Normal)
    }
    pub fn impossible() -> Self {
        Self(BotPlayerDifficulty::Impossible)
    }

    pub fn from_difficulty(diff: BotPlayerDifficulty) -> Self {
        Self(diff)
    }

    /// Chooses a random free cell in the game's grid.
    fn random_move(game: &Game) -> (usize, usize) {
        // Strategy: randomly choose a free cell
        let mut indexes: Vec<(usize, usize)> = Vec::with_capacity(3 * 3);
        for r in 0..3 {
            for c in 0..3 {
                indexes.push((r, c))
            }
        }
        let indexes: &mut [(usize, usize)] = &mut indexes;
        indexes.shuffle(&mut rand::thread_rng());

        for (row, col) in indexes {
            if game.grid().get_cell(*row, *col).is_empty() {
                return (*row, *col);
            }
        }
        panic!("Grid did not have any empty cells.");
    }

    /// Detects if the player playing with `mark` can win in 1 move. If so, returns the position of
    /// their next winning move.
    fn detect_near_win(grid: &Grid, mark: &Mark) -> Option<(usize, usize)> {
        'row_loop: for (i, row) in grid.rows().enumerate() {
            let mut empty = None;
            for (j, cell) in row.iter().enumerate() {
                match cell.try_get_mark() {
                    None => {
                        if empty.is_none() {
                            empty = Some(j);
                        } else {
                            // 2+ empty cells, ignore this row
                            continue 'row_loop;
                        }
                    }
                    Some(m) if m != mark => {
                        // 1+ cell not `mark`, can't be winning
                        continue 'row_loop;
                    }
                    Some(_) => {}
                }
            }
            if let Some(j) = empty {
                // 1 empty cell + 2 `mark`, near win detected
                return Some((i, j));
            }
        }

        'col_loop: for (j, col) in grid.to_cols().enumerate() {
            let mut empty = None;
            for (i, cell) in col.iter().enumerate() {
                match cell.try_get_mark() {
                    None => {
                        if empty.is_none() {
                            empty = Some(i)
                        } else {
                            continue 'col_loop;
                        }
                    }
                    Some(m) if m != mark => {
                        continue 'col_loop;
                    }
                    Some(_) => {}
                }
            }
            if let Some(i) = empty {
                return Some((i, j));
            }
        }

        // Diagonal (\)
        'diag: {
            let mut empty = None;
            for x in 0..=2 {
                let cell = grid.get_cell(x, x);

                match cell.try_get_mark() {
                    None => {
                        if empty.is_none() {
                            empty = Some(x)
                        } else {
                            break 'diag;
                        }
                    }
                    Some(m) if m != mark => {
                        break 'diag;
                    }
                    Some(_) => {}
                }
            }
            if let Some(x) = empty {
                return Some((x, x));
            }
        }

        // Diagonal (/)
        'diag: {
            let mut empty = None;
            for x in 0..=2 {
                let cell = grid.get_cell(x, 2 - x);

                match cell.try_get_mark() {
                    None => {
                        if empty.is_none() {
                            empty = Some(x)
                        } else {
                            break 'diag;
                        }
                    }
                    Some(m) if m != mark => {
                        break 'diag;
                    }
                    Some(_) => {}
                }
            }
            if let Some(x) = empty {
                return Some((x, 2 - x));
            }
        }

        // No match found yet
        None
    }
}

impl Player for BotPlayer {
    fn get_move(&self, game: &Game, mark: &Mark) -> (usize, usize) {
        // TODO: Implement impossible difficulty
        match self.0 {
            // Strategy: randomly choose a free cell
            BotPlayerDifficulty::Easy => BotPlayer::random_move(game),
            // Strategy: block winning move if found, otherwise revert to random
            BotPlayerDifficulty::Normal => {
                match BotPlayer::detect_near_win(game.grid(), &mark.opposite()) {
                    Some(pos) => pos,
                    None => BotPlayer::random_move(game),
                }
            }
            BotPlayerDifficulty::Impossible => todo!(),
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
        fn get_move(&self, _: &Game, _: &Mark) -> (usize, usize) {
            (self.0, self.1)
        }
    }

    #[test]
    fn detect_near_win_detects_row() {
        // |O|O| |
        // | | | |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::O);
        grid.set_cell(0, 1, Mark::O);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::O);
        assert!(pos.is_some_and(|pos| pos == (0, 2)));
    }

    #[test]
    fn detect_near_win_ignores_fake_row() {
        // |O|O|X|
        // | | | |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::O);
        grid.set_cell(0, 1, Mark::O);
        grid.set_cell(0, 2, Mark::X);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::O);
        assert!(pos.is_none());
    }

    #[test]
    fn detect_near_win_ignores_fake_col() {
        // |O| | |
        // | | | |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::O);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::O);
        assert!(pos.is_none());
    }

    #[test]
    fn detect_near_win_detects_col() {
        // |O| | |
        // | | | |
        // |O| | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::O);
        grid.set_cell(2, 0, Mark::O);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::O);
        assert!(pos.is_some_and(|pos| pos == (1, 0)));
    }

    #[test]
    fn detect_near_win_detects_diagonal() {
        // |X| | |
        // | | | |
        // | | |X|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(2, 2, Mark::X);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::X);
        assert!(pos.is_some_and(|pos| pos == (1, 1)));
    }

    #[test]
    fn detect_near_win_detects_2nd_diagonal() {
        // | | |X|
        // | | | |
        // |X| | |
        let mut grid = Grid::default();
        grid.set_cell(0, 2, Mark::X);
        grid.set_cell(2, 0, Mark::X);

        let pos = BotPlayer::detect_near_win(&grid, &Mark::X);
        assert!(pos.is_some_and(|pos| pos == (1, 1)));
    }
}
