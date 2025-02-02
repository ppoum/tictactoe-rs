use std::{error::Error, fmt::Display};

#[derive(Copy, Clone, Debug)]
pub enum GridPlacementError {
    CellInUse,
    OutOfBounds,
}

impl Display for GridPlacementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CellInUse => write!(f, "Cell is not empty"),
            Self::OutOfBounds => write!(f, "Cell is out of bounds"),
        }
    }
}
impl Error for GridPlacementError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
}

impl Mark {
    /// Returns the opposite mark:
    /// - `Mark::X` returns `Mark::O`
    /// - `Mark::O` returns `Mark::X`
    pub fn opposite(&self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }
}

impl Display for Mark {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Mark::X => write!(f, "X"),
            Mark::O => write!(f, "O"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct CellState(Option<Mark>);

impl Display for CellState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.0 {
            None => write!(f, " "),
            Some(p) => write!(f, "{}", p),
        }
    }
}

impl CellState {
    pub fn is_empty(&self) -> bool {
        self.0.is_none()
    }

    pub fn try_get_mark(&self) -> Option<&Mark> {
        self.0.as_ref()
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct Grid {
    inner: [CellState; 9],
}

impl Display for Grid {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.fmt_inner(f)
    }
}

impl Grid {
    pub fn get_cell(&self, row: usize, col: usize) -> &CellState {
        &self.inner[row * 3 + col]
    }

    pub fn set_cell(&mut self, row: usize, col: usize, mark: Mark) {
        self.inner[row * 3 + col] = CellState(Some(mark));
    }

    pub fn try_set_cell(
        &mut self,
        row: usize,
        col: usize,
        mark: Mark,
    ) -> Result<(), GridPlacementError> {
        if !(0..=2).contains(&row) || !(0..=2).contains(&col) {
            return Err(GridPlacementError::OutOfBounds);
        }

        if !self.get_cell(row, col).is_empty() {
            return Err(GridPlacementError::CellInUse);
        }

        self.inner[row * 3 + col] = CellState(Some(mark));
        Ok(())
    }

    pub fn rows(&self) -> impl Iterator<Item = &[CellState]> {
        self.inner.chunks(3)
    }

    pub fn to_cols(&self) -> impl Iterator<Item = [CellState; 3]> {
        let mut cols = [[Default::default(); 3]; 3];
        for (r, row) in self.rows().map(|c| c.to_vec()).enumerate() {
            for (c, cell) in row.into_iter().enumerate() {
                cols[c][r] = cell;
            }
        }
        cols.into_iter()
    }

    pub fn cell_count(&self) -> usize {
        self.inner.iter().filter(|c| !c.is_empty()).count()
    }

    pub fn is_full(&self) -> bool {
        self.inner.iter().all(|c| !c.is_empty())
    }

    pub fn get_winning_mark(&self) -> Option<Mark> {
        // Detect row win
        for row in self.rows() {
            if !row[0].is_empty() && row.iter().all(|&cell| cell == row[0]) {
                return row[0].try_get_mark().copied();
            }
        }

        // Detect col win
        for col in self.to_cols() {
            if !col[0].is_empty() && col.iter().all(|&cell| cell == col[0]) {
                return col[0].try_get_mark().copied();
            }
        }

        // Detect diagonal (\)
        let first = self.get_cell(0, 0);
        if !first.is_empty() && first == self.get_cell(1, 1) && first == self.get_cell(2, 2) {
            return first.try_get_mark().copied();
        }

        // Detect diagonal (/)
        let first = self.get_cell(0, 2);
        if !first.is_empty() && first == self.get_cell(1, 1) && first == self.get_cell(2, 0) {
            return first.try_get_mark().copied();
        }

        None
    }

    #[cfg(not(feature = "unicode"))]
    fn fmt_inner(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Horizontal len = Left serparator + 3 * (left pad + cell value + pad + right separator)
        let side_string = "-".repeat(1 + 3 * 4);
        // Top
        writeln!(f, "{}", side_string)?;
        for row in self.rows() {
            let value_line = row
                .iter()
                .fold("|".to_owned(), |acc, cell| format!("{acc} {cell} |"));
            writeln!(f, "{}", value_line)?;
            writeln!(f, "{}", side_string)?;
        }
        Ok(())
    }

    #[cfg(feature = "unicode")]
    fn fmt_inner(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Horizontal top line: left corner + 2 * (2x line (padding) + line (value) + down part) +
        // (3 lines + right corner)
        let top_line = " \u{250C}".to_owned()
            + &"\u{2500}\u{2500}\u{2500}\u{252C}".repeat(2)
            + "\u{2500}\u{2500}\u{2500}\u{2510}";

        // Same, but corners and down part are replaced
        let middle_line = " \u{251C}".to_owned()
            + &"\u{2500}\u{2500}\u{2500}\u{253C}".repeat(2)
            + "\u{2500}\u{2500}\u{2500}\u{2524}";
        let bottom_line = " \u{2514}".to_owned()
            + &"\u{2500}\u{2500}\u{2500}\u{2534}".repeat(2)
            + "\u{2500}\u{2500}\u{2500}\u{2518}";
        writeln!(f, "{}", top_line)?;
        for (n, row) in self.rows().enumerate() {
            let value_line = row.iter().fold(" \u{2502}".to_owned(), |acc, cell| {
                format!("{acc} {cell} \u{2502}")
            });
            writeln!(f, "{}", value_line)?;
            if n == 2 {
                writeln!(f, "{}", bottom_line)?;
            } else {
                writeln!(f, "{}", middle_line)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_full_detects_full_grid() {
        let mut grid = Grid::default();
        for r in 0..=2 {
            for c in 0..=2 {
                grid.set_cell(r, c, Mark::X);
            }
        }

        assert!(grid.is_full())
    }

    #[test]
    fn find_winner_finds_horizontal_win() {
        for row in 0..=2 {
            let mut grid = Grid::default();
            grid.set_cell(row, 0, Mark::X);
            grid.set_cell(row, 1, Mark::X);
            grid.set_cell(row, 2, Mark::X);

            assert_eq!(grid.get_winning_mark(), Some(Mark::X));
        }
    }

    #[test]
    fn find_winner_finds_vertical_win() {
        for col in 0..=2 {
            let mut grid = Grid::default();
            grid.set_cell(0, col, Mark::O);
            grid.set_cell(1, col, Mark::O);
            grid.set_cell(2, col, Mark::O);

            assert_eq!(grid.get_winning_mark(), Some(Mark::O));
        }
    }

    #[test]
    fn find_winner_find_diagonal() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(1, 1, Mark::X);
        grid.set_cell(2, 2, Mark::X);
        assert_eq!(grid.get_winning_mark(), Some(Mark::X));

        let mut grid = Grid::default();
        grid.set_cell(0, 2, Mark::X);
        grid.set_cell(1, 1, Mark::X);
        grid.set_cell(2, 0, Mark::X);
        assert_eq!(grid.get_winning_mark(), Some(Mark::X));
    }

    #[test]
    fn find_winner_finds_no_winner() {
        let mut grid = Grid::default();
        grid.set_cell(0, 0, Mark::X);
        grid.set_cell(0, 1, Mark::O);
        grid.set_cell(0, 2, Mark::X);

        assert!(grid.get_winning_mark().is_none())
    }
}
