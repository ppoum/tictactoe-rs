use thiserror::Error;

use crate::grid::{Grid, Player};

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Cell is not empty")]
    UnemptyCell,
    #[error("Cell is out of bounds")]
    OutOfBounds,
}

#[derive(Debug, Clone, Copy)]
pub struct Game {
    grid: Grid,
    is_x_turn: bool,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            grid: Default::default(),
            is_x_turn: true,
        }
    }
}

impl Game {
    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn current_player(&self) -> Player {
        if self.is_x_turn {
            Player::X
        } else {
            Player::O
        }
    }

    pub fn try_move(&mut self, row: usize, col: usize) -> Result<(), GameError> {
        if !(0..=2).contains(&row) || !(0..=2).contains(&col) {
            return Err(GameError::OutOfBounds);
        }

        if !self.grid().get_cell(row, col).is_empty() {
            return Err(GameError::UnemptyCell);
        }

        let cell_type = if self.is_x_turn { Player::X } else { Player::O };
        self.grid.set_cell(row, col, cell_type);

        self.is_x_turn = !self.is_x_turn;
        Ok(())
    }

    pub fn find_winner(&self) -> Option<Player> {
        // Detect row win
        for row in self.grid.rows() {
            if !row[0].is_empty() && row.iter().all(|&cell| cell == row[0]) {
                return row[0].try_get_player().copied();
            }
        }

        // Detect col win
        for col in self.grid.to_cols() {
            if !col[0].is_empty() && col.iter().all(|&cell| cell == col[0]) {
                return col[0].try_get_player().copied();
            }
        }

        // Detect diagonal (\)
        let first = self.grid.get_cell(0, 0);
        if !first.is_empty()
            && first == self.grid.get_cell(1, 1)
            && first == self.grid.get_cell(2, 2)
        {
            return first.try_get_player().copied();
        }

        // Detect diagonal (/)
        let first = self.grid.get_cell(0, 2);
        if !first.is_empty()
            && first == self.grid.get_cell(1, 1)
            && first == self.grid.get_cell(2, 0)
        {
            return first.try_get_player().copied();
        }

        None
    }

    /// Returns true if the grid is full
    pub fn is_filled(&self) -> bool {
        self.grid.is_full()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    impl Game {
        fn set_grid(&mut self, grid: Grid) {
            self.grid = grid;
        }
    }

    #[test]
    fn try_move_rotates_player() {
        let mut game = Game::default();

        // First move should always be X
        let player = game.current_player();
        assert_eq!(player, Player::X);
        assert!(game.try_move(0, 0).is_ok());
        assert_eq!(game.grid.get_cell(0, 0).try_get_player(), Some(&player));

        let player = game.current_player();
        assert_eq!(player, Player::O);
        assert!(game.try_move(1, 1).is_ok());
        assert_eq!(game.grid.get_cell(1, 1).try_get_player(), Some(&player));
    }

    #[test]
    fn try_move_errors_when_row_out_of_bounds() {
        let mut game = Game::default();
        // Bounds are from 0 to 2 (0-indexed)
        let high_bound = 3;

        assert!(game.try_move(high_bound, high_bound).is_err())
    }

    #[test]
    fn find_winner_finds_horizontal_win() {
        let mut grid = Grid::default();
        grid.set_cell(2, 0, Player::X);
        grid.set_cell(2, 1, Player::X);
        grid.set_cell(2, 2, Player::X);

        let mut game = Game::default();
        game.set_grid(grid);

        assert_eq!(game.find_winner(), Some(Player::X));
    }

    #[test]
    fn find_winner_finds_vertical_win() {
        let mut grid = Grid::default();
        grid.set_cell(1, 0, Player::O);
        grid.set_cell(1, 1, Player::O);
        grid.set_cell(1, 2, Player::O);

        let mut game = Game::default();
        game.set_grid(grid);

        assert_eq!(game.find_winner(), Some(Player::O));
    }

    #[test]
    fn find_winner_find_diagonal() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Player::X);
        grid.set_cell(1, 1, Player::X);
        grid.set_cell(2, 2, Player::X);

        let mut game = Game::default();
        game.set_grid(grid);

        assert_eq!(game.find_winner(), Some(Player::X));
    }

    #[test]
    fn find_winner_finds_no_winner() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Player::X);
        grid.set_cell(0, 1, Player::O);
        grid.set_cell(0, 2, Player::X);

        let mut game = Game::default();
        game.set_grid(grid);

        assert!(game.find_winner().is_none());
    }
}
