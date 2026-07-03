use cozy_chess::{Board, Color, Piece};

pub type Evaluation = i32;

fn piece_value(piece: Piece) -> Evaluation {
    match piece {
        Piece::King => 20000,
        Piece::Queen => 900,
        Piece::Rook => 500,
        Piece::Bishop => 330,
        Piece::Knight => 320,
        Piece::Pawn => 100,
    }
}

pub fn evaluate(board: &Board) -> Evaluation {
    let mut score = 0;

    for piece in [
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ] {
        let white_count = board.colored_pieces(Color::White, piece).len() as Evaluation;
        let black_count = board.colored_pieces(Color::Black, piece).len() as Evaluation;

        score += piece_value(piece) * (white_count - black_count);
    }

    let perspective = match board.side_to_move() {
        Color::White => 1,
        Color::Black => -1,
    };

    score * perspective
}
