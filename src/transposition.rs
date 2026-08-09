use crate::evaluate::Evaluation;
use crate::search::Ply;
use bytemuck::{Pod, Zeroable, cast};
use cozy_chess::{Board, Move, Piece, Square};
use std::mem::size_of;
use std::num::{NonZeroUsize, TryFromIntError};
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(u8)]
pub enum Bound {
    Exact = 0,
    Lower = 1,
    Upper = 2,
}

pub struct Data {
    pub score: Evaluation,
    pub bound: Bound,
    pub depth: Ply,
    pub best_move: Move,
}

#[derive(Clone, Copy, Pod, Zeroable)]
#[repr(C)]
struct EncodedData {
    score: i16,
    bound: u8,
    depth: u8,
    best_move_from: u8,
    best_move_to: u8,
    best_move_promotion: u8,
    _padding: u8,
}

const PADDING: u8 = u8::MAX;

impl From<Data> for EncodedData {
    fn from(data: Data) -> Self {
        Self {
            score: data.score,
            bound: data.bound as u8,
            depth: data.depth,
            best_move_from: data.best_move.from as u8,
            best_move_to: data.best_move.to as u8,
            best_move_promotion: data
                .best_move
                .promotion
                .map_or(u8::MAX, |piece| piece as u8),
            _padding: PADDING,
        }
    }
}

// TODO: Implement From instead of TryFrom for decoding

impl TryFrom<EncodedData> for Data {
    type Error = ();

    fn try_from(encoded_data: EncodedData) -> Result<Self, Self::Error> {
        // Padding is non-zero, and empty entries are all zero
        // Hence this check failing can be used as a check for empty entries
        if encoded_data._padding != PADDING {
            return Err(());
        }

        Ok(Self {
            score: encoded_data.score,
            bound: match encoded_data.bound {
                0 => Bound::Exact,
                1 => Bound::Lower,
                2 => Bound::Upper,
                _ => return Err(()),
            },
            depth: encoded_data.depth,
            best_move: Move {
                from: Square::try_index(encoded_data.best_move_from as usize).ok_or(())?,
                to: Square::try_index(encoded_data.best_move_to as usize).ok_or(())?,
                promotion: Piece::try_index(encoded_data.best_move_promotion as usize),
            },
        })
    }
}

struct Entry {
    key: AtomicU64,
    data: AtomicU64,
}

impl Entry {
    fn new() -> Self {
        Self {
            key: AtomicU64::new(0),
            data: AtomicU64::new(0),
        }
    }

    fn store(&self, hash: u64, data: Data) {
        let data = cast(EncodedData::from(data));
        let key = hash ^ data;

        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    fn load(&self, hash: u64) -> Option<Data> {
        let key = self.key.load(Ordering::Relaxed);
        let data = self.data.load(Ordering::Relaxed);

        // Check for conflicting hashes or inconsistent entries
        if key ^ data != hash {
            return None;
        }

        Data::try_from(cast::<u64, EncodedData>(data)).ok()
    }
}

pub struct TranspositionTable {
    table: Box<[Entry]>,
}

impl TranspositionTable {
    pub fn with_entries(entries: NonZeroUsize) -> Self {
        Self {
            table: (0..entries.get()).map(|_| Entry::new()).collect(),
        }
    }

    pub fn with_size(bytes: usize) -> Result<Self, TryFromIntError> {
        let entries = bytes / size_of::<Entry>();
        Ok(Self::with_entries(NonZeroUsize::try_from(entries)?))
    }

    fn index(&self, hash: u64) -> usize {
        hash as usize % self.table.len()
    }

    pub fn set(
        &self,
        board: &Board,
        score: Evaluation,
        alpha: Evaluation,
        beta: Evaluation,
        depth: Ply,
        best_move: Move,
    ) {
        let hash = board.hash();
        let entry = &self.table[self.index(hash)];

        let bound = if score <= alpha {
            Bound::Upper
        } else if score >= beta {
            Bound::Lower
        } else {
            Bound::Exact
        };

        let data = Data {
            score,
            bound,
            depth,
            best_move,
        };

        entry.store(hash, data);
    }

    pub fn get(&self, board: &Board) -> Option<Data> {
        let hash = board.hash();
        let entry = &self.table[self.index(hash)];
        let data = entry.load(hash)?;

        Some(data)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_entry_load_is_none() {
        assert!(Entry::new().load(0).is_none());
    }
}
