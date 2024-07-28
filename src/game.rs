use std::{
    error::Error,
    fmt::Display,
    io::{self, BufRead, BufReader, BufWriter, ErrorKind, Write},
    net::{TcpListener, TcpStream, ToSocketAddrs},
};

use crate::{
    grid::{Grid, GridPlacementError, Mark},
    player::Player,
    protocol::{self, ClientHello, PlayerMove, ServerHello},
};

use self::seal::ServerGameState;

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
    pub fn new(player_x: Box<dyn Player>, player_o: Box<dyn Player>) -> Self {
        Self {
            player_x,
            player_o,
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

    pub fn try_move(&mut self) -> Result<(), GridPlacementError> {
        let game_player = self.current_player();
        let (row, col) = game_player.player.get_move(self.grid(), &game_player.mark);

        let mark = if self.is_x_turn { Mark::X } else { Mark::O };
        self.grid.try_set_cell(row, col, mark)?;

        self.is_x_turn = !self.is_x_turn;
        Ok(())
    }

    pub fn find_winner(&self) -> Option<GamePlayer> {
        self.grid
            .get_winning_mark()
            .map(|m| self.mark_to_game_player(&m))
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

#[derive(Debug)]
pub enum NetworkedGameError {
    PlayError(GridPlacementError),
    Io(io::Error),
}

impl Display for NetworkedGameError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlayError(e) => write!(f, "Error while trying a move: {}", e),
            Self::Io(e) => write!(f, "IO error while playing: {}", e),
        }
    }
}
impl Error for NetworkedGameError {}

impl From<GridPlacementError> for NetworkedGameError {
    fn from(value: GridPlacementError) -> Self {
        Self::PlayError(value)
    }
}

impl From<io::Error> for NetworkedGameError {
    fn from(value: io::Error) -> Self {
        Self::Io(value)
    }
}

pub trait NetworkedGame {
    fn grid(&self) -> &Grid;

    fn grid_mut(&mut self) -> &mut Grid;

    fn set_next_turn(&mut self);

    fn is_local_turn(&self) -> bool;

    fn local_mark(&self) -> Mark;
}

trait InternalNetworkBufAccessor {
    fn reader(&mut self) -> &mut BufReader<TcpStream>;
    fn writer(&mut self) -> &mut BufWriter<TcpStream>;
}

#[derive(Debug)]
pub struct RemoteGame {
    reader: BufReader<TcpStream>,
    writer: BufWriter<TcpStream>,
    grid: Grid,
    is_local_turn: bool,
    local_mark: Mark,
}

impl NetworkedGame for RemoteGame {
    fn grid(&self) -> &Grid {
        &self.grid
    }

    fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn is_local_turn(&self) -> bool {
        self.is_local_turn
    }

    fn local_mark(&self) -> Mark {
        self.local_mark
    }

    fn set_next_turn(&mut self) {
        self.is_local_turn = !self.is_local_turn;
    }
}

impl InternalNetworkBufAccessor for RemoteGame {
    fn reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.reader
    }

    fn writer(&mut self) -> &mut BufWriter<TcpStream> {
        &mut self.writer
    }
}

impl RemoteGame {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<RemoteGame> {
        let stream = TcpStream::connect(addr)?;

        let mut reader = BufReader::new(stream.try_clone()?);
        let mut writer = BufWriter::new(stream);
        writer.write_all(&ClientHello.to_bytes())?;
        writer.flush()?;

        let mut buf = vec![];
        reader.read_until(protocol::TERMINATOR, &mut buf)?;
        buf.pop();

        let server_hello = ServerHello::try_from(buf.as_slice()).map_err(|_| {
            io::Error::new(
                ErrorKind::InvalidData,
                "Received malformed SERVER_HELLO packet",
            )
        })?;

        Ok(Self {
            reader,
            writer,
            grid: Grid::default(),
            is_local_turn: server_hello.client_first,
            local_mark: server_hello.client_mark,
        })
    }

    /// * If local is playing, asks the user for input
    /// * If remote is playing, get move from connection
    pub fn try_move(&mut self, local_player: &dyn Player) -> Result<(), NetworkedGameError> {
        try_networked_move(self, local_player)
    }
}

mod seal {
    pub trait ServerGameState {}
}

pub struct NewState(TcpListener);
impl ServerGameState for NewState {}

pub struct ConnectedState(BufReader<TcpStream>, BufWriter<TcpStream>);
impl ServerGameState for ConnectedState {}

#[derive(Debug)]
pub struct ServerGame<S: ServerGameState> {
    state: S,
    grid: Grid,
    is_local_turn: bool,
    local_mark: Mark,
}

#[derive(Clone, Copy, Debug)]
/// Defaults: host playing first with the `X` mark
pub struct ServerGameSettings {
    pub host_plays_first: bool,
    pub host_mark: Mark,
}

impl Default for ServerGameSettings {
    fn default() -> Self {
        Self {
            host_plays_first: true,
            host_mark: Mark::X,
        }
    }
}

impl ServerGame<NewState> {
    pub fn bind<A: ToSocketAddrs>(addr: A, settings: &ServerGameSettings) -> io::Result<Self> {
        let state = NewState(TcpListener::bind(addr)?);

        Ok(Self {
            state,
            grid: Grid::default(),
            is_local_turn: settings.host_plays_first,
            local_mark: settings.host_mark,
        })
    }

    pub fn listen(self) -> io::Result<ServerGame<ConnectedState>> {
        let listener = self.state.0;

        let reader;
        let writer;
        loop {
            let (socket, _) = listener.accept()?;

            let mut r = BufReader::new(socket.try_clone()?);
            let mut w = BufWriter::new(socket);

            // Expect CLIENT_HELLO
            let mut buf = vec![];
            r.read_until(protocol::TERMINATOR, &mut buf)?;
            buf.pop();
            match ClientHello::try_from(buf.as_slice()) {
                Ok(_) => {}
                Err(_) => continue,
            }

            // Send SERVER_HELLO
            let pkt = ServerHello {
                client_first: !self.is_local_turn,
                client_mark: self.local_mark.opposite(),
            }
            .to_bytes();
            w.write_all(&pkt)?;
            w.flush()?;

            reader = r;
            writer = w;
            break;
        }

        let state = ConnectedState(reader, writer);

        Ok(ServerGame::<ConnectedState> {
            state,
            grid: self.grid,
            is_local_turn: self.is_local_turn,
            local_mark: self.local_mark,
        })
    }
}

impl NetworkedGame for ServerGame<ConnectedState> {
    fn grid(&self) -> &Grid {
        &self.grid
    }

    fn grid_mut(&mut self) -> &mut Grid {
        &mut self.grid
    }

    fn is_local_turn(&self) -> bool {
        self.is_local_turn
    }

    fn local_mark(&self) -> Mark {
        self.local_mark
    }

    fn set_next_turn(&mut self) {
        self.is_local_turn = !self.is_local_turn;
    }
}

impl InternalNetworkBufAccessor for ServerGame<ConnectedState> {
    fn reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.state.0
    }

    fn writer(&mut self) -> &mut BufWriter<TcpStream> {
        &mut self.state.1
    }
}

impl ServerGame<ConnectedState> {
    /// * If local is playing, asks the user for input
    /// * If remote is playing, get move from connection
    pub fn try_move(&mut self, local_player: &dyn Player) -> Result<(), NetworkedGameError> {
        try_networked_move(self, local_player)
    }
}

fn try_networked_move<G: NetworkedGame + InternalNetworkBufAccessor>(
    game: &mut G,
    local_player: &dyn Player,
) -> Result<(), NetworkedGameError> {
    // Get move
    let (row, col) = if game.is_local_turn() {
        local_player.get_move(game.grid(), &game.local_mark())
    } else {
        let mut buf = vec![];
        game.reader().read_until(protocol::TERMINATOR, &mut buf)?;

        // Expect 1 data byte + terminator
        if buf.len() != 2 {
            return Err(
                io::Error::new(ErrorKind::InvalidData, "PlayerMove packet too long").into(),
            );
        }

        PlayerMove::from(buf[0]).to_tuple()
    };

    // Try applying move
    let mark = if game.is_local_turn() {
        game.local_mark()
    } else {
        game.local_mark().opposite()
    };
    game.grid_mut().try_set_cell(row, col, mark)?;

    if game.is_local_turn() {
        // Send move to remote player
        let pkt = PlayerMove(row, col);
        game.writer().write_all(&pkt.to_bytes())?;
        game.writer().flush()?;
    }

    game.set_next_turn();
    Ok(())
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
            Box::<player::tests::MockPlayer>::default(),
            Box::<player::tests::MockPlayer>::default(),
        )
    }

    #[test]
    fn try_move_rotates_player() {
        let player_x = Box::new(player::tests::MockPlayer(0, 0));
        let player_o = Box::new(player::tests::MockPlayer(1, 1));
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
