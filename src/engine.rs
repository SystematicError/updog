use cozy_chess::{Board, Move};

pub struct Engine {
    board: Board,
    history: Vec<u64>,
}

impl Engine {
    pub fn new() -> Self {
        let board = Board::default();

        Self {
            history: vec![board.hash()],
            board: board,
        }
    }

    pub fn set_position(&mut self, board: Board, history: Vec<u64>) {
        self.board = board;
        self.history = history;
    }

    pub fn best_move(&mut self) -> Option<(&Board, Move)> {
        let mut best_move = None;

        self.board.generate_moves(|moves| {
            for mv in moves {
                best_move = Some(mv);
                return true;
            }

            false
        });

        best_move.map(|mv| (&self.board, mv))
    }
}
