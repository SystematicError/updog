use cozy_chess::{Board, Color, Move, Piece, Rank, Square};

pub fn capture_pair(board: &Board, mv: Move) -> Option<(Piece, Piece)> {
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
        match board.piece_on(mv.to) {
            Some(piece) => Some(piece),
            None => None,
        }
    };

    victim.map(|v| (attacker, v))
}

pub fn is_capture(board: &Board, mv: Move) -> bool {
    capture_pair(board, mv).is_none()
}
