use crate::evaluate::{Evaluation, EvaluationUtils, evaluate};
use crate::ordering::generate_ordered_moves;
use crate::pv::PVLine;
use cozy_chess::{Board, Move};
use std::time::Duration;

pub type Ply = u8;

pub struct SearchOptions {
    // Standard clock timing
    pub wtime: Duration,
    pub btime: Duration,
    pub winc: Duration,
    pub binc: Duration,

    // Other timing options
    pub movetime: Duration,
    pub infinite: bool,

    // Search restrictions
    pub depth: Ply,
    pub nodes: usize,
}

impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            wtime: Duration::ZERO,
            btime: Duration::ZERO,
            winc: Duration::ZERO,
            binc: Duration::ZERO,

            movetime: Duration::ZERO,
            infinite: false,

            depth: Ply::MAX,
            nodes: usize::MAX,
        }
    }
}

pub struct Searcher {
    thread: SearchThread,
}

impl Searcher {
    pub fn new() -> Self {
        Self {
            thread: SearchThread::default(),
        }
    }

    pub fn board(&self) -> &Board {
        &self.thread.board
    }

    pub fn set_position(&mut self, mut board: Board, moves: Vec<Move>) {
        for mv in moves {
            board.play_unchecked(mv);
        }

        self.thread.set_position(board)
    }

    pub fn best_move(
        &mut self,
        options: SearchOptions,
        on_complete: impl FnOnce(Option<(&Board, Move)>),
    ) {
        on_complete(self.deepen(options).map(|mv| (self.board(), mv)));
    }

    fn deepen(&mut self, options: SearchOptions) -> Option<Move> {
        for depth in 1..=options.depth {
            let score = self.thread.search(depth);

            println!(
                "info depth {depth} nodes {} score {} pv {}",
                self.thread.info.nodes,
                score.display(),
                self.thread.pv_line.display(&self.thread.board)
            );
        }

        self.thread.pv_line.first()
    }
}

struct SearchInfo {
    nodes: usize,
}

impl SearchInfo {
    fn new() -> Self {
        Self { nodes: 0 }
    }

    fn reset(&mut self) {
        self.nodes = 0;
    }
}

struct SearchThread {
    board: Board,
    pv_line: PVLine,
    info: SearchInfo,
}

impl SearchThread {
    fn new(board: Board) -> Self {
        Self {
            board,
            pv_line: PVLine::new(),
            info: SearchInfo::new(),
        }
    }

    fn set_position(&mut self, board: Board) {
        self.board = board;
    }

    fn search(&mut self, depth: Ply) -> Evaluation {
        self.pv_line.clear();
        self.info.reset();

        Self::negamax(&self.board, &mut self.pv_line, &mut self.info, depth)
    }

    fn negamax(
        board: &Board,
        pv_line: &mut PVLine,
        info: &mut SearchInfo,
        depth: Ply,
    ) -> Evaluation {
        info.nodes += 1;

        if depth == 0 {
            return evaluate(board);
        }

        let mut best_score = -Evaluation::INFINITY;
        let new_line = &mut PVLine::new();

        for mv in generate_ordered_moves(board) {
            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            let score = -Self::negamax(&new_board, new_line, info, depth - 1);

            if score > best_score {
                best_score = score;
                pv_line.extend(mv, new_line);
            }
        }

        best_score
    }
}

impl Default for SearchThread {
    fn default() -> Self {
        Self::new(Board::default())
    }
}
