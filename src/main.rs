mod engine;
mod position;
mod uci;

use crate::engine::Engine;
use crate::uci::Uci;
use cozy_chess::util::display_uci_move;
use std::io::{BufRead, stdin};
use std::process::exit;

fn main() {
    let mut engine = Engine::new();

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.unwrap()) {
            match command {
                Uci::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!("option name Threads type spin default 1 min 1 max 1");
                    println!("option name Hash type spin default 0 min 0 max 0");
                    println!("uciok");
                }

                Uci::IsReady => println!("readyok"),

                Uci::SetOption(_name, _value) => {}

                Uci::Position(position) => engine.set_position(position),

                Uci::Go => {
                    if let Some((board, mv)) = engine.best_move() {
                        println!("bestmove {}", display_uci_move(board, mv));
                    } else {
                        println!("bestmove (none)");
                    }
                }

                Uci::Quit => exit(0),
            }
        }
    }
}
