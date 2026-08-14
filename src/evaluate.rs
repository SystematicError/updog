use crate::search::Ply;
use cozy_chess::{Board, Color, Piece, Square};
use std::fmt;

pub type Evaluation = i16;

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
        let mate_range = (Evaluation::MATE - Ply::MAX as Evaluation)..=Evaluation::MATE;
        let mated_range = Evaluation::MATED..=(Evaluation::MATED + Ply::MAX as Evaluation);

        if mate_range.contains(&self.0) {
            let moves = (Evaluation::MATE - self.0 + 1) / 2;
            write!(formatter, "mate {moves}")?;
        } else if mated_range.contains(&self.0) {
            let moves = (self.0 - Evaluation::MATED) / 2;
            write!(formatter, "mate -{moves}")?;
        } else {
            write!(formatter, "cp {}", self.0)?;
        }

        Ok(())
    }
}

// Reimplementation of TSCP's implementation of the PeSTO evaluation function
// https://www.tckerrigan.com/Chess/TSCP/

enum Phase {
    Mid,
    End,
}

type PieceSquareTable = [Evaluation; Square::NUM];

const fn create_table(piece: Piece, phase: Phase, mut table: PieceSquareTable) -> PieceSquareTable {
    let piece_value = match phase {
        Phase::Mid => match piece {
            Piece::Pawn => 82,
            Piece::Knight => 337,
            Piece::Bishop => 365,
            Piece::Rook => 477,
            Piece::Queen => 1025,
            Piece::King => 0,
        },

        Phase::End => match piece {
            Piece::Pawn => 94,
            Piece::Knight => 281,
            Piece::Bishop => 297,
            Piece::Rook => 512,
            Piece::Queen => 936,
            Piece::King => 0,
        },
    };

    // TODO: Use map with const closures instead
    // return table.map(|square_value| square_value + piece_value)

    let mut i = 0;
    while i < Square::NUM {
        table[i] += piece_value;
        i += 1;
    }

    table
}

#[rustfmt::skip]
const PAWN_TABLE_MID: PieceSquareTable = create_table(Piece::Pawn, Phase::Mid, [
    0,   0,   0,   0,   0,   0,  0,   0,
   98, 134,  61,  95,  68, 126, 34, -11,
   -6,   7,  26,  31,  65,  56, 25, -20,
  -14,  13,   6,  21,  23,  12, 17, -23,
  -27,  -2,  -5,  12,  17,   6, 10, -25,
  -26,  -4,  -4, -10,   3,   3, 33, -12,
  -35,  -1, -20, -23, -15,  24, 38, -22,
    0,   0,   0,   0,   0,   0,  0,   0,
]);

#[rustfmt::skip]
const PAWN_TABLE_END: PieceSquareTable = create_table(Piece::Pawn, Phase::End, [
    0,   0,   0,   0,   0,   0,   0,   0,
  178, 173, 158, 134, 147, 132, 165, 187,
   94, 100,  85,  67,  56,  53,  82,  84,
   32,  24,  13,   5,  -2,   4,  17,  17,
   13,   9,  -3,  -7,  -7,  -8,   3,  -1,
    4,   7,  -6,   1,   0,  -5,  -1,  -8,
   13,   8,   8,  10,  13,   0,   2,  -7,
    0,   0,   0,   0,   0,   0,   0,   0,
]);

#[rustfmt::skip]
const KNIGHT_TABLE_MID: PieceSquareTable = create_table(Piece::Knight, Phase::Mid, [
    -167, -89, -34, -49,  61, -97, -15, -107,
     -73, -41,  72,  36,  23,  62,   7,  -17,
     -47,  60,  37,  65,  84, 129,  73,   44,
      -9,  17,  19,  53,  37,  69,  18,   22,
     -13,   4,  16,  13,  28,  19,  21,   -8,
     -23,  -9,  12,  10,  19,  17,  25,  -16,
     -29, -53, -12,  -3,  -1,  18, -14,  -19,
    -105, -21, -58, -33, -17, -28, -19,  -23,
]);

#[rustfmt::skip]
const KNIGHT_TABLE_END: PieceSquareTable = create_table(Piece::Knight, Phase::End, [
    -58, -38, -13, -28, -31, -27, -63, -99,
    -25,  -8, -25,  -2,  -9, -25, -24, -52,
    -24, -20,  10,   9,  -1,  -9, -19, -41,
    -17,   3,  22,  22,  22,  11,   8, -18,
    -18,  -6,  16,  25,  16,  17,   4, -18,
    -23,  -3,  -1,  15,  10,  -3, -20, -22,
    -42, -20, -10,  -5,  -2, -20, -23, -44,
    -29, -51, -23, -15, -22, -18, -50, -64,
]);

#[rustfmt::skip]
const BISHOP_TABLE_MID: PieceSquareTable = create_table(Piece::Bishop, Phase::Mid, [
    -29,   4, -82, -37, -25, -42,   7,  -8,
    -26,  16, -18, -13,  30,  59,  18, -47,
    -16,  37,  43,  40,  35,  50,  37,  -2,
     -4,   5,  19,  50,  37,  37,   7,  -2,
     -6,  13,  13,  26,  34,  12,  10,   4,
      0,  15,  15,  15,  14,  27,  18,  10,
      4,  15,  16,   0,   7,  21,  33,   1,
    -33,  -3, -14, -21, -13, -12, -39, -21,
]);

#[rustfmt::skip]
const BISHOP_TABLE_END: PieceSquareTable = create_table(Piece::Bishop, Phase::End, [
    -14, -21, -11,  -8, -7,  -9, -17, -24,
     -8,  -4,   7, -12, -3, -13,  -4, -14,
      2,  -8,   0,  -1, -2,   6,   0,   4,
     -3,   9,  12,   9, 14,  10,   3,   2,
     -6,   3,  13,  19,  7,  10,  -3,  -9,
    -12,  -3,   8,  10, 13,   3,  -7, -15,
    -14, -18,  -7,  -1,  4,  -9, -15, -27,
    -23,  -9, -23,  -5, -9, -16,  -5, -17,
]);

#[rustfmt::skip]
const ROOK_TABLE_MID: PieceSquareTable = create_table(Piece::Rook, Phase::Mid, [
    32,  42,  32,  51, 63,  9,  31,  43,
    27,  32,  58,  62, 80, 67,  26,  44,
    -5,  19,  26,  36, 17, 45,  61,  16,
   -24, -11,   7,  26, 24, 35,  -8, -20,
   -36, -26, -12,  -1,  9, -7,   6, -23,
   -45, -25, -16, -17,  3,  0,  -5, -33,
   -44, -16, -20,  -9, -1, 11,  -6, -71,
   -19, -13,   1,  17, 16,  7, -37, -26,
]);

#[rustfmt::skip]
const ROOK_TABLE_END: PieceSquareTable = create_table(Piece::Rook, Phase::End, [
    13, 10, 18, 15, 12,  12,   8,   5,
    11, 13, 13, 11, -3,   3,   8,   3,
     7,  7,  7,  5,  4,  -3,  -5,  -3,
     4,  3, 13,  1,  2,   1,  -1,   2,
     3,  5,  8,  4, -5,  -6,  -8, -11,
    -4,  0, -5, -1, -7, -12,  -8, -16,
    -6, -6,  0,  2, -9,  -9, -11,  -3,
    -9,  2,  3, -1, -5, -13,   4, -20,
]);

#[rustfmt::skip]
const QUEEN_TABLE_MID: PieceSquareTable = create_table(Piece::Queen, Phase::Mid, [
    -28,   0,  29,  12,  59,  44,  43,  45,
    -24, -39,  -5,   1, -16,  57,  28,  54,
    -13, -17,   7,   8,  29,  56,  47,  57,
    -27, -27, -16, -16,  -1,  17,  -2,   1,
     -9, -26,  -9, -10,  -2,  -4,   3,  -3,
    -14,   2, -11,  -2,  -5,   2,  14,   5,
    -35,  -8,  11,   2,   8,  15,  -3,   1,
     -1, -18,  -9,  10, -15, -25, -31, -50,
]);

#[rustfmt::skip]
const QUEEN_TABLE_END: PieceSquareTable = create_table(Piece::Queen, Phase::End, [
    -9,  22,  22,  27,  27,  19,  10,  20,
   -17,  20,  32,  41,  58,  25,  30,   0,
   -20,   6,   9,  49,  47,  35,  19,   9,
     3,  22,  24,  45,  57,  40,  57,  36,
   -18,  28,  19,  47,  31,  34,  39,  23,
   -16, -27,  15,   6,   9,  17,  10,   5,
   -22, -23, -30, -16, -16, -23, -36, -32,
   -33, -28, -22, -43,  -5, -32, -20, -41,
]);

#[rustfmt::skip]
const KING_TABLE_MID: PieceSquareTable = create_table(Piece::King, Phase::Mid, [
    -65,  23,  16, -15, -56, -34,   2,  13,
     29,  -1, -20,  -7,  -8,  -4, -38, -29,
     -9,  24,   2, -16, -20,   6,  22, -22,
    -17, -20, -12, -27, -30, -25, -14, -36,
    -49,  -1, -27, -39, -46, -44, -33, -51,
    -14, -14, -22, -46, -44, -30, -15, -27,
      1,   7,  -8, -64, -43, -16,   9,   8,
    -15,  36,  12, -54,   8, -28,  24,  14,
]);

#[rustfmt::skip]
const KING_TABLE_END: PieceSquareTable = create_table(Piece::King, Phase::End, [
    -74, -35, -18, -18, -11,  15,   4, -17,
    -12,  17,  14,  17,  17,  38,  23,  11,
     10,  17,  23,  15,  20,  45,  44,  13,
     -8,  22,  24,  27,  26,  33,  26,   3,
    -18,  -4,  21,  24,  27,  23,   9, -11,
    -19,  -3,  11,  21,  23,  16,   7,  -9,
    -27, -11,   4,  13,  14,   4,  -5, -17,
    -53, -34, -21, -11, -28, -14, -24, -43
]);

fn phase_increment(piece: Piece) -> Evaluation {
    match piece {
        Piece::Pawn => 0,
        Piece::Knight => 1,
        Piece::Bishop => 1,
        Piece::Rook => 2,
        Piece::Queen => 4,
        Piece::King => 0,
    }
}

pub fn evaluate(board: &Board) -> Evaluation {
    let mut score_mid = 0;
    let mut score_end = 0;
    let mut phase = 0;

    for (piece, mid_table, end_table) in [
        (Piece::Pawn, PAWN_TABLE_MID, PAWN_TABLE_END),
        (Piece::Knight, KNIGHT_TABLE_MID, KNIGHT_TABLE_END),
        (Piece::Bishop, BISHOP_TABLE_MID, BISHOP_TABLE_END),
        (Piece::Rook, ROOK_TABLE_MID, ROOK_TABLE_END),
        (Piece::Queen, QUEEN_TABLE_MID, QUEEN_TABLE_END),
        (Piece::King, KING_TABLE_MID, KING_TABLE_END),
    ] {
        let white_pieces = board.colored_pieces(Color::White, piece);
        let black_pieces = board.colored_pieces(Color::Black, piece);

        for square in white_pieces {
            score_mid += mid_table[square.relative_to(Color::Black) as usize];
            score_end += end_table[square.relative_to(Color::Black) as usize];
            phase += phase_increment(piece);
        }

        for square in black_pieces {
            score_mid -= mid_table[square as usize];
            score_end -= end_table[square as usize];
            phase += phase_increment(piece);
        }
    }

    let phase_mid = phase.min(24);
    let phase_end = 24 - phase_mid;

    // PERF: Is int casting needed for tapared evaluation?
    let score = (score_mid as i32 * phase_mid as i32 + score_end as i32 * phase_end as i32) / 24;

    let perspective = match board.side_to_move() {
        Color::White => 1,
        Color::Black => -1,
    };

    score as Evaluation * perspective
}
