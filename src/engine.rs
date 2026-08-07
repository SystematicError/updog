use crate::search::{SearchResult, deepen};
use crate::uci::{SearchOptions, TimeOptions};
use cozy_chess::{Board, Move};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

pub struct Engine {
    board: Board,
    board_hashes: Vec<u64>,
    stop: Arc<AtomicBool>,
}

impl Engine {
    pub fn new() -> Self {
        let board = Board::default();

        Self {
            board_hashes: vec![board.hash()],
            board,
            stop: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn set_position(&mut self, mut board: Board, moves: Vec<Move>) {
        self.board_hashes.clear();
        self.board_hashes.push(board.hash());

        for mv in moves {
            board.play_unchecked(mv);
            self.board_hashes.push(board.hash());
        }

        self.board = board;
    }

    pub fn best_move(
        &self,
        time_options: TimeOptions,
        search_options: SearchOptions,
        on_complete: impl FnOnce(SearchResult) + Send + 'static,
    ) {
        let board = self.board.clone();
        let board_hashes = self.board_hashes.clone();
        let stop = Arc::clone(&self.stop);

        spawn(|| {
            on_complete(deepen::<true>(
                board,
                board_hashes,
                time_options,
                search_options,
                stop,
            ))
        });
    }

    pub fn stop(&mut self) {
        self.stop.store(true, Ordering::Release);
    }
}
