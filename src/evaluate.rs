use cozy_chess::{Board, Color, Piece};
use std::fmt;

pub type Evaluation = i32;

pub trait EvaluationUtils {
    const INFINITY: Self;
    const DRAW: Self;

    fn display(self) -> impl fmt::Display;
}

impl EvaluationUtils for Evaluation {
    const INFINITY: Self = Self::MAX;
    const DRAW: Self = 0;

    fn display(self) -> impl fmt::Display {
        EvaluationDisplay(self)
    }
}

struct EvaluationDisplay(Evaluation);

impl fmt::Display for EvaluationDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "cp {}", self.0)?;
        Ok(())
    }
}

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

// TODO: Implement evaluation
pub fn evaluate(board: &Board) -> Evaluation {
    let mut score = 0;

    for piece in [
        Piece::Queen,
        Piece::Rook,
        Piece::Bishop,
        Piece::Knight,
        Piece::Pawn,
    ] {
        // Material score

        let white_pieces = board.colored_pieces(Color::White, piece).len() as Evaluation;
        let black_pieces = board.colored_pieces(Color::Black, piece).len() as Evaluation;

        score += piece_value(piece) * (white_pieces - black_pieces);
    }

    let perspective = match board.side_to_move() {
        Color::White => 1,
        Color::Black => -1,
    };

    score * perspective
}
