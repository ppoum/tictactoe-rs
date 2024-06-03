use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mark {
    X,
    O,
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

    pub fn is_full(&self) -> bool {
        !self.inner.iter().any(|c| c.is_empty())
    }

    #[cfg(not(feature = "unicode"))]
    fn fmt_inner(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Horizontal len = Left serparator + 3 * (left pad + cell value + pad + right separator)
        let side_string = "-".repeat(1 + 3 * 4);
        // Top
        writeln!(f, "{}", side_string)?;
        for row in self.rows() {
            let test = row
                .iter()
                .fold("|".to_owned(), |acc, cell| format!("{acc} {cell} |"));
            writeln!(f, "{}", test)?;
            writeln!(f, "{}", side_string)?;
        }
        Ok(())
    }

    #[cfg(feature = "unicode")]
    fn fmt_inner(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
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
}
