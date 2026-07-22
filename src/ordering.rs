use cozy_chess::{Board, Move};

// TODO: Implement move ordering
pub fn generate_ordered_moves(board: &Board) -> Vec<Move> {
    let mut ordered_moves = Vec::new();

    board.generate_moves(|moves| {
        ordered_moves.extend(moves);
        false
    });

    ordered_moves
}
