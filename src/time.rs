use crate::uci::TimeOptions;
use cozy_chess::{Board, Color};
use std::time::{Duration, Instant};

pub enum TimeManager {
    Infinite,
    Fixed { start: Instant, limit: Duration },
}

impl TimeManager {
    pub fn stopped(&self) -> bool {
        match self {
            TimeManager::Infinite => false,
            TimeManager::Fixed { start, limit } => start.elapsed() >= *limit,
        }
    }
}

impl TimeOptions {
    pub fn manager(self, board: &Board) -> TimeManager {
        let start = Instant::now();

        match self {
            TimeOptions::Infinite => TimeManager::Infinite,
            TimeOptions::MoveTime(limit) => TimeManager::Fixed { start, limit },

            TimeOptions::Clock {
                wtime,
                btime,
                winc,
                binc,
            } => TimeManager::Fixed {
                start,
                limit: clock_time_limit(board, wtime, btime, winc, binc),
            },
        }
    }
}

fn clock_time_limit(
    board: &Board,
    wtime: Duration,
    btime: Duration,
    winc: Duration,
    binc: Duration,
) -> Duration {
    let base = match board.side_to_move() {
        Color::White => wtime,
        Color::Black => btime,
    };

    let increment = match board.side_to_move() {
        Color::White => winc,
        Color::Black => binc,
    };

    (base / 20) + (increment / 2)
}
