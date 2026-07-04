use crate::evaluate::{Evaluation, evaluate};
use cozy_chess::{Board, GameStatus, Move};

type Depth = u8;

struct PVLine {
    size: usize,
    line: [Move; Depth::MAX as usize],
}

impl PVLine {
    pub fn new() -> Self {
        Self {
            size: 0,
            line: [unsafe { std::mem::zeroed() }; Depth::MAX as usize],
        }
    }

    pub fn moves(&self) -> &[Move] {
        &self.line[..self.size]
    }

    pub fn first(&self) -> Option<Move> {
        if self.size <= 0 {
            return None;
        }

        Some(self.line[0])
    }

    fn extend(&mut self, mv: Move, new_line: &Self) {
        self.line[0] = mv;
        self.line[1..=new_line.size].copy_from_slice(&new_line.line[..new_line.size]);
        self.size = new_line.size + 1;
    }
}

struct SearchInfo {
    pub nodes: usize,
}

impl SearchInfo {
    pub fn new() -> Self {
        Self { nodes: 0 }
    }
}

pub fn deepen(board: &Board, history: &mut Vec<u64>) -> Option<Move> {
    let mut best_move = None;

    for depth in 1..=4 {
        let pv_line = &mut PVLine::new();
        let info = &mut SearchInfo::new();

        let score = negamax(
            board,
            history,
            pv_line,
            info,
            -Evaluation::MAX,
            Evaluation::MAX,
            depth,
        );

        best_move = pv_line.first();

        println!(
            "info depth {} score cp {} nodes {} pv {}",
            depth,
            score,
            info.nodes,
            pv_line
                .moves()
                .iter()
                .map(|&mv| cozy_chess::util::display_uci_move(board, mv).to_string())
                .collect::<Vec<_>>()
                .join(" "),
        );
    }

    best_move
}

fn negamax(
    board: &Board,
    history: &mut Vec<u64>,
    pv_line: &mut PVLine,
    info: &mut SearchInfo,
    mut alpha: Evaluation,
    beta: Evaluation,
    depth: Depth,
) -> Evaluation {
    info.nodes += 1;

    match game_status(board, history) {
        GameStatus::Won => return -Evaluation::MAX + 1,
        GameStatus::Drawn => return 0,
        GameStatus::Ongoing => {}
    }

    if depth == 0 {
        return evaluate(board);
    }

    let mut best_score = -Evaluation::MAX;
    let new_line = &mut PVLine::new();

    board.generate_moves(|moves| {
        for mv in moves {
            let mut new_board = board.clone();
            new_board.play_unchecked(mv);

            history.push(new_board.hash());
            let score = -negamax(
                &new_board,
                history,
                new_line,
                info,
                -beta,
                -alpha,
                depth - 1,
            );
            history.pop();

            if score > best_score {
                best_score = score;
                pv_line.extend(mv, &new_line);

                if score > alpha {
                    alpha = score;
                }
            }

            if score >= beta {
                return true;
            }
        }

        false
    });

    best_score
}

fn game_status(board: &Board, history: &mut Vec<u64>) -> GameStatus {
    let status = board.status();

    if status != GameStatus::Ongoing {
        return status;
    }

    let current_hash = board.hash();

    let repetitions = history
        .iter()
        .rev()
        .take(board.halfmove_clock() as usize + 1)
        .step_by(2)
        .filter(|&&hash| hash == current_hash)
        .count();

    if repetitions >= 3 {
        return GameStatus::Drawn;
    }

    GameStatus::Ongoing
}
