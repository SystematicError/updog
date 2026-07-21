use crate::search::{SearchOptions, deepen};
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

    pub fn best_move(&self, options: SearchOptions, on_complete: impl FnOnce(Option<Move>)) {
        on_complete(deepen(&self.board, options));
    }
}
