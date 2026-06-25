use crate::captures::is_capture;
use crate::evaluate::evaluate;
use crate::ordering::generate_ordered_moves;
use crate::position::Position;
use cozy_chess::{Board, GameStatus, Move};

const MAX_SCORE: i32 = i32::MAX;
const MIN_SCORE: i32 = -MAX_SCORE;
const MATED_SCORE: i32 = MIN_SCORE + 1;
const DRAWN_SCORE: i32 = 0;

pub fn search(position: &Position) -> Option<Move> {
    let mut best_move = None;

    _search(position, MIN_SCORE, MAX_SCORE, true, &mut best_move, 4);

    best_move
}

fn _search(
    position: &Position,
    mut alpha: i32,
    beta: i32,
    root: bool,
    best_move: &mut Option<Move>,
    depth: u8,
) -> i32 {
    let moves = generate_ordered_moves(position.board());

    match position.status() {
        GameStatus::Drawn => return DRAWN_SCORE,
        GameStatus::Won => return MATED_SCORE,
        GameStatus::Ongoing => (),
    }

    if depth == 0 {
        return evaluate(position.board());
        // return quiescence(board, MIN_SCORE, MAX_SCORE);
    }

    let mut best_score = MIN_SCORE;

    for mv in moves {
        let mut new_board = position.clone();
        new_board.play_unchecked(mv);

        let score = -_search(&new_board, -beta, -alpha, false, best_move, depth - 1);

        if score > best_score {
            best_score = score;

            if score > alpha {
                alpha = score;
            }

            if root {
                *best_move = Some(mv);
            }
        }

        if !root && score >= beta {
            break;
        }
    }

    best_score
}

fn quiescence(board: &Board, mut alpha: i32, beta: i32) -> i32 {
    let mut best_score = evaluate(board);

    if best_score >= beta {
        return best_score;
    }

    if best_score > alpha {
        alpha = best_score;
    }

    for mv in generate_ordered_moves(board) {
        if !is_capture(board, mv) {
            continue;
        }

        let mut new_board = board.clone();
        new_board.play_unchecked(mv);

        let score = -quiescence(&new_board, -beta, -alpha);

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
