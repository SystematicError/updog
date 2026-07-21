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

pub fn deepen(board: &Board, options: SearchOptions) -> Option<Move> {
    let mut pv_line = PVLine::new();
    let mut info = SearchInfo::new();

    for depth in 1..=options.depth {
        let score = search(board, &mut pv_line, &mut info, depth);

        println!(
            "info depth {depth} nodes {} score {} pv {}",
            info.nodes,
            score.display(),
            pv_line.display(board)
        );

        // NOTE: May not be necessary, since pv lines are constructed form the root onwards
        pv_line.clear();
    }

    pv_line.first()
}

struct SearchInfo {
    nodes: usize,
}

impl SearchInfo {
    fn new() -> Self {
        Self { nodes: 0 }
    }
}

fn search(board: &Board, pv_line: &mut PVLine, info: &mut SearchInfo, depth: Ply) -> Evaluation {
    info.nodes += 1;

    if depth == 0 {
        return evaluate(board);
    }

    let mut best_score = -Evaluation::INFINITY;
    let new_line = &mut PVLine::new();

    for mv in generate_ordered_moves(board) {
        let mut new_board = board.clone();
        new_board.play_unchecked(mv);

        let score = -search(&new_board, new_line, info, depth - 1);

        if score > best_score {
            best_score = score;
            pv_line.extend(mv, new_line);
        }
    }

    best_score
}
