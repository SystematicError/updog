use cozy_chess::{Board, Color, Move, Piece, Rank, Square};
use std::cmp::Reverse;

fn capture_pair(board: &Board, mv: Move) -> Option<(Piece, Piece)> {
    let attacker = board
        .piece_on(mv.from)
        .expect("Generated move must be legal");

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

#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum OrderingMove<T: Ord> {
    Hash,
    Capture(T),
    Other,
}

fn ordering_key(board: &Board, mv: Move, hash_move: Option<Move>) -> impl Ord {
    // Hash move
    if let Some(hash_move) = hash_move
        && hash_move == mv
    {
        return OrderingMove::Hash;
    }

    // MVV-LVA
    if let Some((attacker, victim)) = capture_pair(board, mv) {
        return OrderingMove::Capture((Reverse(victim), attacker));
    }

    OrderingMove::Other
}

pub fn order_moves(board: &Board, moves: &mut [Move], hash_move: Option<Move>) {
    moves.sort_unstable_by_key(|&mv| ordering_key(board, mv, hash_move));
}
