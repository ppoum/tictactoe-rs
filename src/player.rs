use std::{
    fmt::Debug,
    io::{self, BufRead, Write},
};

use rand::seq::SliceRandom;

use crate::grid::{Grid, Mark};

pub trait Player: Debug {
    // Gets the player's next move. Strategy dependent on player implementation.
    fn get_move(&self, grid: &Grid, mark: &Mark) -> (usize, usize);
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
    fn get_move(&self, grid: &Grid, _: &Mark) -> (usize, usize) {
        loop {
            let row = self.stdin_read_valid_number("Select a row") - 1;
            let col = self.stdin_read_valid_number("Select a column") - 1;

            if !grid.get_cell(row, col).is_empty() {
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
    fn random_move(grid: &Grid) -> (usize, usize) {
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
            if grid.get_cell(*row, *col).is_empty() {
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

    /// Plays the optimal move every time
    ///
    /// # Playing first
    /// 1.  Play a corner.
    /// 2.  Opponent doesn't play in the middle cell:
    ///     1. Play the other corner of the unblocked edge.
    ///     2. Win, or play in the corner that sees your 2 other cells.
    ///     3. Play the remaining winning move.
    /// 3.  Opponent plays in the middle cell:
    ///     1. Play the opposite corner from the 1st move.
    ///     2. Try to win or block the opponent's move.
    ///     3. Repeat until draw.
    ///
    /// # Playing second
    /// 1. Opponent starts in a corner.
    ///     1. Play the center cell.
    ///     2. Block the move, or choose an edge cell (NOT a corner)
    ///     3. Try to win, otherwise block.
    /// 2. Opponent starts in the center.
    ///     1. Play a corner.
    ///     2. Try to win, otherwise block.
    /// 3. Opponent starts on an edge
    ///     1. Play the center cell.
    ///     2. If they block opposite to the center (row or col == XOX), play a corner, otherwise
    ///        block.
    ///     3. Try to win, otherwise block.
    fn perfect_move(grid: &Grid, mark: &Mark) -> (usize, usize) {
        match grid.cell_count() {
            0 => {
                // We have the first move
                (0, 0)
            }
            1 => {
                // We have the second move; play center if free, corner otherwise
                if grid.get_cell(1, 1).is_empty() {
                    (1, 1)
                } else {
                    (0, 0)
                }
            }
            2 => {
                // 2nd move (we played first)
                if grid.get_cell(1, 1).is_empty() {
                    // 1. Play the other corner of the unblocked edge
                    if grid.get_cell(0, 1).is_empty() && grid.get_cell(0, 2).is_empty() {
                        (0, 2)
                    } else {
                        (2, 0)
                    }
                } else {
                    // 1. Play the opposite corner from the 1st move.
                    (2, 2)
                }
            }
            3 => {
                // 2nd move (we played 2nd)
                if let Some(block) = Self::detect_near_win(grid, &mark.opposite()) {
                    block
                } else if grid.get_cell(1, 1).try_get_mark() == Some(mark)
                    && ((grid.get_cell(0, 1).try_get_mark() == Some(&mark.opposite())
                        && grid.get_cell(2, 1).try_get_mark() == Some(&mark.opposite()))
                        || (grid.get_cell(1, 0).try_get_mark() == Some(&mark.opposite())
                            && grid.get_cell(1, 2).try_get_mark() == Some(&mark.opposite())))
                {
                    // XOX edgecase: we have center, they have 2 cells opposite of the center; play
                    // a corner
                    (0, 0)
                } else {
                    // Play a non-corner cell
                    if grid.get_cell(0, 1).is_empty() {
                        (0, 1)
                    } else if (grid.get_cell(1, 0)).is_empty() {
                        (1, 0)
                    } else {
                        (1, 2)
                    }
                }
            }
            4 => {
                // 3rd move (we played first)
                if grid.get_cell(1, 1).is_empty() {
                    // 2. Win, or play in the corner that sees your 2 other cells.
                    if let Some(win) = Self::detect_near_win(grid, mark) {
                        win
                    } else {
                        // Figure out which of the free corner sees our 2 other corners
                        // Either the diagonal (2, 2), or if not empty, then only 1 corner should remain
                        if grid.get_cell(2, 2).is_empty() {
                            (2, 2)
                        } else if grid.get_cell(0, 2).is_empty() {
                            (0, 2)
                        } else {
                            (2, 0)
                        }
                    }
                } else {
                    Self::detect_near_win(grid, &mark.opposite()).unwrap()
                }
            }
            x if x > 4 => {
                // Win or block
                if let Some(win) = Self::detect_near_win(grid, mark) {
                    win
                } else if let Some(block) = Self::detect_near_win(grid, &mark.opposite()) {
                    block
                } else {
                    Self::random_move(grid)
                }
            }
            _ => unreachable!(),
        }
    }
}

impl Player for BotPlayer {
    fn get_move(&self, grid: &Grid, mark: &Mark) -> (usize, usize) {
        match self.0 {
            // Strategy: randomly choose a free cell
            BotPlayerDifficulty::Easy => BotPlayer::random_move(grid),
            // Strategy: block winning move if found, otherwise revert to random
            BotPlayerDifficulty::Normal => {
                match BotPlayer::detect_near_win(grid, &mark.opposite()) {
                    Some(pos) => pos,
                    None => BotPlayer::random_move(grid),
                }
            }
            BotPlayerDifficulty::Impossible => BotPlayer::perfect_move(grid, mark),
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
        fn get_move(&self, _: &Grid, _: &Mark) -> (usize, usize) {
            (self.0, self.1)
        }
    }

    fn position_is_corner(pos: (usize, usize)) -> bool {
        let (row, col) = pos;
        (row == 0 || row == 2) && (col == 0 || col == 2)
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

    #[test]
    fn perfect_move_x_correct_first_move() {
        // |!| | |
        // | | | |
        // | | | |
        let grid = Grid::default();

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert!(position_is_corner(pos))
    }

    #[test]
    fn perfect_move_x_correct_second_move_o_middle() {
        // |X| | |
        // | |O| |
        // | | |!|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 2))
    }

    #[test]
    fn perfect_move_x_correct_third_move_o_middle() {
        // |X|!| |
        // | |O| |
        // | |O|X|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 1, Mark::O);
        grid.set_cell(2, 1, Mark::O);
        grid.set_cell(2, 2, Mark::X);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (0, 1))
    }

    #[test]
    fn perfect_move_x_correct_second_move_o_other_1() {
        // |X|O| |
        // | | | |
        // |!| | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 0))
    }

    #[test]
    fn perfect_move_x_correct_second_move_o_other_2() {
        // |X| |!|
        // |O| | |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 0, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (0, 2))
    }

    #[test]
    fn perfect_move_x_correct_second_move_o_other_3() {
        // |X| |O|
        // | | | |
        // |!| | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 2, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 0))
    }

    #[test]
    fn perfect_move_x_correct_third_move_o_other_1() {
        // |X| |O|
        // |O| | |
        // |X| |!|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(2, 0, Mark::X);
        grid.set_cell(0, 2, Mark::O);
        grid.set_cell(1, 0, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 2))
    }

    #[test]
    fn perfect_move_x_correct_third_move_o_other_2() {
        // |X|O|X|
        // |O| | |
        // | | |!|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 2, Mark::X);
        grid.set_cell(0, 1, Mark::O);
        grid.set_cell(1, 0, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 2))
    }

    #[test]
    fn perfect_move_x_correct_third_move_o_other_3() {
        // |X|O|X|
        // | | | |
        // |!| |O|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 2, Mark::X);
        grid.set_cell(0, 1, Mark::O);
        grid.set_cell(2, 2, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (2, 0))
    }

    #[test]
    fn perfect_move_x_correct_last_move_o_other() {
        // |X| |O|
        // |O|!| |
        // |X|O|X|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(2, 0, Mark::X);
        grid.set_cell(2, 2, Mark::X);
        grid.set_cell(0, 2, Mark::O);
        grid.set_cell(1, 0, Mark::O);
        grid.set_cell(2, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::X);
        assert_eq!(pos, (1, 1))
    }

    #[test]
    fn perfect_move_o_correct_first_move_x_corner() {
        // |X| | |
        // | |!| |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert_eq!(pos, (1, 1))
    }

    #[test]
    fn perfect_move_o_correct_second_move_x_corner_1() {
        // |X|!| |
        // |!|O|!|
        // | |!|X|
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(2, 2, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert!(!position_is_corner(pos))
    }

    #[test]
    fn perfect_move_o_correct_second_move_x_corner_2() {
        // |X|!|X|
        // | |O| |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 2, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert_eq!(pos, (0, 1))
    }

    #[test]
    fn perfect_move_o_correct_second_move_x_corner_3() {
        // |X| | |
        // |X|O| |
        // |!| | |
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 0, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert_eq!(pos, (2, 0))
    }

    #[test]
    fn perfect_move_detects_xox_start_row() {
        // | | | |
        // |X|!| |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(1, 0, Mark::X);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert_eq!(pos, (1, 1))
    }

    #[test]
    fn perfect_move_detects_xox_start_col() {
        // | |X| |
        // | |!| |
        // | | | |
        let mut grid = Grid::default();
        grid.set_cell(0, 1, Mark::X);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert_eq!(pos, (1, 1))
    }

    #[test]
    fn perfect_move_detects_xox_row() {
        // |!| |!|
        // |X|O|X|
        // |!| |!|
        let mut grid = Grid::default();
        grid.set_cell(1, 0, Mark::X);
        grid.set_cell(1, 2, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert!(position_is_corner(pos))
    }
    #[test]
    fn perfect_move_detects_xox_col() {
        // |!|X|!|
        // | |O| |
        // |!|X|!|
        let mut grid = Grid::default();
        grid.set_cell(0, 1, Mark::X);
        grid.set_cell(2, 1, Mark::X);
        grid.set_cell(1, 1, Mark::O);

        let pos = BotPlayer::perfect_move(&grid, &Mark::O);
        assert!(position_is_corner(pos))
    }
}
