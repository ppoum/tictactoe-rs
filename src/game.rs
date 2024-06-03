use std::fmt::Display;

use thiserror::Error;

use crate::{
    grid::{Grid, Mark},
    player::Player,
};

#[derive(Error, Debug)]
pub enum GameError {
    #[error("Cell is not empty")]
    UnemptyCell,
    #[error("Cell is out of bounds")]
    OutOfBounds,
}

#[derive(Debug)]
pub struct GamePlayer<'a> {
    pub mark: Mark,
    pub player: &'a dyn Player,
}

impl Display for GamePlayer<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.mark)
    }
}

pub struct Game {
    grid: Grid,
    player_x: Box<dyn Player>,
    player_o: Box<dyn Player>,
    is_x_turn: bool,
}

impl Game {
    pub fn new(player_x: impl Player + 'static, player_o: impl Player + 'static) -> Self {
        Self {
            player_x: Box::new(player_x),
            player_o: Box::new(player_o),
            grid: Grid::default(),
            is_x_turn: true,
        }
    }

    pub fn grid(&self) -> &Grid {
        &self.grid
    }

    pub fn current_player(&self) -> GamePlayer {
        if self.is_x_turn {
            GamePlayer {
                mark: Mark::X,
                player: self.player_x.as_ref(),
            }
        } else {
            GamePlayer {
                mark: Mark::O,
                player: self.player_o.as_ref(),
            }
        }
    }

    pub fn try_move(&mut self) -> Result<(), GameError> {
        let player = self.current_player().player;
        let (row, col) = player.get_move(self);

        if !(0..=2).contains(&row) || !(0..=2).contains(&col) {
            return Err(GameError::OutOfBounds);
        }

        if !self.grid().get_cell(row, col).is_empty() {
            return Err(GameError::UnemptyCell);
        }

        let cell_type = if self.is_x_turn { Mark::X } else { Mark::O };
        self.grid.set_cell(row, col, cell_type);

        self.is_x_turn = !self.is_x_turn;
        Ok(())
    }

    pub fn find_winner(&self) -> Option<GamePlayer> {
        // Detect row win
        for row in self.grid.rows() {
            if !row[0].is_empty() && row.iter().all(|&cell| cell == row[0]) {
                return row[0]
                    .try_get_mark()
                    .map(|mark| self.mark_to_game_player(mark));
            }
        }

        // Detect col win
        for col in self.grid.to_cols() {
            if !col[0].is_empty() && col.iter().all(|&cell| cell == col[0]) {
                return col[0]
                    .try_get_mark()
                    .map(|mark| self.mark_to_game_player(mark));
            }
        }

        // Detect diagonal (\)
        let first = self.grid.get_cell(0, 0);
        if !first.is_empty()
            && first == self.grid.get_cell(1, 1)
            && first == self.grid.get_cell(2, 2)
        {
            return first
                .try_get_mark()
                .map(|mark| self.mark_to_game_player(mark));
        }

        // Detect diagonal (/)
        let first = self.grid.get_cell(0, 2);
        if !first.is_empty()
            && first == self.grid.get_cell(1, 1)
            && first == self.grid.get_cell(2, 0)
        {
            return first
                .try_get_mark()
                .map(|mark| self.mark_to_game_player(mark));
        }

        None
    }

    /// Returns true if the grid is full
    pub fn is_filled(&self) -> bool {
        self.grid.is_full()
    }

    fn mark_to_game_player(&self, mark: &Mark) -> GamePlayer {
        match mark {
            Mark::X => GamePlayer {
                mark: *mark,
                player: self.player_x.as_ref(),
            },
            Mark::O => GamePlayer {
                mark: *mark,
                player: self.player_o.as_ref(),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::player::{self};

    use super::*;

    impl Game {
        fn set_grid(&mut self, grid: Grid) {
            self.grid = grid;
        }
    }

    fn mock_mock_game() -> Game {
        Game::new(
            player::tests::MockPlayer::default(),
            player::tests::MockPlayer::default(),
        )
    }

    #[test]
    fn try_move_rotates_player() {
        let player_x = player::tests::MockPlayer(0, 0);
        let player_o = player::tests::MockPlayer(1, 1);
        let mut game = Game::new(player_x, player_o);

        let player = game.current_player();
        assert_eq!(player.mark, Mark::X);
        assert!(game.try_move().is_ok());
        assert_eq!(game.grid().get_cell(0, 0).try_get_mark(), Some(&Mark::X));

        let player = game.current_player();
        assert_eq!(player.mark, Mark::O);
        assert!(game.try_move().is_ok());
        assert!(game.try_move().is_err())
    }

    #[test]
    fn find_winner_finds_horizontal_win() {
        let mut grid = Grid::default();
        grid.set_cell(2, 0, Mark::X);
        grid.set_cell(2, 1, Mark::X);
        grid.set_cell(2, 2, Mark::X);

        let mut game = mock_mock_game();
        game.set_grid(grid);

        assert_eq!(game.find_winner().unwrap().mark, Mark::X);
    }

    #[test]
    fn find_winner_finds_vertical_win() {
        let mut grid = Grid::default();
        grid.set_cell(1, 0, Mark::O);
        grid.set_cell(1, 1, Mark::O);
        grid.set_cell(1, 2, Mark::O);

        let mut game = mock_mock_game();
        game.set_grid(grid);

        assert_eq!(game.find_winner().unwrap().mark, Mark::O);
    }

    #[test]
    fn find_winner_find_diagonal() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 1, Mark::X);
        grid.set_cell(2, 2, Mark::X);

        let mut game = mock_mock_game();
        game.set_grid(grid);

        assert_eq!(game.find_winner().unwrap().mark, Mark::X);
    }

    #[test]
    fn find_winner_finds_no_winner() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 1, Mark::O);
        grid.set_cell(0, 2, Mark::X);

        let mut game = mock_mock_game();
        game.set_grid(grid);

        assert!(game.find_winner().is_none());
    }
}
