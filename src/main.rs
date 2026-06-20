use cozy_chess::Board;
use cozy_chess::util::{display_uci_move, parse_uci_move};
use std::io::{BufRead, stdin};
use std::process::exit;

mod engine;

enum UciCommand {
    Uci,
    IsReady,
    SetOption(String, Option<String>),
    Position(Board),
    Go,
    Quit,
}

impl UciCommand {
    fn parse(input: &str) -> Option<Self> {
        let mut tokens = input.split_ascii_whitespace();

        let command = tokens.next()?;

        Some(match command {
            "uci" => UciCommand::Uci,
            "isready" => UciCommand::IsReady,

            "setoption" => {
                if tokens.next()? != "name" {
                    return None;
                };

                let name = tokens.next()?.to_string();

                let value_token = tokens.next();

                if let None = value_token {
                    UciCommand::SetOption(name, None)
                } else if let Some("value") = value_token {
                    UciCommand::SetOption(name, tokens.next().map(|s| s.to_string()))
                } else {
                    return None;
                }
            }

            "position" => {
                let mut board = match tokens.next()? {
                    "startpos" => Board::default(),

                    "fen" => {
                        let fen: Vec<_> = tokens.by_ref().take(6).collect();

                        if fen.len() != 6 {
                            return None;
                        }

                        Board::from_fen(&fen.join(" "), false).ok()?
                    }

                    _ => return None,
                };

                let moves_token = tokens.next();

                if let None = moves_token {
                    UciCommand::Position(board)
                } else if let Some("moves") = moves_token {
                    for mv in tokens {
                        let mv = parse_uci_move(&board, mv);

                        if let Ok(mv) = mv {
                            board.play(mv);
                        } else {
                            break;
                        }
                    }

                    UciCommand::Position(board)
                } else {
                    return None;
                }
            }

            "go" => UciCommand::Go,

            "quit" => UciCommand::Quit,

            _ => return None,
        })
    }
}

fn main() {
    let mut engine = engine::Engine::new();

    for line in stdin().lock().lines() {
        if let Some(command) = UciCommand::parse(&line.unwrap()) {
            match command {
                UciCommand::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!("uciok");
                }

                UciCommand::IsReady => {
                    println!("readyok");
                }

                UciCommand::SetOption(name, value) => {}

                UciCommand::Position(board) => engine.set_board(board),

                UciCommand::Go => {
                    if let Some((board, mv)) = engine.best_move() {
                        println!("bestmove {}", display_uci_move(board, mv));
                    }
                }

                UciCommand::Quit => {
                    exit(0);
                }
            }
        }
    }
}
