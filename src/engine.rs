use crate::position::Position;
use cozy_chess::{Board, Move};

pub struct Engine {
    position: Position,
}

impl Engine {
    pub fn new() -> Self {
        Self {
            position: Position::default(),
        }
    }

    pub fn set_position(&mut self, position: Position) {
        self.position = position;
    }

    pub fn best_move(&mut self) -> Option<(&Board, Move)> {
        let mut best_move = None;

        self.position.board().generate_moves(|moves| {
            for mv in moves {
                best_move = Some(mv);
                return true;
            }

            false
        });

        best_move.map(|mv| (self.position.board(), mv))
    }
}
