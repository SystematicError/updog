use cozy_chess::{Board, IllegalMoveError, Move};

pub struct Position {
    board: Board,
    history: Vec<u64>,
}

impl Position {
    pub fn new(board: Board) -> Self {
        Self {
            history: vec![board.hash()],
            board: board,
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn play_unchecked(&mut self, mv: Move) {
        self.board.play_unchecked(mv);
        self.history.push(self.board.hash());
    }

    pub fn try_play(&mut self, mv: Move) -> Result<(), IllegalMoveError> {
        self.board.try_play(mv)?;
        self.history.push(self.board.hash());
        Ok(())
    }
}

impl Default for Position {
    fn default() -> Self {
        Self::new(Board::default())
    }
}
