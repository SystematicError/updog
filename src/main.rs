mod engine;
mod uci;

use crate::engine::Engine;
use crate::uci::Uci;
use std::io::{BufRead, stdin};
use std::process::exit;

fn main() {
    let chess960 = false;
    let mut engine = Engine::new();

    for line in stdin().lock().lines() {
        if let Some(command) = Uci::parse(&line.expect("Should be able to read line"), chess960) {
            match command {
                Uci::Uci => {
                    println!("id name Updog");
                    println!("id author SystematicError");
                    println!("option name Threads type spin default 1 min 1 max 1");
                    println!("option name Hash type spin default 1 min 1 max 1");
                    println!("uciok");
                }

                Uci::IsReady => println!("readyok"),

                Uci::NewGame => todo!(),

                Uci::SetOption(_name, _value) => todo!(),

                Uci::Position(board, moves) => engine.set_position(board, moves),

                Uci::Go(_parameters) => todo!(),

                Uci::Stop => todo!(),

                Uci::Quit => exit(0),

                Uci::D => println!("{}", engine.board()),

                Uci::Bench(_depth) => todo!(),
            }
        }
    }
}
