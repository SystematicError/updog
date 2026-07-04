use crate::search::Depth;
use cozy_chess::Move;

pub struct PVLine {
    size: usize,
    line: [Move; Depth::MAX as usize],
}

impl PVLine {
    pub fn new() -> Self {
        Self {
            size: 0,
            line: [unsafe { std::mem::zeroed() }; Depth::MAX as usize],
        }
    }

    pub fn moves(&self) -> &[Move] {
        &self.line[..self.size]
    }

    pub fn first(&self) -> Option<Move> {
        if self.size <= 0 {
            return None;
        }

        Some(self.line[0])
    }

    pub fn clear(&mut self) {
        self.size = 0;
    }

    pub fn extend(&mut self, mv: Move, new_line: &Self) {
        self.line[0] = mv;
        self.line[1..=new_line.size].copy_from_slice(&new_line.line[..new_line.size]);
        self.size = new_line.size + 1;
    }
}
