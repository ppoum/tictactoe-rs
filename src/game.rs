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
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
