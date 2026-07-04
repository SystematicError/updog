mod display;
mod engine;
mod evaluate;
mod search;
mod uci;

use crate::display::display_board;
use crate::engine::Engine;
use crate::uci::Uci;
use cozy_chess::util::display_uci_move;
use std::io::{BufRead, stdin};
use std::process::exit;

fn main() {
    let mut chess960 = false;
    let mut engine = Engine::new();

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.unwrap(), chess960) {
            match command {
                Uci::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!("option name UCI_Chess960 type check default false");
                    println!("option name Threads type spin default 1 min 1 max 1");
                    println!("option name Hash type spin default 1 min 1 max 1");
                    println!("uciok");
                }

                Uci::IsReady => println!("readyok"),

                Uci::SetOption(name, value) => {
                    (|| {
                        match name.as_str() {
                            "UCI_Chess960" => chess960 = value?.parse().ok()?,
                            _ => {}
                        }

                        Some(())
                    })();
                }

                Uci::Position(board, history) => engine.set_position(board, history),

                Uci::Go => {
                    if let Some((board, mv)) = engine.best_move() {
                        println!("bestmove {}", display_uci_move(board, mv));
                    } else {
                        println!("bestmove (none)");
                    }
                }

                Uci::Quit => exit(0),

                Uci::D => {
                    display_board(engine.board());
                }
            }
        }
    }
}
