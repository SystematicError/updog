use crate::search::{SearchFinalResult, SearchHandler, SearchResult, Searcher};
use crate::time::TimeManager;
use crate::transposition::TranspositionTable;
use crate::uci::{SearchOptions, TimeOptions};
use cozy_chess::{Board, Move};
use std::num::TryFromIntError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::spawn;

struct StandardHandler<F: Fn(SearchResult) + Send + 'static> {
    stop_flag: Arc<AtomicBool>,
    time_manager: TimeManager,
    node_limit: Option<usize>,
    handle_result: F,
}

impl<F: Fn(SearchResult) + Send + 'static> StandardHandler<F> {
    pub fn new(
        board: &Board,
        stop_flag: Arc<AtomicBool>,
        time_options: TimeOptions,
        node_limit: Option<usize>,
        handle_result: F,
    ) -> Self {
        Self {
            stop_flag,
            time_manager: time_options.manager(board),
            node_limit,
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

        self.time_manager.stopped()
            || self.stop_flag.load(Ordering::Acquire)
            || (self.node_limit.map_or(false, |limit| nodes > limit))
    }

    fn handle_result(&self, result: SearchResult) {
        (self.handle_result)(result);
    }
}

pub struct Engine {
    board: Board,
    board_hashes: Vec<u64>,
    transposition_table: Arc<TranspositionTable>,
    stop_flag: Arc<AtomicBool>,
}

impl Engine {
    pub fn with_table_size(bytes: usize) -> Result<Self, TryFromIntError> {
        let board = Board::default();

        Ok(Self {
            board_hashes: vec![board.hash()],
            board,
            transposition_table: Arc::new(TranspositionTable::with_size(bytes)?),
            stop_flag: Arc::new(AtomicBool::new(true)),
        })
    }

    pub fn new_game(&mut self) {
        let board = Board::default();
        self.board_hashes.clear();
        self.board_hashes.push(board.hash());
        self.board = board;

        self.set_stop_flag(true);
        self.clear_table();
    }

    pub fn clear_table(&self) {
        self.transposition_table.clear();
    }

    pub fn resize_table(&mut self, bytes: usize) -> Result<(), TryFromIntError> {
        let new_table = TranspositionTable::with_size(bytes)?;
        self.transposition_table = Arc::new(new_table);

        Ok(())
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
        handle_result: impl Fn(SearchResult) + Send + 'static,
        handle_final_result: impl Fn(SearchFinalResult) + Send + 'static,
    ) {
        self.set_stop_flag(false);

        let board = self.board.clone();
        let board_hashes = self.board_hashes.clone();
        let transposition_table = Arc::clone(&self.transposition_table);
        let handler = StandardHandler::new(
            &self.board,
            Arc::clone(&self.stop_flag),
            time_options,
            search_options.nodes,
            handle_result,
        );

        let mut searcher = Searcher::new(board, board_hashes, transposition_table, handler);

        spawn(move || {
            handle_final_result(searcher.deepen(search_options));
        });
    }

    fn set_stop_flag(&self, flag: bool) {
        self.stop_flag.store(flag, Ordering::Release);
    }

    pub fn stop(&self) {
        self.set_stop_flag(true);
    }
}
