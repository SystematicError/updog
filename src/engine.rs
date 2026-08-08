use crate::search::{SearchFinalResult, SearchHandler, SearchResult, Searcher};
use crate::time::TimeManager;
use crate::uci::{SearchOptions, TimeOptions};
use cozy_chess::{Board, Move};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

struct StandardHandler<F: Fn(SearchResult) + Send + 'static> {
    stop_flag: Arc<AtomicBool>,
    time_manager: TimeManager,
    handle_result: F,
}

impl<F: Fn(SearchResult) + Send + 'static> StandardHandler<F> {
    pub fn new(
        board: &Board,
        stop_flag: Arc<AtomicBool>,
        time_options: TimeOptions,
        handle_result: F,
    ) -> Self {
        Self {
            stop_flag,
            time_manager: time_options.manager(board),
            handle_result,
        }
    }
}

const STOP_CHECK_FREQUENCY: usize = 1024;

impl<F: Fn(SearchResult) + Send + 'static> SearchHandler for StandardHandler<F> {
    fn stopped(&self, nodes: usize) -> bool {
        if !nodes.is_multiple_of(STOP_CHECK_FREQUENCY) {
            return false;
        }

        self.time_manager.stopped() || self.stop_flag.load(Ordering::Acquire)
    }

    fn handle_result(&self, result: SearchResult) {
        (self.handle_result)(result);
    }
}

pub struct Engine {
    board: Board,
    board_hashes: Vec<u64>,
    stop_flag: Arc<AtomicBool>,
}

impl Engine {
    pub fn new() -> Self {
        let board = Board::default();

        Self {
            board_hashes: vec![board.hash()],
            board,
            stop_flag: Arc::new(AtomicBool::new(true)),
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
        &mut self,
        time_options: TimeOptions,
        search_options: SearchOptions,
        handle_result: impl Fn(SearchResult) + Send + 'static,
        handle_final_result: impl Fn(SearchFinalResult) + Send + 'static,
    ) {
        self.set_stop_flag(false);

        let handler = StandardHandler::new(
            &self.board,
            Arc::clone(&self.stop_flag),
            time_options,
            handle_result,
        );

        let mut searcher = Searcher::new(self.board.clone(), self.board_hashes.clone(), handler);

        spawn(move || {
            handle_final_result(searcher.deepen(search_options));
        });
    }

    fn set_stop_flag(&mut self, flag: bool) {
        self.stop_flag.store(flag, Ordering::Release);
    }

    pub fn stop(&mut self) {
        self.set_stop_flag(true);
    }
}
