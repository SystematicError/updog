use crate::evaluate::{Evaluation, EvaluationUtils, evaluate};
use crate::ordering::order_moves;
use crate::pv::PVLine;
use crate::time::TimeManager;
use crate::uci::{SearchOptions, TimeOptions};
use cozy_chess::{Board, GameStatus, Move};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

pub type Ply = u8;

pub struct SearchResult {
    pub board: Board,
    pub best_move: Option<Move>,
    pub info: SearchInfo,
}

pub fn deepen<const LOG: bool>(
    board: Board,
    mut board_hashes: Vec<u64>,
    time_options: TimeOptions,
    search_options: SearchOptions,
    stop: Arc<AtomicBool>,
) -> SearchResult {
    stop.store(false, Ordering::Release);

    let mut best_move = None;

    let mut pv_line = PVLine::new();
    let mut info = SearchInfo::new();
    let time_manager = time_options.manager(&board);

    for depth in 1..=search_options.depth.unwrap_or(Ply::MAX) {
        let score = search(
            &board,
            &mut board_hashes,
            &mut pv_line,
            &mut info,
            &time_manager,
            &stop,
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

        if LOG {
            println!(
                "info depth {depth} nodes {} score {} pv {}",
                info.nodes,
                score.display(),
                pv_line.display(&board)
            );
        }

        // NOTE: May not be necessary, since pv lines are constructed form the root onwards
        pv_line.clear();

        if time_manager.stopped() || stop.load(Ordering::Acquire) {
            break;
        }
    }

    // Only terminate infinite searches when manually stopped
    if search_options.depth.is_none() {
        while !stop.load(Ordering::Acquire) {
            sleep(Duration::from_millis(5));
        }
    }

    SearchResult {
        board,
        best_move,
        info,
    }
}

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

const STOP_CHECK_FREQUENCY: usize = 1024;

#[allow(clippy::too_many_arguments)]
fn search(
    board: &Board,
    board_hashes: &mut Vec<u64>,
    pv_line: &mut PVLine,
    info: &mut SearchInfo,
    time_manager: &TimeManager,
    stop: &Arc<AtomicBool>,
    mut alpha: Evaluation,
    beta: Evaluation,
    depth: Ply,
    ply: Ply,
) -> Evaluation {
    info.nodes += 1;

    if depth == 0 {
        pv_line.clear();
        return evaluate(board);
    }

    if info.nodes.is_multiple_of(STOP_CHECK_FREQUENCY)
        && (time_manager.stopped() || stop.load(Ordering::Acquire))
    {
        info.stopped = true;
        return Evaluation::DRAW;
    }

    let mut moves = generate_moves(board);

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
    let new_line = &mut PVLine::new();

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
            new_line,
            info,
            time_manager,
            stop,
            -beta,
            -alpha,
            depth - 1,
            ply + 1,
        );
        board_hashes.pop();

        if score > best_score {
            best_score = score;
            pv_line.extend(mv, new_line);

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

fn generate_moves(board: &Board) -> Vec<Move> {
    let mut all_moves = Vec::new();

    board.generate_moves(|moves| {
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
