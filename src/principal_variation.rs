use crate::search::Depth;
use arrayvec::ArrayVec;
use cozy_chess::Move;

pub struct PVLine {
    line: ArrayVec<Move, { Depth::MAX as usize }>,
}

impl PVLine {
    pub fn new() -> Self {
        Self {
            line: ArrayVec::new(),
        }
    }

    pub fn moves(&self) -> &[Move] {
        &self.line.as_slice()
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
}
