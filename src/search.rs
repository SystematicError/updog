use crate::evaluate::{Evaluation, EvaluationUtils, evaluate};
use crate::ordering::order_moves;
use crate::pv::PVLine;
use crate::transposition::{Bound, Data, TranspositionTable};
use crate::uci::SearchOptions;
use cozy_chess::{Board, GameStatus, Move};
use std::sync::Arc;
use std::thread::sleep;
use std::time::{Duration, Instant};

pub type Ply = u8;

pub struct SearchInfo {
    pub start: Instant,
    pub nodes: usize,
    pub stopped: bool,
}

pub struct SearchResult<'a> {
    pub board: &'a Board,
    pub depth: Ply,
    pub score: Evaluation,
    pub info: &'a SearchInfo,
    pub hashfull: usize,
    pub pv_line: &'a PVLine,
}

pub struct SearchFinalResult<'a> {
    pub board: &'a Board,
    pub info: SearchInfo,
    pub best_move: Option<Move>,
}

pub trait SearchHandler {
    fn stopped(&self, nodes: usize) -> bool;
    fn handle_result(&self, result: SearchResult);
}

pub struct Searcher<H: SearchHandler> {
    board: Board,
    board_hashes: Vec<u64>,
    transposition_table: Arc<TranspositionTable>,
    handler: H,
}

impl<H: SearchHandler> Searcher<H> {
    pub fn new(
        board: Board,
        board_hashes: Vec<u64>,
        transposition_table: Arc<TranspositionTable>,
        handler: H,
    ) -> Self {
        Self {
            board,
            board_hashes,
            transposition_table,
            handler,
        }
    }

    pub fn deepen(&mut self, search_options: SearchOptions) -> SearchFinalResult<'_> {
        let mut pv_line = PVLine::new();
        let mut thread = SearchThread {
            board_hashes: &mut self.board_hashes,
            transposition_table: &self.transposition_table,
            handler: &self.handler,
            info: SearchInfo {
                nodes: 0,
                stopped: false,
                start: Instant::now(),
            },
            best_score: -Evaluation::INFINITY,
            best_move: None,
        };

        for depth in 1..=search_options.depth.unwrap_or(Ply::MAX) {
            let score = thread.negamax::<true>(
                &self.board,
                &mut pv_line,
                -Evaluation::INFINITY,
                Evaluation::INFINITY,
                depth,
                0,
            );

            // Skip result handling if the iteration was interrupted
            if thread.info.stopped {
                break;
            }

            // Override the best score and move with that of the completed iteration
            // Important for when an iteration reports a worse score for the position
            thread.best_score = score;
            thread.best_move = pv_line.first();

            self.handler.handle_result(SearchResult {
                board: &self.board,
                depth,
                score,
                info: &thread.info,
                hashfull: self.transposition_table.len_permille(),
                pv_line: &pv_line,
            });

            // NOTE: May not be necessary, since pv lines are constructed form the root onwards
            pv_line.clear();

            if self.handler.stopped(0) {
                break;
            }
        }

        // Only terminate infinite searches when manually stopped
        if search_options.depth.is_none() {
            while !self.handler.stopped(0) {
                sleep(Duration::from_millis(5));
            }
        }

        SearchFinalResult {
            board: &self.board,
            best_move: thread.best_move,
            info: thread.info,
        }
    }
}

struct SearchThread<'a, H: SearchHandler> {
    board_hashes: &'a mut Vec<u64>,
    transposition_table: &'a Arc<TranspositionTable>,
    handler: &'a H,
    info: SearchInfo,
    best_score: Evaluation,
    best_move: Option<Move>,
}

impl<H: SearchHandler> SearchThread<'_, H> {
    fn negamax<const PV_NODE: bool>(
        &mut self,
        board: &Board,
        pv_line: &mut PVLine,
        mut alpha: Evaluation,
        beta: Evaluation,
        depth: Ply,
        ply: Ply,
    ) -> Evaluation {
        // TODO: Considering reordering the following:
        //     1. Transposition table check
        //     2. Depth == 0 check
        //     3. Stop check
        //     4. Game status check

        self.info.nodes += 1;

        let alpha_original = alpha;

        let mut hash_move = None;

        // Probe the transposition table
        if let Some(entry) = self.transposition_table.get(board) {
            if !PV_NODE
                && entry.depth >= depth
                && ((entry.bound == Bound::Exact)
                    || (entry.bound == Bound::Lower && entry.score >= beta)
                    || (entry.bound == Bound::Upper && entry.score <= alpha))
            {
                pv_line.clear();
                return entry.score;
            }

            hash_move = Some(entry.best_move);
        }

        // Leaf node checks

        if depth == 0 {
            pv_line.clear();
            return self.quiescence(board, -Evaluation::INFINITY, Evaluation::INFINITY);
        }

        if self.handler.stopped(self.info.nodes) {
            self.info.stopped = true;
            return Evaluation::DRAW;
        }

        let mut moves = generate_moves::<false>(board);

        match game_status(board, self.board_hashes, moves.is_empty()) {
            GameStatus::Won => {
                pv_line.clear();
                return Evaluation::mated_in(ply);
            }

            GameStatus::Drawn => {
                pv_line.clear();
                return Evaluation::DRAW;
            }

            GameStatus::Ongoing => {}
        }

        order_moves(board, &mut moves, hash_move);

        let mut best_score = -Evaluation::INFINITY;
        let mut new_line = PVLine::new();

        let mut first_move = true;

        for mv in moves {
            if self.info.stopped {
                return Evaluation::DRAW;
            }

            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            self.board_hashes.push(new_board.hash());

            let mut score = -Evaluation::INFINITY;

            // Try a zero window search
            if !PV_NODE || !first_move {
                score = -self.negamax::<false>(
                    &new_board,
                    &mut new_line,
                    -(alpha + 1),
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            }

            // Do a full window search on PV nodes or if the zero window search fails
            if PV_NODE && (first_move || score > alpha) {
                score = -self.negamax::<true>(
                    &new_board,
                    &mut new_line,
                    -beta,
                    -alpha,
                    depth - 1,
                    ply + 1,
                );
            }

            self.board_hashes.pop();

            if score > best_score {
                best_score = score;
                pv_line.extend(mv, &new_line);

                // Track the best move across all search iterations
                if ply == 0 && !self.info.stopped && score > self.best_score {
                    self.best_score = score;
                    self.best_move = Some(mv);
                }

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                break;
            }

            first_move = false;
        }

        // Store result in the transposition table
        self.transposition_table.set(
            board,
            Data {
                score: best_score,
                bound: if best_score <= alpha_original {
                    Bound::Upper
                } else if best_score >= beta {
                    Bound::Lower
                } else {
                    Bound::Exact
                },
                depth,
                best_move: pv_line
                    .first()
                    .expect("PV Line must be populated due to prior mate and draw checks"),
            },
        );

        best_score
    }

    fn quiescence(&mut self, board: &Board, mut alpha: Evaluation, beta: Evaluation) -> Evaluation {
        self.info.nodes += 1;

        // TODO: Query transposition table in quiescent search
        // TODO: Check check (evasions)
        // TODO: Check leaf nodes?

        // Stand pat

        let mut best_score = evaluate(board);

        if best_score >= beta {
            return best_score;
        }

        if best_score > alpha {
            alpha = best_score
        }

        let mut moves = generate_moves::<true>(board);
        order_moves(board, &mut moves, None);

        for mv in moves {
            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            let score = -self.quiescence(&new_board, -beta, -alpha);

            if score > best_score {
                best_score = score;

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                break;
            }
        }

        best_score
    }
}

fn generate_moves<const CAPTURES_ONLY: bool>(board: &Board) -> Vec<Move> {
    let mut all_moves = Vec::new();

    let enemies = board.colors(!board.side_to_move());

    board.generate_moves(|moves| {
        let moves = if CAPTURES_ONLY {
            // NOTE: En passant captures not included
            let mut captures = moves;
            captures.to &= enemies;

            captures
        } else {
            moves
        };

        all_moves.extend(moves);
        false
    });

    all_moves
}

fn game_status(board: &Board, board_hashes: &[u64], no_moves: bool) -> GameStatus {
    if no_moves {
        if board.checkers().is_empty() {
            // Stalemates
            return GameStatus::Drawn;
        }

        // Checkmates
        return GameStatus::Won;
    }

    if board.halfmove_clock() >= 100 {
        // 50 move rule
        return GameStatus::Drawn;
    }

    let current_hash = board.hash();
    let repetitions = board_hashes
        .iter()
        .rev()
        .take(board.halfmove_clock() as usize + 1)
        .step_by(2)
        .skip(1)
        .filter(|&&hash| hash == current_hash)
        .count();

    if repetitions >= 2 {
        // Threefold repetition
        return GameStatus::Drawn;
    }

    // TODO: Check insufficient material

    GameStatus::Ongoing
}
