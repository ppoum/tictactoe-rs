use thiserror::Error;

use crate::grid::{Grid, Player};

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Cell is not empty")]
    UnemptyCell,
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
        if !self.grid().get_cell(row, col).is_empty() {
            return Err(GameError::UnemptyCell);
        }

        let cell_type = if self.is_x_turn { Player::X } else { Player::O };
        self.grid.set_cell(row, col, cell_type);

        self.is_x_turn = !self.is_x_turn;
        Ok(())
    }
}
