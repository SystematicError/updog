use cozy_chess::{Board, Color, File, Piece, Rank, Square};

pub fn display_board(board: &Board) {
    println!("┌{}┐", ["───"; File::NUM].join("┬"));

    println!(
        "{}",
        Square::ALL
            .map(|square| {
                let square = square.relative_to(Color::Black);

                let mut piece = match board.piece_on(square) {
                    Some(Piece::King) => 'k',
                    Some(Piece::Queen) => 'q',
                    Some(Piece::Rook) => 'r',
                    Some(Piece::Bishop) => 'b',
                    Some(Piece::Knight) => 'n',
                    Some(Piece::Pawn) => 'p',
                    None => ' ',
                }
                .to_string();

                if let Some(Color::White) = board.color_on(square) {
                    piece = piece.to_uppercase()
                }

                piece
            })
            .chunks(File::NUM)
            .enumerate()
            .map(|(i, rank)| format!("│ {} │ {}", rank.to_vec().join(" │ "), Rank::NUM - i))
            .collect::<Vec<_>>()
            .join(&format!("\n├{}┤\n", ["───"; File::NUM].join("┼")))
    );

    println!("└{}┘", ["───"; File::NUM].join("┴"));

    println!(
        "  {}  ",
        ('a'..='z')
            .take(File::NUM)
            .map(|c| c.to_string())
            .collect::<Vec<_>>()
            .join("   ")
    );

    println!();

    println!("Hash: {:X}", board.hash());

    println!(
        "Checkers: {}",
        board
            .checkers()
            .iter()
            .map(|square| format!("{:?}", square))
            .collect::<Vec<_>>()
            .join(" ")
    )
}
