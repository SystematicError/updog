use crate::evaluate::piece_value;
use cozy_chess::{Board, Color, Move, Piece, Rank, Square};
use std::cmp::Reverse;

fn capture_pair(board: &Board, mv: Move) -> Option<(Piece, Piece)> {
    let attacker = board.piece_on(mv.from).unwrap();

    let rank = match board.side_to_move() {
        Color::White => Rank::Sixth,
        Color::Black => Rank::Third,
    };

    let victim = if attacker == Piece::Pawn
        && let Some(file) = board.en_passant()
        && mv.to == Square::new(file, rank)
    {
        Some(Piece::Pawn)
    } else {
        board.piece_on(mv.to)
    };

    victim.map(|v| (attacker, v))
}

fn mvv_lva_key(board: &Board, mv: Move) -> impl Ord {
    let (attacker, victim) = match capture_pair(board, mv) {
        Some(pair) => pair,
        None => return (Reverse(0), 0),
    };

    (Reverse(piece_value(victim)), piece_value(attacker))
}

pub fn order_moves(board: &Board, moves: &mut [Move]) {
    moves.sort_unstable_by_key(|&mv| mvv_lva_key(board, mv));
}
