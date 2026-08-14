use cozy_chess::{Board, Color, Move, Piece, Rank, Square};
use std::cmp::Reverse;

fn piece_value(piece: Piece) -> u8 {
    match piece {
        Piece::Pawn => 1,
        Piece::Knight => 2,
        Piece::Bishop => 3,
        Piece::Rook => 4,
        Piece::Queen => 5,
        Piece::King => 6,
    }
}

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

fn hash_move_key(mv: Move, hash_move: Option<Move>) -> impl Ord {
    if let Some(hash_move) = hash_move
        && hash_move == mv
    {
        return false;
    }

    true
}

fn mvv_lva_key(board: &Board, mv: Move) -> impl Ord {
    let (attacker, victim) = match capture_pair(board, mv) {
        Some(pair) => pair,
        None => return (Reverse(0), 0),
    };

    (Reverse(piece_value(victim)), piece_value(attacker))
}

pub fn order_moves(board: &Board, moves: &mut [Move], hash_move: Option<Move>) {
    moves.sort_unstable_by_key(|&mv| (hash_move_key(mv, hash_move), mvv_lva_key(board, mv)));
}
