use crate::search::Ply;
use arrayvec::ArrayVec;
use cozy_chess::util::display_uci_move;
use cozy_chess::{Board, Move};
use std::fmt;

pub struct PVLine {
    line: ArrayVec<Move, { Ply::MAX as usize }>,
}

impl PVLine {
    pub fn new() -> Self {
        Self {
            line: ArrayVec::new(),
        }
    }

    pub fn first(&self) -> Option<Move> {
        self.line.first().copied()
    }

    pub fn clear(&mut self) {
        self.line.clear();
    }

    pub fn extend(&mut self, mv: Move, new_line: &Self) {
        self.line.clear();
        self.line.push(mv);
        self.line.try_extend_from_slice(&new_line.line).unwrap();
    }

    pub fn display<'a>(&'a self, board: &'a Board) -> impl fmt::Display {
        PVLineDisplay {
            pv_line: self,
            board,
        }
    }
}

struct PVLineDisplay<'a> {
    pv_line: &'a PVLine,
    board: &'a Board,
}

impl fmt::Display for PVLineDisplay<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for (i, &mv) in self.pv_line.line.iter().enumerate() {
            if i > 0 {
                write!(formatter, " ")?;
            }

            write!(formatter, "{}", display_uci_move(self.board, mv))?;
        }

        Ok(())
    }
}
