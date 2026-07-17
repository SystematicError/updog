use crate::search::{Ply, SearchOptions};
use cozy_chess::util::parse_uci_move;
use cozy_chess::{Board, Move};
use std::time::Duration;

#[allow(clippy::enum_variant_names)]
pub enum Uci {
    Uci,
    IsReady,
    NewGame,
    SetOption(String, Option<String>),
    Position(Board, Vec<Move>),
    Go(SearchOptions),
    Stop,
    Quit,

    // Non standard commands
    D,
    Bench(Ply),
}

const BENCH_DEFAULT_DEPTH: Ply = 7;

impl Uci {
    pub fn parse(input: &str, chess960: bool) -> Option<Self> {
        let mut tokens = input.split_whitespace();

        let command = tokens.next()?;

        let parsed = match command {
            "uci" => Self::Uci,
            "isready" => Self::IsReady,
            "ucinewgame" => Self::NewGame,

            "setoption" => {
                if tokens.next()? != "name" {
                    return None;
                }

                let name: Vec<_> = tokens.by_ref().take_while(|&t| t != "value").collect();

                if name.is_empty() {
                    return None;
                }

                let name = name.join(" ");

                let value: Vec<_> = tokens.by_ref().collect();

                let value = if value.is_empty() {
                    None
                } else {
                    Some(value.join(" "))
                };

                Self::SetOption(name, value)
            }

            "position" => {
                let board = match tokens.next()? {
                    "startpos" => Board::default(),

                    "fen" => {
                        let fen: Vec<_> = tokens.by_ref().take(6).collect();

                        if fen.len() != 6 {
                            return None;
                        }

                        Board::from_fen(&fen.join(" "), chess960).ok()?
                    }

                    _ => return None,
                };

                if let Some(token) = tokens.next()
                    && token != "moves"
                {
                    return None;
                }

                let mut moves = Vec::new();
                let mut current_board = board.clone();

                // Apply all moves, or until a malformed or illegal move is encountered
                tokens
                    .by_ref()
                    .map_while(|mv| {
                        let mv = parse_uci_move(&current_board, mv).ok()?;
                        current_board.try_play(mv).ok()?;
                        moves.push(mv);
                        Some(())
                    })
                    .count();

                // Consume all tokens after any malformed or illegal move
                tokens.by_ref().count();

                Self::Position(board, moves)
            }

            "go" => {
                let mut options = SearchOptions::default();

                while let Some(token) = tokens.next() {
                    match token {
                        "wtime" => {
                            options.wtime = Duration::from_millis(tokens.next()?.parse().ok()?);
                        }

                        "btime" => {
                            options.btime = Duration::from_millis(tokens.next()?.parse().ok()?);
                        }

                        "winc" => {
                            options.winc = Duration::from_millis(tokens.next()?.parse().ok()?);
                        }

                        "binc" => {
                            options.binc = Duration::from_millis(tokens.next()?.parse().ok()?);
                        }

                        "movetime" => {
                            options.movetime = Duration::from_millis(tokens.next()?.parse().ok()?);
                        }

                        "infinite" => options.infinite = true,

                        "depth" => {
                            options.depth = tokens.next()?.parse().ok()?;
                        }

                        "nodes" => {
                            options.nodes = tokens.next()?.parse().ok()?;
                        }

                        _ => return None,
                    }
                }

                Self::Go(options)
            }

            "stop" => Self::Stop,
            "quit" => Self::Quit,

            "d" => Self::D,

            "bench" => Self::Bench(match tokens.next() {
                Some(depth) => depth.parse().ok()?,
                None => BENCH_DEFAULT_DEPTH,
            }),

            _ => return None,
        };

        // Ensure all tokens have been consumed
        if tokens.next().is_some() {
            return None;
        }

        Some(parsed)
    }
}
