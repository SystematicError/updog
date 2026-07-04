use crate::principal_variation::PVLine;
use crate::search::deepen;
use cozy_chess::{Board, Move};

pub struct Engine {
    board: Board,
    history: Vec<u64>,
    pv_line: PVLine,
}

impl Engine {
    pub fn new() -> Self {
        let board = Board::default();

        Self {
            history: vec![board.hash()],
            board: board,
            pv_line: PVLine::new(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn set_position(&mut self, board: Board, history: Vec<u64>) {
        self.board = board;
        self.history = history;
    }

    pub fn best_move(&mut self) -> Option<(&Board, Move)> {
        deepen(&self.board, &mut self.history, &mut self.pv_line).map(|mv| (&self.board, mv))
    }
}
