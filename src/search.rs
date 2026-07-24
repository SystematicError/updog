use crate::evaluate::{Evaluation, EvaluationUtils, evaluate};
use crate::ordering::generate_ordered_moves;
use crate::pv::PVLine;
use crate::uci::{SearchOptions, TimeOptions};
use cozy_chess::{Board, Move};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

pub type Ply = u8;

pub fn deepen(
    board: Board,
    time_options: TimeOptions,
    search_options: SearchOptions,
    stop: Arc<AtomicBool>,
) -> Option<(Board, Move)> {
    stop.store(false, Ordering::Release);

    let mut best_move = None;

    let mut pv_line = PVLine::new();
    let mut info = SearchInfo::new();

    for depth in 1..=search_options.depth.unwrap_or(Ply::MAX) {
        let score = search(&board, &mut pv_line, &mut info, &stop, depth);

        // Discard results if the iteration was stoppped
        if info.stopped {
            break;
        }

        best_move = pv_line.first();

        println!(
            "info depth {depth} nodes {} score {} pv {}",
            info.nodes,
            score.display(),
            pv_line.display(&board)
        );

        // NOTE: May not be necessary, since pv lines are constructed form the root onwards
        pv_line.clear();

        if stop.load(Ordering::Acquire) {
            break;
        }
    }

    // Only terminate infinite searches when manually stopped
    if search_options.depth.is_none() {
        while !stop.load(Ordering::Acquire) {
            sleep(Duration::from_millis(5));
        }
    }

    best_move.map(|mv| (board, mv))
}

struct SearchInfo {
    nodes: usize,
    stopped: bool,
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

fn search(
    board: &Board,
    pv_line: &mut PVLine,
    info: &mut SearchInfo,
    stop: &Arc<AtomicBool>,
    depth: Ply,
) -> Evaluation {
    info.nodes += 1;

    if depth == 0 {
        return evaluate(board);
    }

    if info.nodes.is_multiple_of(STOP_CHECK_FREQUENCY) && stop.load(Ordering::Acquire) {
        info.stopped = true;
        return Evaluation::DRAW;
    }

    let mut best_score = -Evaluation::INFINITY;
    let new_line = &mut PVLine::new();

    let moves = generate_ordered_moves(board);

    // HACK: Store best move in forced mate lines
    if moves.is_empty() {
        return best_score + 1;
    }

    for mv in moves {
        if info.stopped {
            return Evaluation::DRAW;
        }

        let mut new_board = board.clone();
        new_board.play_unchecked(mv);

        let score = -search(&new_board, new_line, info, stop, depth - 1);

        if score > best_score {
            best_score = score;
            pv_line.extend(mv, new_line);
        }
    }

    best_score
}
