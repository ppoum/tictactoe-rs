use std::fmt::Display;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Player {
    X,
    O,
}

impl Display for Player {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Player::X => write!(f, "X"),
            Player::O => write!(f, "O"),
        }
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct CellState(Option<Player>);

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
    pub fn set_cell(&mut self, row: usize, col: usize, player: Player) {
        self.inner[row * 3 + col] = CellState(Some(player));
    }

    pub fn rows(&self) -> impl Iterator<Item = &[CellState]> {
        self.inner.chunks(3)
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
