use crate::evaluate::{Evaluation, evaluate};
use crate::principal_variation::PVLine;
use cozy_chess::{Board, GameStatus, Move};

pub type Depth = u8;

struct SearchInfo {
    pub nodes: usize,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self { nodes: 0 }
    }
}

pub fn deepen(board: &Board, history: &mut Vec<u64>, pv_line: &mut PVLine) -> Option<Move> {
    for depth in 1..=4 {
        pv_line.clear();
        let info = &mut SearchInfo::new();

        let score = negamax(
            board,
            history,
            pv_line,
            info,
            -Evaluation::MAX,
            Evaluation::MAX,
            depth,
        );

        println!(
            "info depth {} score cp {} nodes {} pv {}",
            depth,
            score,
            info.nodes,
            pv_line
                .moves()
                .iter()
                .map(|&mv| cozy_chess::util::display_uci_move(board, mv).to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    pv_line.first()
}

fn negamax(
    board: &Board,
    history: &mut Vec<u64>,
    pv_line: &mut PVLine,
    info: &mut SearchInfo,
    mut alpha: Evaluation,
    beta: Evaluation,
    depth: Depth,
) -> Evaluation {
    info.nodes += 1;

    match game_status(board, history) {
        GameStatus::Won => return -Evaluation::MAX + 1,
        GameStatus::Drawn => return 0,
        GameStatus::Ongoing => {}
    }

    if depth == 0 {
        return evaluate(board);
    }

    let mut best_score = -Evaluation::MAX;
    let new_line = &mut PVLine::new();

    board.generate_moves(|moves| {
        for mv in moves {
            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            history.push(new_board.hash());
            let score = -negamax(
                &new_board,
                history,
                new_line,
                info,
                -beta,
                -alpha,
                depth - 1,
            );
            history.pop();

            if score > best_score {
                best_score = score;
                pv_line.extend(mv, &new_line);

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                return true;
            }
        }

        false
    });

    best_score
}

fn game_status(board: &Board, history: &mut Vec<u64>) -> GameStatus {
    let status = board.status();

    if status != GameStatus::Ongoing {
        return status;
    }

    let current_hash = board.hash();

    let repetitions = history
        .iter()
        .rev()
        .take(board.halfmove_clock() as usize + 1)
        .step_by(2)
        .filter(|&&hash| hash == current_hash)
        .count();

    if repetitions >= 3 {
        return GameStatus::Drawn;
    }

    GameStatus::Ongoing
}
