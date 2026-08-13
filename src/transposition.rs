use crate::evaluate::Evaluation;
use crate::search::Ply;
use bytemuck::{Pod, Zeroable, cast, zeroed_slice_box};
use cozy_chess::{Board, Move, Piece, Square};
use std::mem::size_of;
use std::num::{NonZeroUsize, TryFromIntError};
use std::sync::atomic::{AtomicU64, Ordering};

#[repr(u8)]
#[derive(PartialEq)]
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
            _padding: 0,
        }
    }
}

impl From<EncodedData> for Data {
    fn from(encoded_data: EncodedData) -> Self {
        Self {
            score: encoded_data.score,
            bound: match encoded_data.bound {
                1 => Bound::Lower,
                2 => Bound::Upper,
                _ => Bound::Exact,
            },
            depth: encoded_data.depth,
            best_move: Move {
                from: Square::index(encoded_data.best_move_from as usize),
                to: Square::index(encoded_data.best_move_to as usize),
                promotion: Piece::try_index(encoded_data.best_move_promotion as usize),
            },
        }
    }
}

// Entries use the XOR technique for lockless access and data consistency
// https://craftychess.com/hyatt/hashing.html
#[derive(Zeroable)]
struct Entry {
    key: AtomicU64,
    data: AtomicU64,
}

impl Entry {
    fn store(&self, hash: u64, data: Data) {
        let data = cast(EncodedData::from(data));
        let key = hash ^ data;

        self.key.store(key, Ordering::Relaxed);
        self.data.store(data, Ordering::Relaxed);
    }

    fn load(&self, hash: u64) -> Option<Data> {
        let key = self.key.load(Ordering::Relaxed);
        let data = self.data.load(Ordering::Relaxed);

        // Check for empty entries
        if data == 0 {
            return None;
        }

        // Check for conflicting hashes or inconsistent entries
        if key ^ data != hash {
            return None;
        }

        Some(Data::from(cast::<_, EncodedData>(data)))
    }

    fn clear(&self) {
        self.key.store(0, Ordering::Relaxed);
        self.data.store(0, Ordering::Relaxed);
    }
}

pub struct TranspositionTable {
    table: Box<[Entry]>,
}

impl TranspositionTable {
    pub fn with_entries(entries: NonZeroUsize) -> Self {
        Self {
            table: zeroed_slice_box(entries.get()),
        }
    }

    pub fn with_size(bytes: usize) -> Result<Self, TryFromIntError> {
        let entries = bytes / size_of::<Entry>();
        Ok(Self::with_entries(NonZeroUsize::try_from(entries)?))
    }

    fn index(&self, hash: u64) -> usize {
        hash as usize % self.table.len()
    }

    pub fn set(&self, board: &Board, data: Data) {
        let hash = board.hash();
        let entry = &self.table[self.index(hash)];
        entry.store(hash, data);
    }

    pub fn get(&self, board: &Board) -> Option<Data> {
        let hash = board.hash();
        let entry = &self.table[self.index(hash)];
        let data = entry.load(hash)?;

        Some(data)
    }

    pub fn clear(&self) {
        for entry in self.table.iter() {
            entry.clear();
        }
    }
}
