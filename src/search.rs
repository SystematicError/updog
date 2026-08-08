use crate::evaluate::{Evaluation, EvaluationUtils, evaluate};
use crate::ordering::order_moves;
use crate::pv::PVLine;
use crate::uci::SearchOptions;
use cozy_chess::{Board, GameStatus, Move};
use std::thread::sleep;
use std::time::Duration;

pub type Ply = u8;

pub struct SearchInfo {
    pub nodes: usize,
    pub stopped: bool,
}

impl SearchInfo {
    fn new() -> Self {
        Self {
            nodes: 0,
            stopped: false,
        }
    }
}

// TODO: Merge SearchResult and SearchFinalResult together?

pub struct SearchResult<'a> {
    pub board: &'a Board,
    pub depth: Ply,
    pub score: Evaluation,
    pub info: &'a SearchInfo,
    pub pv_line: &'a PVLine,
}

pub struct SearchFinalResult<'a> {
    pub board: &'a Board,
    pub best_move: Option<Move>,
    pub info: SearchInfo,
}

pub trait SearchHandler {
    fn stopped(&self, nodes: usize) -> bool;
    fn handle_result(&self, result: SearchResult);
}

pub struct Searcher<H: SearchHandler> {
    board: Board,
    board_hashes: Vec<u64>,
    handler: H,
}

impl<H: SearchHandler> Searcher<H> {
    pub fn new(board: Board, board_hashes: Vec<u64>, handler: H) -> Self {
        Self {
            board,
            board_hashes,
            handler,
        }
    }

    pub fn deepen(&mut self, search_options: SearchOptions) -> SearchFinalResult<'_> {
        let mut best_move = None;

        let mut pv_line = PVLine::new();
        let mut info = SearchInfo::new();

        for depth in 1..=search_options.depth.unwrap_or(Ply::MAX) {
            let score = search(
                &self.board,
                &mut self.board_hashes,
                &mut pv_line,
                &mut info,
                &self.handler,
                -Evaluation::INFINITY,
                Evaluation::INFINITY,
                depth,
                0,
            );

            // Discard results if the iteration was stoppped
            if info.stopped {
                break;
            }

            best_move = pv_line.first();

            self.handler.handle_result(SearchResult {
                board: &self.board,
                depth,
                score,
                info: &info,
                pv_line: &pv_line,
            });

            // NOTE: May not be necessary, since pv lines are constructed form the root onwards
            pv_line.clear();

            if self.handler.stopped(info.nodes) {
                break;
            }
        }

        // Only terminate infinite searches when manually stopped
        if search_options.depth.is_none() {
            while !self.handler.stopped(info.nodes) {
                sleep(Duration::from_millis(5));
            }
        }

        SearchFinalResult {
            board: &self.board,
            best_move,
            info,
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn search(
    board: &Board,
    board_hashes: &mut Vec<u64>,
    pv_line: &mut PVLine,
    info: &mut SearchInfo,
    handler: &impl SearchHandler,
    mut alpha: Evaluation,
    beta: Evaluation,
    depth: Ply,
    ply: Ply,
) -> Evaluation {
    info.nodes += 1;

    if depth == 0 {
        pv_line.clear();
        return quiescence(board, info, -Evaluation::INFINITY, Evaluation::INFINITY);
    }

    if handler.stopped(info.nodes) {
        info.stopped = true;
        return Evaluation::DRAW;
    }

    let mut moves = generate_moves::<false>(board);

    // NOTE: Would it be better to check this before the 0 depth check?
    match game_status(board, board_hashes, moves.is_empty()) {
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

    order_moves(board, &mut moves);

    let mut best_score = -Evaluation::INFINITY;
    let mut new_line = PVLine::new();

    for mv in moves {
        if info.stopped {
            return Evaluation::DRAW;
        }

        let mut new_board = board.clone();
        new_board.play_unchecked(mv);

        board_hashes.push(new_board.hash());
        let score = -search(
            &new_board,
            board_hashes,
            &mut new_line,
            info,
            handler,
            -beta,
            -alpha,
            depth - 1,
            ply + 1,
        );
        board_hashes.pop();

        if score > best_score {
            best_score = score;
            pv_line.extend(mv, &new_line);

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

fn quiescence(
    board: &Board,
    info: &mut SearchInfo,
    mut alpha: Evaluation,
    beta: Evaluation,
) -> Evaluation {
    info.nodes += 1;

    // Stand pat

    let mut best_score = evaluate(board);

    if best_score >= beta {
        return best_score;
    }

    if best_score > alpha {
        alpha = best_score
    }

    let mut moves = generate_moves::<true>(board);
    order_moves(board, &mut moves);

    for mv in moves {
        let mut new_board = board.clone();
        new_board.play_unchecked(mv);

        let score = -quiescence(&new_board, info, -beta, -alpha);

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

    let current_hash = board_hashes.last().unwrap();
    let repetitions = board_hashes
        .iter()
        .rev()
        .take(board.halfmove_clock() as usize + 1)
        .step_by(2)
        .skip(1)
        .filter(|&hash| hash == current_hash)
        .count();

    if repetitions >= 2 {
        // Threefold repetition
        return GameStatus::Drawn;
    }

    // TODO: Check insufficient material

    GameStatus::Ongoing
}
