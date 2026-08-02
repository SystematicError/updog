use crate::search::Ply;
use cozy_chess::{Board, Color, Piece, Square};
use std::fmt;

pub type Evaluation = i32;

pub trait EvaluationUtils {
    const INFINITY: Self;
    const DRAW: Self;
    const MATE: Self;
    const MATED: Self;

    fn mated_in(ply: Ply) -> Self;

    fn display(self) -> impl fmt::Display;
}

impl EvaluationUtils for Evaluation {
    const INFINITY: Self = Self::MAX;
    const DRAW: Self = 0;
    const MATE: Self = Self::INFINITY - 1;
    const MATED: Self = -Self::MATE;

    fn mated_in(ply: Ply) -> Self {
        Self::MATED + ply as Self
    }

    fn display(self) -> impl fmt::Display {
        EvaluationDisplay(self)
    }
}

struct EvaluationDisplay(Evaluation);

impl fmt::Display for EvaluationDisplay {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let mate_bound = Evaluation::MATE - Ply::MAX as Evaluation;
        let mated_bound = Evaluation::MATED + Ply::MAX as Evaluation;

        if (mate_bound..=Evaluation::MATE).contains(&self.0) {
            write!(formatter, "mate {}", (Evaluation::MATE - self.0 + 1) / 2)?;
        } else if (Evaluation::MATED..=mated_bound).contains(&self.0) {
            write!(formatter, "mate -{}", (self.0 - Evaluation::MATED) / 2)?;
        } else {
            write!(formatter, "cp {}", self.0)?;
        }

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

type PieceSquareTable = [Evaluation; Square::NUM];

#[rustfmt::skip]
const QUEEN_TABLE: PieceSquareTable = [
    -20,-10,-10, -5, -5,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5,  5,  5,  5,  0,-10,
     -5,  0,  5,  5,  5,  5,  0, -5,
      0,  0,  5,  5,  5,  5,  0, -5,
    -10,  5,  5,  5,  5,  5,  0,-10,
    -10,  0,  5,  0,  0,  0,  0,-10,
    -20,-10,-10, -5, -5,-10,-10,-20
];

#[rustfmt::skip]
const ROOK_TABLE: PieceSquareTable = [
    0,  0,  0,  0,  0,  0,  0,  0,
    5, 10, 10, 10, 10, 10, 10,  5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
   -5,  0,  0,  0,  0,  0,  0, -5,
    0,  0,  0,  5,  5,  0,  0,  0
];

#[rustfmt::skip]
const BISHOP_TABLE: PieceSquareTable = [
    -20,-10,-10,-10,-10,-10,-10,-20,
    -10,  0,  0,  0,  0,  0,  0,-10,
    -10,  0,  5, 10, 10,  5,  0,-10,
    -10,  5,  5, 10, 10,  5,  5,-10,
    -10,  0, 10, 10, 10, 10,  0,-10,
    -10, 10, 10, 10, 10, 10, 10,-10,
    -10,  5,  0,  0,  0,  0,  5,-10,
    -20,-10,-10,-10,-10,-10,-10,-20,
];

#[rustfmt::skip]
const KNIGHT_TABLE: PieceSquareTable = [
    -50,-40,-30,-30,-30,-30,-40,-50,
    -40,-20,  0,  0,  0,  0,-20,-40,
    -30,  0, 10, 15, 15, 10,  0,-30,
    -30,  5, 15, 20, 20, 15,  5,-30,
    -30,  0, 15, 20, 20, 15,  0,-30,
    -30,  5, 10, 15, 15, 10,  5,-30,
    -40,-20,  0,  5,  5,  0,-20,-40,
    -50,-40,-30,-30,-30,-30,-40,-50,
];

#[rustfmt::skip]
const PAWN_TABLE: PieceSquareTable = [
    0,  0,  0,  0,  0,  0,  0,  0,
   50, 50, 50, 50, 50, 50, 50, 50,
   10, 10, 20, 30, 30, 20, 10, 10,
    5,  5, 10, 25, 25, 10,  5,  5,
    0,  0,  0, 20, 20,  0,  0,  0,
    5, -5,-10,  0,  0,-10, -5,  5,
    5, 10, 10,-20,-20, 10, 10,  5,
    0,  0,  0,  0,  0,  0,  0,  0
];

pub fn evaluate(board: &Board) -> Evaluation {
    let mut score = 0;

    for (piece, table) in [
        (Piece::Queen, QUEEN_TABLE),
        (Piece::Rook, ROOK_TABLE),
        (Piece::Bishop, BISHOP_TABLE),
        (Piece::Knight, KNIGHT_TABLE),
        (Piece::Pawn, PAWN_TABLE),
    ] {
        let white_pieces = board.colored_pieces(Color::White, piece);
        let black_pieces = board.colored_pieces(Color::Black, piece);

        // Material score

        score += piece_value(piece)
            * (white_pieces.len() as Evaluation - black_pieces.len() as Evaluation);

        // Piece square score

        for square in white_pieces {
            score += table[square.relative_to(Color::Black) as usize];
        }

        for square in black_pieces {
            score -= table[square as usize];
        }
    }

    let perspective = match board.side_to_move() {
        Color::White => 1,
        Color::Black => -1,
    };

    score * perspective
}
