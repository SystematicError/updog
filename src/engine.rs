use cozy_chess::{Board, Move};

pub struct Engine {
    board: Board,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            board: Board::default(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn set_position(&mut self, mut board: Board, moves: Vec<Move>) {
        for mv in moves {
            board.play_unchecked(mv);
        }

        self.board = board;
    }
}
